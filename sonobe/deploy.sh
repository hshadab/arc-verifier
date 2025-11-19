#!/bin/bash

# Arc Testnet Deployment Script
# Usage: PRIVATE_KEY=0xYourPrivateKey ./deploy.sh

set -e

# Arc Testnet Configuration
RPC_URL="${ARC_TESTNET_RPC_URL:-https://rpc.testnet.arc.network}"
CHAIN_ID=5042002

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}  Arc Testnet - CompositeFundVerifier Deployment${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

# Check if private key is set
if [ -z "$PRIVATE_KEY" ]; then
    echo -e "${RED}Error: PRIVATE_KEY environment variable not set${NC}"
    echo ""
    echo "Usage:"
    echo "  PRIVATE_KEY=0xYourPrivateKey ./deploy.sh"
    echo ""
    echo "Or set it in your environment:"
    echo "  export PRIVATE_KEY=0xYourPrivateKey"
    echo "  ./deploy.sh"
    exit 1
fi

echo -e "${GREEN}✓${NC} RPC URL: $RPC_URL"
echo -e "${GREEN}✓${NC} Chain ID: $CHAIN_ID"
echo ""

# Get deployer address
echo "📍 Getting deployer address..."
DEPLOYER=$(/home/hshadab/.foundry/bin/cast wallet address "$PRIVATE_KEY")
echo -e "${GREEN}✓${NC} Deployer: $DEPLOYER"

# Check balance
echo ""
echo "💰 Checking deployer balance..."
BALANCE=$(/home/hshadab/.foundry/bin/cast balance "$DEPLOYER" --rpc-url "$RPC_URL")
echo -e "${GREEN}✓${NC} Balance: $BALANCE wei"

# Deploy the verifier contract
echo ""
echo "🚀 Deploying CompositeFundVerifier.sol..."
echo ""

DEPLOY_OUTPUT=$(/home/hshadab/.foundry/bin/forge create CompositeFundVerifier.sol:NovaDecider \
    --rpc-url "$RPC_URL" \
    --private-key "$PRIVATE_KEY" \
    --legacy \
    2>&1)

echo "$DEPLOY_OUTPUT"

# Extract deployed address
DEPLOYED_ADDRESS=$(echo "$DEPLOY_OUTPUT" | grep "Deployed to:" | awk '{print $3}')

if [ -z "$DEPLOYED_ADDRESS" ]; then
    echo -e "${RED}✗ Deployment failed${NC}"
    exit 1
fi

echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}✅ Deployment Successful!${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo "📋 Deployment Details:"
echo "   Contract: NovaDecider"
echo "   Address: $DEPLOYED_ADDRESS"
echo "   Network: Arc Testnet"
echo "   Chain ID: $CHAIN_ID"
echo "   Deployer: $DEPLOYER"
echo ""
echo "🔗 Block Explorer:"
echo "   https://arc-sepolia.explorer.alchemy.com/address/$DEPLOYED_ADDRESS"
echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo "📝 Next Steps:"
echo "   1. Verify proof with: /home/hshadab/.foundry/bin/cast call $DEPLOYED_ADDRESS 'verifyNovaProof(uint256[28])' \$(cat composite-proof.calldata) --rpc-url $RPC_URL"
echo "   2. View calldata: cat composite-proof.calldata"
echo "   3. View inputs: cat composite-proof.inputs"
echo ""

# Save deployment info
cat > deployment-info.txt <<EOF
CompositeFundVerifier Deployment
================================
Deployed at: $(date)
Contract: NovaDecider
Address: $DEPLOYED_ADDRESS
Network: Arc Testnet
Chain ID: $CHAIN_ID
RPC URL: $RPC_URL
Deployer: $DEPLOYER
Block Explorer: https://arc-sepolia.explorer.alchemy.com/address/$DEPLOYED_ADDRESS

Proof Files:
- Verifier Contract: CompositeFundVerifier.sol
- Calldata: composite-proof.calldata
- Inputs: composite-proof.inputs

Verification Command:
/home/hshadab/.foundry/bin/cast call $DEPLOYED_ADDRESS \\
    'verifyNovaProof(uint256[28])' \\
    \$(cat composite-proof.calldata) \\
    --rpc-url $RPC_URL
EOF

echo -e "${GREEN}✓${NC} Deployment info saved to: deployment-info.txt"
echo ""
