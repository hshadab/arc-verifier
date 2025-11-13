export default function Hero() {
  return (
    <div className="pt-32 pb-20 px-4 sm:px-6 lg:px-8">
      <div className="max-w-7xl mx-auto text-center">
        <div className="inline-flex items-center space-x-2 px-4 py-2 rounded-full glass-effect mb-6">
          <span className="w-2 h-2 rounded-full bg-arc-primary animate-pulse"></span>
          <span className="text-sm text-arc-primary">Live on Arc Testnet</span>
        </div>

        <h1 className="text-5xl md:text-7xl font-bold mb-6 bg-clip-text text-transparent bg-arc-gradient">
          Zero-Knowledge Fund Compliance
        </h1>

        <p className="text-xl md:text-2xl text-gray-300 mb-8 max-w-3xl mx-auto">
          Prove regulatory compliance <span className="text-arc-primary font-semibold">without revealing</span> your fund&apos;s portfolio details
        </p>

        <div className="flex flex-col sm:flex-row items-center justify-center gap-4 mb-12">
          <div className="glass-effect px-6 py-3 rounded-lg">
            <div className="text-3xl font-bold text-arc-primary">$0.02</div>
            <div className="text-sm text-gray-400">Per Verification</div>
          </div>

          <div className="glass-effect px-6 py-3 rounded-lg">
            <div className="text-3xl font-bold text-arc-primary">20ms</div>
            <div className="text-sm text-gray-400">On-Chain Verification</div>
          </div>

          <div className="glass-effect px-6 py-3 rounded-lg">
            <div className="text-3xl font-bold text-arc-primary">100%</div>
            <div className="text-sm text-gray-400">Privacy Preserved</div>
          </div>
        </div>

        <div className="flex flex-wrap justify-center gap-3 text-sm text-gray-400">
          <span className="flex items-center gap-2">
            <svg className="w-5 h-5 text-green-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
            </svg>
            Nova Recursive Proofs
          </span>
          <span className="flex items-center gap-2">
            <svg className="w-5 h-5 text-green-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
            </svg>
            Production Ready
          </span>
          <span className="flex items-center gap-2">
            <svg className="w-5 h-5 text-green-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
            </svg>
            32/32 Tests Passing
          </span>
        </div>
      </div>
    </div>
  )
}
