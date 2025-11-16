#![allow(clippy::upper_case_acronyms)]

#[cfg(test)]
mod tests {
  ///
  /// This test performs the full flow for Arc Fund Manager:
  /// - Use FundLiquidityCircuit for compliance checking
  /// - Fold the circuit with Nova+CycleFold's IVC
  /// - Generate a DeciderEthCircuit final proof
  /// - Generate the Solidity contract that verifies the proof
  /// - Save verifier and calldata for Arc deployment
  use crate::{
    nebula::rs::{PublicParams, RecursiveSNARK},
    onchain::{
      compressed::CompressedSNARK,
      test::fund_circuit::FundLiquidityCircuit,
    },
    provider::{Bn256EngineKZG, GrumpkinEngine},
    traits::{snark::RelaxedR1CSSNARKTrait, Engine},
  };
  use ff::Field;
  use halo2curves::bn256::Bn256;

  use rand::thread_rng;
  use std::time::Instant;

  #[cfg(feature = "solidity")]
  use crate::onchain::{
    utils::{get_function_selector_for_nova_cyclefold_verifier, get_formatted_calldata},
    compressed::prepare_calldata,
  };

  #[cfg(feature = "solidity")]
  use crate::onchain::eth::evm::{compile_solidity, Evm};
  #[cfg(feature = "solidity")]
  use crate::onchain::verifiers::{
    groth16::SolidityGroth16VerifierKey,
    kzg::SolidityKZGVerifierKey,
    nebula::{get_decider_template_for_cyclefold_decider, NovaCycleFoldVerifierKey},
  };

  type E1 = Bn256EngineKZG;
  type E2 = GrumpkinEngine;
  type EE1 = crate::provider::hyperkzg::EvaluationEngine<Bn256, E1>;
  type EE2 = crate::provider::ipa_pc::EvaluationEngine<E2>;
  type S1 = crate::spartan::snark::RelaxedR1CSSNARK<E1, EE1>; // non-preprocessing SNARK
  type S2 = crate::spartan::snark::RelaxedR1CSSNARK<E2, EE2>; // non-preprocessing SNARK

