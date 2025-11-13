export default function Header() {
  return (
    <header className="fixed top-0 w-full z-50 glass-effect">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="flex justify-between items-center py-4">
          <div className="flex items-center space-x-2">
            <div className="w-10 h-10 rounded-lg bg-arc-gradient flex items-center justify-center">
              <span className="text-2xl">🔒</span>
            </div>
            <div>
              <h1 className="text-xl font-bold">Arc ZK Compliance</h1>
              <p className="text-xs text-gray-400">Zero-Knowledge Fund Verification</p>
            </div>
          </div>

          <div className="flex items-center space-x-4">
            <div className="hidden md:flex items-center space-x-2 px-4 py-2 rounded-lg bg-green-500/10 border border-green-500/20">
              <div className="w-2 h-2 rounded-full bg-green-500 animate-pulse"></div>
              <span className="text-sm text-green-400">Arc Testnet</span>
            </div>

            <a
              href="https://github.com/hshadab/arc-verifier"
              target="_blank"
              rel="noopener noreferrer"
              className="px-4 py-2 rounded-lg glass-effect hover:glow-effect transition-all"
            >
              View on GitHub
            </a>
          </div>
        </div>
      </div>
    </header>
  )
}
