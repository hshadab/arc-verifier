export default function Header() {
  return (
    <header className="fixed top-0 w-full z-50 glass-effect border-b border-gray-800">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6 relative">
        {/* Status badge - Top Right */}
        <div className="absolute top-6 right-4 sm:right-6 lg:right-8">
          <div className="flex items-center space-x-2 px-4 py-2 rounded-lg bg-green-500/10 border border-green-500/20">
            <div className="w-2 h-2 rounded-full bg-green-500 animate-pulse"></div>
            <span className="text-sm text-green-400">Arc Testnet</span>
          </div>
        </div>

        {/* Logos */}
        <div className="flex items-center justify-center gap-8 mb-4">
          <img
            src="https://cdn.prod.website-files.com/685311a976e7c248b5dfde95/688f6e47eca8d8e359537b5f_logo-ondark.svg"
            alt="Arc Logo"
            className="h-12 object-contain"
          />
          <img
            src="https://cdn.prod.website-files.com/65d52b07d5bc41614daa723f/665df12739c532f45b665fe7_logo-novanet.svg"
            alt="NovaNet Logo"
            className="h-10 object-contain"
          />
        </div>

        {/* Title with gradient */}
        <h1 className="text-4xl md:text-5xl font-bold text-center mb-2 bg-gradient-to-r from-arc-primary to-arc-secondary bg-clip-text text-transparent">
          On-chain Proof Verification
        </h1>

        {/* Subtitle */}
        <p className="text-lg md:text-xl text-center text-gray-400 max-w-2xl mx-auto">
          Use Arc to Verify Zero Knowledge Proofs of Fund Compliance
        </p>
      </div>
    </header>
  )
}