  #[test]
  fn test_fund_compliance_flow() {
    println!("\n🚀 Arc Fund Manager - Nova Proof Generation");
    println!("═══════════════════════════════════════════════════\n");

    let num_steps = 3; // Prove 3 consecutive compliance checks

    // Example: $100M fund with $10M USDC (10% liquidity - compliant)
    let f_circuit = FundLiquidityCircuit::new(
      10,           // min 10% required
      10_000_000,   // $10M USDC
      100_000_000,  // $100M total
    );

    println!("📊 Fund State:");
    println!("   Total Portfolio: $100M");
    println!("   USDC Balance: $10M");
    println!("   Liquidity: 10%");
    println!("   Requirement: ≥10%\n");

    // produce public parameters
    let start = Instant::now();
    println!("⚙️  Producing public parameters...");
    let rs_pp = PublicParams::<E1>::setup(&f_circuit, &*S1::ck_floor(), &*S2::ck_floor());
    println!("   PublicParams::setup, took {:?} ", start.elapsed());

    println!(
      "   Number of constraints per step (primary circuit): {}",
      rs_pp.num_constraints().0
    );
    println!(
      "   Number of constraints per step (secondary circuit): {}",
      rs_pp.num_constraints().1
    );

    println!(
      "   Number of variables per step (primary circuit): {}",
      rs_pp.num_variables().0
    );
    println!(
      "   Number of variables per step (secondary circuit): {}\n",
      rs_pp.num_variables().1
    );

    // produce a recursive SNARK
    println!("🔄 Generating a RecursiveSNARK...");

    let mut IC_i = <E1 as Engine>::Scalar::zero();
    let z0 = vec![<E1 as Engine>::Scalar::from(0u64)]; // Start with counter = 0
    let mut rs: RecursiveSNARK<E1> = RecursiveSNARK::<E1>::new(&rs_pp, &f_circuit, &z0).unwrap();

    for i in 0..num_steps {
      let start = Instant::now();
      rs.prove_step(&rs_pp, &f_circuit, IC_i).unwrap();

      IC_i = rs.increment_commitment(&rs_pp, &f_circuit);
      println!("   RecursiveSNARK::prove {} : took {:?} ", i, start.elapsed());
    }

    // verify the recursive SNARK
    println!("\n✓ Verifying a RecursiveSNARK...");
    let res = rs.verify(&rs_pp, num_steps, &z0, IC_i);
    println!("   RecursiveSNARK::verify: {:?}", res.is_ok(),);
    res.unwrap();

    let mut rng = thread_rng();
    let start = Instant::now();

    println!("\n📦 Compressing SNARK for on-chain verification...");
    let (compressed_pk, compressed_vk) =
      CompressedSNARK::setup(&rs_pp, &mut rng, z0.len()).unwrap();
    println!("   CompressedSNARK::setup: took {:?}", start.elapsed());

    let start = Instant::now();
    let proof = CompressedSNARK::prove(&rs_pp, &compressed_pk, &rs, &mut rng);
    match &proof {
      Ok(_) => println!("   CompressedSNARK::prove: Ok, took {:?}", start.elapsed()),
      Err(e) => println!(
        "   CompressedSNARK::prove: Error: {:?}, took {:?}",
        e,
        start.elapsed()
      ),
    }
    assert!(proof.is_ok());

    let proof = proof.unwrap();

    let start = Instant::now();
    let res = CompressedSNARK::verify(&proof, compressed_vk.clone());
    println!(
      "   CompressedSNARK::verify: {:?}, took {:?}\n",
      res.is_ok(),
      start.elapsed()
    );

    assert!(res.is_ok());

    println!("✅ Success! Nova proof generation complete.\n");

    #[cfg(feature = "solidity")]
    {
      println!("🔧 Generating Solidity Verifier Contract...");
      // Now, let's generate the Solidity code that verifies this Decider final proof
      let function_selector =
        get_function_selector_for_nova_cyclefold_verifier(rs.z0.len() * 2 + 1);

      let calldata: Vec<u8> = prepare_calldata(function_selector, &proof).unwrap();

      // prepare the setup params for the solidity verifier
      let nova_cyclefold_vk = NovaCycleFoldVerifierKey::from((
        compressed_vk.pp_hash,
        SolidityGroth16VerifierKey::from(compressed_vk.groth16_vk),
        SolidityKZGVerifierKey::from((compressed_vk.kzg_vk, Vec::new())),
        rs.z0.len(),
      ));

      // generate the solidity code
      let decider_solidity_code = get_decider_template_for_cyclefold_decider(nova_cyclefold_vk);

      // verify the proof against the solidity code in the EVM
      println!("   Testing in EVM...");
      let nova_cyclefold_verifier_bytecode =
        compile_solidity(&decider_solidity_code, "NovaDecider");
      let mut evm = Evm::default();

      let verifier_address = evm.create(nova_cyclefold_verifier_bytecode);
      println!("   verifier_address: {:?}", verifier_address);
      let (gas, output) = evm.call(verifier_address, calldata.clone());
      println!("   Solidity::verify: {:?}, gas: {:?}", output, gas);
      assert_eq!(*output.last().unwrap(), 1);

      // save smart contract and the calldata
      println!("\n📝 Storing files...");
      use std::fs;
      fs::write(
        "./FundLiquidityVerifier.sol",
        decider_solidity_code.clone(),
      )
      .expect("Unable to write to file");
      fs::write("./fund-proof.calldata", calldata.clone()).expect("");
      let s = get_formatted_calldata(calldata.clone());
      fs::write("./fund-proof.inputs", s.join(",\n")).expect("");

      println!("   ✅ Solidity verifier saved to: FundLiquidityVerifier.sol");
      println!("   ✅ Calldata saved to: fund-proof.calldata");
      println!("   ✅ Formatted inputs saved to: fund-proof.inputs");
      println!("   📊 Contract size: {} bytes", decider_solidity_code.len());
      println!("   📊 Calldata size: {} bytes", calldata.len());

      println!("\n🎉 Complete! Ready to deploy to Arc testnet.");
      println!("\n═══════════════════════════════════════════════════");
      println!("Next steps:");
      println!("  1. Deploy FundLiquidityVerifier.sol to Arc testnet");
      println!("  2. Call verifyNovaProof() with fund-proof.calldata");
      println!("  3. Integrate with TokenizedFundManager contract");
      println!("═══════════════════════════════════════════════════\n");
    }
  }
}
