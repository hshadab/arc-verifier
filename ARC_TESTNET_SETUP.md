# Arc Testnet Setup Guide

## 1. Network Configuration

### Primary RPC Endpoint (Official)
```
https://rpc.testnet.arc.network
```

### Alternative RPC (Thirdweb)
```
https://5042002.rpc.thirdweb.com
```

### Chain Details
- **Chain ID:** `5042002`
- **Network Name:** Arc Testnet
- **Native Gas Token:** USDC
- **Block Explorer:** https://testnet.arcscan.app
- **Status:** Public Testnet (Live since Oct 28, 2025)

---

## 2. Add Arc Testnet to MetaMask

### Manual Configuration
1. Open MetaMask
2. Click Network dropdown → "Add Network" → "Add Network Manually"
3. Enter these details:

```
Network Name:      Arc Testnet
RPC URL:           https://rpc.testnet.arc.network
Chain ID:          5042002
Currency Symbol:   USDC
Block Explorer:    https://testnet.arcscan.app
```

4. Save

### Or Use This .env Config
```bash
# Arc Testnet Configuration
ARC_TESTNET_RPC_URL="https://rpc.testnet.arc.network"
ARC_CHAIN_ID=5042002
ARC_EXPLORER="https://testnet.arcscan.app"
```

---

## 3. Get Test USDC (Faucet)

### Circle Official Faucet
**URL:** https://faucet.circle.com

### Steps:
1. Go to https://faucet.circle.com
2. Select **"USDC"** as stablecoin
3. Choose **"Arc Testnet"** from network dropdown
4. Enter your wallet address
5. Click **"Send 10 USDC"**

### Limits:
- **Amount:** 10 USDC per request
- **Rate Limit:** 1 request per hour per address
- **Need More?** Request on Circle's Discord

### Test Immediately:
After receiving tokens, verify on block explorer:
```
https://testnet.arcscan.app/address/YOUR_ADDRESS
```

---

## 4. Create/Import Wallet for Deployment

### Option A: Generate New Wallet
```bash
# Using cast (Foundry)
cast wallet new

# Or using OpenSSL
openssl rand -hex 32 > .private_key
```

### Option B: Use Existing Wallet
Export your private key from MetaMask:
1. MetaMask → Account Details → Export Private Key
2. Enter password
3. Copy private key (keep it secure!)

### Store Securely
```bash
# Create .env file (DO NOT COMMIT TO GIT!)
echo "PRIVATE_KEY=0xYOUR_PRIVATE_KEY_HERE" > .env
echo ".env" >> .gitignore
```

---

## 5. Verify Connection

### Using curl
```bash
curl -X POST https://rpc.testnet.arc.network \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}'

# Should return: {"jsonrpc":"2.0","id":1,"result":"0x4cf5f2"}
# (0x4cf5f2 in hex = 5042002 in decimal)
```

### Using cast (Foundry)
```bash
cast chain-id --rpc-url https://rpc.testnet.arc.network
# Should return: 5042002

cast block-number --rpc-url https://rpc.testnet.arc.network
# Should return current block number
```

### Check Balance
```bash
cast balance YOUR_ADDRESS --rpc-url https://rpc.testnet.arc.network
# After faucet, should show 10000000 (10 USDC with 6 decimals)
```

---

## 6. Foundry Configuration

### foundry.toml
```toml
[profile.default]
src = "contracts/src"
out = "contracts/out"
libs = ["contracts/lib"]
solc_version = "0.8.20"

[rpc_endpoints]
arc_testnet = "https://rpc.testnet.arc.network"

[etherscan]
arc_testnet = { key = "${ARCSCAN_API_KEY}", url = "https://testnet.arcscan.app/api" }
```

### Deploy Script Example
```bash
forge create \
  --rpc-url $ARC_TESTNET_RPC_URL \
  --private-key $PRIVATE_KEY \
  --verify \
  contracts/TokenizedFundManager.sol:TokenizedFundManager
```

---

## 7. Important Notes

### Gas Fees
- **Paid in USDC** (not ETH!)
- Predictable and low cost
- Make sure you have test USDC before deploying

### Network Stability
- Arc is in public testnet phase
- May experience instability or downtime
- Not for production use yet

### Sub-second Finality
- Transactions confirm in <1 second
- Instant settlement for testing
- Powered by Malachite consensus

---

## 8. Quick Setup Commands

```bash
# 1. Create new wallet
cast wallet new > wallet.txt

# 2. Set environment
export PRIVATE_KEY=0xYOUR_KEY
export ARC_TESTNET_RPC_URL=https://rpc.testnet.arc.network

# 3. Get your address
cast wallet address --private-key $PRIVATE_KEY

# 4. Go to faucet (manual step)
open https://faucet.circle.com

# 5. Verify balance
cast balance $(cast wallet address --private-key $PRIVATE_KEY) \
  --rpc-url $ARC_TESTNET_RPC_URL
```

---

## 9. Next Steps for Our Project

Once you have:
- ✅ Wallet created/imported
- ✅ Test USDC received (10 USDC minimum)
- ✅ RPC connection verified

We can proceed to:
1. Complete range proofs in circuits
2. Generate real proofs with Arecibo
3. Extract Solidity verifiers
4. Deploy contracts to Arc testnet
5. Run end-to-end demonstration

---

## Resources

- **Arc Docs:** https://docs.arc.network
- **Faucet:** https://faucet.circle.com
- **Explorer:** https://testnet.arcscan.app
- **Circle Discord:** For extra testnet tokens
- **Official Site:** https://www.arc.network

---

**Last Updated:** 2025-11-12
**Network Status:** Public Testnet (Live)
**Ready for Development:** ✅
