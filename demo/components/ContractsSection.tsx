export default function ContractsSection() {
  const contracts = [
    {
      name: 'NovaDecider Verifier',
      address: '0x076E915833620074669Eccd70aD8836EfA143A7B',
      description: 'Auto-generated Solidity verifier for Nova proofs',
      features: ['KZG10 commitments', 'Groth16 compression', '795K gas per verification'],
      status: 'deployed',
    },
    {
      name: 'TokenizedFundManager',
      address: '0xaAdc1327a66D992F3d8E6fBa57F6BE7e810d80DE',
      description: 'Fund compliance manager with integrated verifier',
      features: ['Daily rebalance limits', 'Audit trail', 'Multi-circuit support'],
      status: 'deployed',
    },
  ]

  return (
    <div className="py-20 px-4 sm:px-6 lg:px-8 bg-arc-darker/50">
      <div className="max-w-7xl mx-auto">
        <div className="text-center mb-12">
          <h2 className="text-4xl font-bold mb-4">Deployed Contracts</h2>
          <p className="text-xl text-gray-400">
            Live on Arc Testnet (Chain ID: 5042002)
          </p>
        </div>

        <div className="grid md:grid-cols-2 gap-8">
          {contracts.map((contract, idx) => (
            <div key={idx} className="glass-effect p-8 rounded-2xl hover:glow-effect transition-all">
              <div className="flex items-start justify-between mb-4">
                <h3 className="text-2xl font-bold">{contract.name}</h3>
                <span className="px-3 py-1 rounded-full bg-green-500/20 text-green-400 text-sm border border-green-500/30">
                  ✓ {contract.status}
                </span>
              </div>

              <p className="text-gray-400 mb-6">{contract.description}</p>

              <div className="mb-6">
                <div className="text-sm text-gray-400 mb-2">Contract Address</div>
                <div className="flex items-center gap-2">
                  <code className="flex-1 px-3 py-2 rounded-lg bg-arc-darker text-sm font-mono text-arc-primary overflow-x-auto">
                    {contract.address}
                  </code>
                  <button
                    onClick={() => navigator.clipboard.writeText(contract.address)}
                    className="p-2 rounded-lg glass-effect hover:bg-white/10 transition-all"
                    title="Copy address"
                  >
                    📋
                  </button>
                </div>
              </div>

              <div className="space-y-2 mb-6">
                <div className="text-sm text-gray-400 mb-2">Features</div>
                {contract.features.map((feature, fidx) => (
                  <div key={fidx} className="flex items-center gap-2 text-sm">
                    <svg className="w-4 h-4 text-green-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                    </svg>
                    <span>{feature}</span>
                  </div>
                ))}
              </div>

              <a
                href={`https://testnet.arcscan.app/address/${contract.address}`}
                target="_blank"
                rel="noopener noreferrer"
                className="block w-full text-center px-4 py-2 rounded-lg glass-effect hover:glow-effect transition-all"
              >
                View on Arc Explorer →
              </a>
            </div>
          ))}
        </div>

        <div className="mt-12 glass-effect p-8 rounded-2xl">
          <h3 className="text-2xl font-bold mb-6">Technology Stack</h3>
          <div className="grid md:grid-cols-3 gap-6">
            <div>
              <div className="text-sm text-gray-400 mb-2">Zero-Knowledge Proofs</div>
              <ul className="space-y-1 text-sm">
                <li>• Nova (Incremental IVC)</li>
                <li>• Sonobe v0.1.0</li>
                <li>• BN254 curves</li>
                <li>• KZG10 + Groth16</li>
              </ul>
            </div>
            <div>
              <div className="text-sm text-gray-400 mb-2">Circuits</div>
              <ul className="space-y-1 text-sm">
                <li>• Bellpepper (R1CS)</li>
                <li>• 34,914 constraints</li>
                <li>• Rust implementation</li>
                <li>• 22/22 tests passing</li>
              </ul>
            </div>
            <div>
              <div className="text-sm text-gray-400 mb-2">Smart Contracts</div>
              <ul className="space-y-1 text-sm">
                <li>• Solidity 0.8.20</li>
                <li>• Foundry framework</li>
                <li>• 10/10 tests passing</li>
                <li>• Production ready</li>
              </ul>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}
