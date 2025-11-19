# CompositeFundVerifier Deployment Guide

## 📋 What's Ready

Your Nova folding proof system is fully generated and ready for deployment:

✅ **Generated Files:**
- `CompositeFundVerifier.sol` (37KB) - Solidity verifier contract
- `composite-proof.calldata` (900 bytes) - Proof data for verification
- `composite-proof.inputs` (1.9KB) - Human-readable proof inputs
- `deploy.sh` - Automated deployment script

✅ **Proof Details:**
- Proves 3 consecutive compliance checks across all 3 rules
- Position Limit: 35% ≤ 40% ✓
- Liquidity: 10% ≥ 10% ✓
- Whitelist: Asset verified ✓
- Gas cost: ~795,738 gas (~$0.02)

## 🚀 Deployment Options

### Option 1: Using the Deployment Script (Recommended)

```bash
cd /home/hshadab/arc-verifier/sonobe

# Set your private key
export PRIVATE_KEY=0xYourPrivateKeyHere

# Run deployment
./deploy.sh
```

The script will:
1. Check your balance on Arc testnet
2. Deploy the CompositeFundVerifier.sol contract
3. Display the deployed contract address
4. Save deployment info to `deployment-info.txt`
5. Provide next steps for verification

### Option 2: Manual Deployment with Forge

```bash
cd /home/hshadab/arc-verifier/sonobe

# Deploy directly
/home/hshadab/.foundry/bin/forge create CompositeFundVerifier.sol:NovaDecider \
    --rpc-url https://rpc.testnet.arc.network \
    --private-key $PRIVATE_KEY \
    --legacy
```

### Option 3: Using Environment File

Create a `.env` file:

```bash
cat > .env <<EOF
ARC_TESTNET_RPC_URL=https://rpc.testnet.arc.network
ARC_CHAIN_ID=5042002
PRIVATE_KEY=0xYourPrivateKeyHere
EOF

# Load environment
source .env

# Deploy
./deploy.sh
```

## 🔐 Getting a Private Key

If you need a test account:

```bash
# Generate a new account
/home/hshadab/.foundry/bin/cast wallet new

# Or use an existing account from your wallet (MetaMask, etc.)
```

**⚠️ Security Note:** Never commit your private key to version control!

## 🧪 Testing the Deployment

After deployment, verify your proof:

```bash
# Replace <CONTRACT_ADDRESS> with your deployed address
/home/hshadab/.foundry/bin/cast call <CONTRACT_ADDRESS> \
    'verifyNovaProof(uint256[28])' \
    $(cat composite-proof.calldata) \
    --rpc-url https://rpc.testnet.arc.network

# Expected output: 0x0000000000000000000000000000000000000000000000000000000000000001
# (This is 'true' in Solidity)
```

## 📊 Arc Testnet Details

- **Network Name:** Arc Testnet
- **RPC URL:** https://rpc.testnet.arc.network
- **Chain ID:** 5042002
- **Block Explorer:** https://arc-sepolia.explorer.alchemy.com
- **Faucet:** (Check Arc documentation for testnet tokens)

## 📁 File Locations

All files are in: `/home/hshadab/arc-verifier/sonobe/`

```
sonobe/
├── CompositeFundVerifier.sol    # Verifier contract (ready to deploy)
├── composite-proof.calldata     # Proof data (900 bytes)
├── composite-proof.inputs       # Formatted proof inputs
├── deploy.sh                    # Deployment script
├── deployment-info.txt          # (Created after deployment)
└── persisted_params/            # Reusable parameters (90MB)
    ├── decider_pp.bin           # Groth16 prover params
    ├── decider_vp.bin           # Groth16 verifier params
    ├── nova_prover_params.bin   # Nova prover params
    ├── nova_cs_vp.bin           # Nova commitment scheme params
    └── nova_cf_cs_vp.bin        # Nova CycleFold params
```

## 🔄 Generating New Proofs

To generate a new proof with different parameters:

```bash
cd /home/hshadab/arc-verifier/sonobe

# Edit fund_compliance_full_flow.rs to change parameters
# Then run:
cargo run --release --example fund_compliance_full_flow

# This will:
# - Use cached parameters (18s load time)
# - Generate new proof (~21s total)
# - Update CompositeFundVerifier.sol (if parameters change)
# - Update composite-proof.calldata
```

## ✅ Current Status

- [x] Rust 1.88.0 installed
- [x] Sonobe compiled (781 dependencies)
- [x] Parameters generated and cached (90MB)
- [x] Proof generated and verified locally
- [x] Solidity verifier contract generated
- [x] Foundry installed and configured
- [x] Deployment script created
- [ ] **Deploy to Arc testnet** ← You are here
- [ ] Verify proof on-chain
- [ ] Document deployment address

## 🆘 Troubleshooting

**"No .env file found"**
- Create one using Option 3 above

**"Insufficient funds"**
- Get Arc testnet tokens from faucet
- Check balance: `/home/hshadab/.foundry/bin/cast balance <YOUR_ADDRESS> --rpc-url https://rpc.testnet.arc.network`

**"Invalid private key"**
- Ensure it starts with `0x`
- Don't include quotes: `PRIVATE_KEY=0x123...` not `PRIVATE_KEY="0x123..."`

**"Contract too large"**
- Use `--legacy` flag (already in deploy.sh)

## 📞 Next Steps After Deployment

1. Save your deployed contract address
2. Test proof verification on-chain
3. Integrate with your fund management application
4. Generate new proofs as fund state changes
5. Verify all new proofs use the same verifier (parameters must match)

---

**Generated by:** Sonobe v0.1.0 (Nova + CycleFold + Groth16)
**Proof Type:** Composite 3-rule compliance check
**Folding Steps:** 3
**Verification Cost:** ~795,738 gas (~$0.02)
**Parameters:** Reusable (cached in persisted_params/)
