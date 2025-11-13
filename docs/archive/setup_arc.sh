#!/bin/bash
# Arc Testnet Quick Setup Script

echo "🚀 Arc Testnet Setup"
echo "===================="
echo ""

# Check if cast is installed
if ! command -v cast &> /dev/null; then
    echo "❌ Foundry not found. Install it first:"
    echo "curl -L https://foundry.paradigm.xyz | bash"
    echo "foundryup"
    exit 1
fi

echo "✅ Foundry detected"
echo ""

# Generate new wallet if needed
if [ ! -f .env ]; then
    echo "📝 Generating new wallet..."
    cast wallet new > wallet_info.txt
    PRIVATE_KEY=$(grep "Private key:" wallet_info.txt | awk '{print $3}')
    ADDRESS=$(grep "Address:" wallet_info.txt | awk '{print $2}')
    
    echo "PRIVATE_KEY=$PRIVATE_KEY" > .env
    echo "ARC_TESTNET_RPC_URL=https://rpc.testnet.arc.network" >> .env
    echo "ARC_CHAIN_ID=5042002" >> .env
    
    echo "✅ Wallet generated!"
    echo "   Address: $ADDRESS"
    echo "   Private key saved to .env"
    echo ""
    echo "⚠️  KEEP wallet_info.txt SECURE and backed up!"
    echo ""
else
    echo "ℹ️  .env file already exists"
    source .env
    ADDRESS=$(cast wallet address --private-key $PRIVATE_KEY)
    echo "   Using address: $ADDRESS"
    echo ""
fi

# Verify connection
echo "🔗 Testing Arc testnet connection..."
CHAIN_ID=$(cast chain-id --rpc-url https://rpc.testnet.arc.network 2>/dev/null)
if [ "$CHAIN_ID" == "5042002" ]; then
    echo "✅ Connected to Arc testnet (Chain ID: $CHAIN_ID)"
    echo ""
else
    echo "❌ Failed to connect to Arc testnet"
    exit 1
fi

# Check balance
echo "💰 Checking USDC balance..."
BALANCE=$(cast balance $ADDRESS --rpc-url https://rpc.testnet.arc.network)
BALANCE_USDC=$(echo "scale=2; $BALANCE / 1000000" | bc)
echo "   Balance: $BALANCE_USDC USDC"
echo ""

if [ "$BALANCE" == "0" ]; then
    echo "📢 Next steps:"
    echo "   1. Go to https://faucet.circle.com"
    echo "   2. Select 'USDC' and 'Arc Testnet'"
    echo "   3. Enter address: $ADDRESS"
    echo "   4. Get 10 USDC"
    echo "   5. Run this script again to verify"
    echo ""
else
    echo "✅ Wallet funded! Ready to deploy."
    echo ""
    echo "📝 Your deployment command:"
    echo "   source .env"
    echo "   forge create --rpc-url \$ARC_TESTNET_RPC_URL --private-key \$PRIVATE_KEY YourContract"
fi

echo ""
echo "🔍 View your account:"
echo "   https://testnet.arcscan.app/address/$ADDRESS"
echo ""
