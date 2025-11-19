# 🚀 Quick Deploy to Arc Testnet

## 📦 What's Ready

All files are generated and ready in `/home/hshadab/arc-verifier/sonobe/`:

```
✅ CompositeFundVerifier.sol  (37KB)  - Your verifier contract
✅ composite-proof.calldata   (900B)  - Proof data
✅ composite-proof.inputs     (1.9KB) - Readable inputs
✅ deploy.sh                          - Deployment script
✅ .env.template                      - Config template
✅ persisted_params/          (90MB)  - Cached parameters
```

## ⚡ Deploy Now (3 commands)

```bash
cd /home/hshadab/arc-verifier/sonobe

# 1. Set your private key
export PRIVATE_KEY=0xYourPrivateKeyHere

# 2. Deploy!
./deploy.sh
```

**That's it!** The script handles everything:
- Checks Arc testnet connectivity ✓
- Verifies your balance
- Deploys the contract
- Shows the contract address
- Saves deployment info

## 🔑 Need a Private Key?

### Option A: Generate New Test Account
```bash
/home/hshadab/.foundry/bin/cast wallet new
# Save the private key, then get Arc testnet tokens
```

### Option B: Export from MetaMask
1. MetaMask → Account Details → Export Private Key
2. Copy the key (starts with 0x)
3. Use in command above

### Option C: Create .env File
```bash
# Copy template
cp .env.template .env

# Edit with your key
nano .env

# Load it
source .env

# Deploy
./deploy.sh
```

## 📍 Arc Testnet Info

- **RPC:** https://rpc.testnet.arc.network
- **Chain ID:** 5042002
- **Explorer:** https://arc-sepolia.explorer.alchemy.com
- **Status:** ✅ Connected (verified)

## 🧪 After Deployment

Test your proof verification:

```bash
# The deploy script will show you this command with your contract address
/home/hshadab/.foundry/bin/cast call <CONTRACT_ADDRESS> \
    'verifyNovaProof(uint256[28])' \
    $(cat composite-proof.calldata) \
    --rpc-url https://rpc.testnet.arc.network
```

Expected result: `0x0000...0001` (true)

## 💡 What This Proves

Your proof verifies 3 consecutive periods of:
- ✅ Position limit: 35% ≤ 40%
- ✅ Liquidity requirement: 10% ≥ 10%
- ✅ Whitelist compliance: All assets verified
- ✅ All folded into ONE proof (795,738 gas)

## 📚 More Details

See `DEPLOYMENT_README.md` for:
- Troubleshooting
- Manual deployment options
- Generating new proofs
- Integration guide

---

**Ready when you are!** Just need your private key to deploy. 🚀
