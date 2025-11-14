Nova/Sonobe Integration Notes

Overview
- Circuits in this repo use bellpepper. For on-chain verification, we leverage Sonobe’s Nova + CycleFold with KZG10 and Groth16 compression (BN254), which generates a Solidity verifier and calldata.
- The arecibo crate in this repo exposes compatible traits/CS helpers and Solidity templates (askama) that are used by Sonobe’s verifier generation flow.

What We Use Today
- Proof system: Nova IVC (Sonobe)
- Curves: BN254 (EVM-friendly)
- Commitments: KZG10
- Final compression: Groth16 (DeciderEth)
- Solidity verifier: auto-generated (NovaDecider)

File Pointers
- circuits/examples/nova_proof_with_verifier.rs and generate_real_nova_proof.rs show end-to-end Nova usage and generation of Solidity verifier and calldata.
- sonobe/FundLiquidityVerifier.sol and sonobe/fund-proof.calldata are outputs from running the Sonobe example for the liquidity circuit.
- contracts/src/TokenizedFundManager.sol calls the on-chain Nova verifier via INovaDecider in _verifyLiquidity.

Arecibo vs Sonobe
- Sonobe is used for the stable Nova + compression path producing EVM-verifiable decider proofs. The arecibo module here is a supporting dependency for circuit traits and Solidity templates; the solver/prover path used is Sonobe’s.

Next Steps
- Generate and deploy verifiers for the PositionLimit and Whitelist circuits (update _verifyPositionLimit and _verifyWhitelist accordingly).
- Replace simplified Merkle hashing used in circuits with Poseidon before deploying a whitelist verifier.

