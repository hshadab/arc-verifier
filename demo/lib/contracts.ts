// Arc Network Testnet Configuration
export const ARC_TESTNET = {
  id: 5042002,
  name: 'Arc Testnet',
  network: 'arc-testnet',
  nativeCurrency: {
    decimals: 18,
    name: 'ETH',
    symbol: 'ETH',
  },
  rpcUrls: {
    default: { http: ['https://rpc.arc.gelato.digital'] },
    public: { http: ['https://rpc.arc.gelato.digital'] },
  },
  blockExplorers: {
    default: { name: 'Arc Explorer', url: 'https://arc-testnet.blockscout.com' },
  },
}

// Deployed contract addresses
export const CONTRACTS = {
  NovaDeciderVerifier: '0x076E915833620074669Eccd70aD8836EfA143A7B',
  TokenizedFundManager: '0xaAdc1327a66D992F3d8E6fBa57F6BE7e810d80DE',
} as const

// Nova Decider Verifier ABI (just the verify function)
export const NOVA_VERIFIER_ABI = [
  {
    inputs: [
      { name: 'a', type: 'uint256[2]' },
      { name: 'b', type: 'uint256[2][2]' },
      { name: 'c', type: 'uint256[2]' },
      { name: 'input', type: 'uint256[38]' },
    ],
    name: 'verifyProof',
    outputs: [{ name: '', type: 'bool' }],
    stateMutability: 'view',
    type: 'function',
  },
] as const
