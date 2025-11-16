export default function Hero() {
  return (
    <div className="pt-52 pb-20 px-4 sm:px-6 lg:px-8">
      <div className="max-w-7xl mx-auto text-center">
        <h1 className="text-5xl md:text-7xl font-bold mb-6 bg-clip-text text-transparent bg-arc-gradient">
          Zero-Knowledge Fund Compliance
        </h1>

        <p className="text-xl md:text-2xl text-gray-300 mb-12 max-w-3xl mx-auto">
          Prove regulatory compliance <span className="text-arc-primary font-semibold">without revealing</span> your fund&apos;s portfolio details
        </p>

        <div className="flex flex-col sm:flex-row items-center justify-center gap-4 mb-12">
          <div className="glass-effect px-8 py-4 rounded-lg hover:glow-effect transition-all">
            <div className="text-4xl font-bold text-arc-primary">$0.02</div>
            <div className="text-sm text-gray-400 mt-1">Per Proof Verification</div>
          </div>

          <div className="glass-effect px-8 py-4 rounded-lg hover:glow-effect transition-all">
            <div className="text-4xl font-bold text-arc-primary">~128B</div>
            <div className="text-sm text-gray-400 mt-1">Compact Proof Size</div>
          </div>

          <div className="glass-effect px-8 py-4 rounded-lg hover:glow-effect transition-all">
            <div className="text-4xl font-bold text-arc-primary">100%</div>
            <div className="text-sm text-gray-400 mt-1">Privacy Preserved</div>
          </div>
        </div>

        <div className="flex flex-wrap justify-center gap-3 text-sm text-gray-400">
          <span className="flex items-center gap-2">
            <svg className="w-5 h-5 text-green-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
            </svg>
            Groth16 zkSNARK
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
            3 Compliance Checks
          </span>
        </div>
      </div>
    </div>
  )
}
