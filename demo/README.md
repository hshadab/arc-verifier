# Arc ZK Compliance Demo

Interactive demo showcasing zero-knowledge fund compliance on Arc Network.

## Features

- 🔒 **Privacy-Preserving** - Prove compliance without revealing balances
- ⚡ **Fast Verification** - 20ms on-chain verification
- 💰 **Cost-Effective** - Only $0.02 per verification
- 🎯 **Production Ready** - Real Nova proofs on Arc testnet

## Quick Start

```bash
cd demo
npm install
npm run dev
```

Open [http://localhost:3000](http://localhost:3000)

## Demo Flow

1. **View Fund State** - See private portfolio data
2. **Generate Proof** - Create zero-knowledge proof (~22s)
3. **Verify On-Chain** - Submit to Arc testnet verifier
4. **See Results** - Compliance proven without revealing amounts

## Deployed Contracts

- **NovaDecider Verifier**: `0x076E915833620074669Eccd70aD8836EfA143A7B`
- **TokenizedFundManager**: `0xaAdc1327a66D992F3d8E6fBa57F6BE7e810d80DE`
- **Network**: Arc Testnet (Chain ID: 5042002)

## Technology

- Next.js 14 + TypeScript
- Tailwind CSS
- ethers.js / wagmi
- RainbowKit wallet connection
