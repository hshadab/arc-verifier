export default function Footer() {
  return (
    <footer className="py-12 px-4 sm:px-6 lg:px-8 border-t border-gray-800">
      <div className="max-w-7xl mx-auto">
        <div className="grid md:grid-cols-4 gap-8 mb-8">
          <div>
            <h3 className="font-bold mb-4">Arc ZK Compliance</h3>
            <p className="text-sm text-gray-400">
              Zero-knowledge fund compliance on Arc Network
            </p>
          </div>

          <div>
            <h4 className="font-semibold mb-4">Resources</h4>
            <ul className="space-y-2 text-sm text-gray-400">
              <li>
                <a href="https://github.com/hshadab/arc-verifier" className="hover:text-arc-primary transition-colors">
                  GitHub Repository
                </a>
              </li>
              <li>
                <a href="https://github.com/hshadab/arc-verifier/blob/main/INTEGRATION_COMPLETE.md" className="hover:text-arc-primary transition-colors">
                  Documentation
                </a>
              </li>
              <li>
                <a href="https://github.com/hshadab/arc-verifier/blob/main/END_TO_END_TEST.md" className="hover:text-arc-primary transition-colors">
                  Test Results
                </a>
              </li>
            </ul>
          </div>

          <div>
            <h4 className="font-semibold mb-4">Technology</h4>
            <ul className="space-y-2 text-sm text-gray-400">
              <li>
                <a href="https://github.com/privacy-scaling-explorations/sonobe" className="hover:text-arc-primary transition-colors">
                  Sonobe (Nova)
                </a>
              </li>
              <li>
                <a href="https://arc.network" className="hover:text-arc-primary transition-colors">
                  Arc Network
                </a>
              </li>
              <li>
                <a href="https://eprint.iacr.org/2021/370" className="hover:text-arc-primary transition-colors">
                  Nova Paper
                </a>
              </li>
            </ul>
          </div>

          <div>
            <h4 className="font-semibold mb-4">Network</h4>
            <ul className="space-y-2 text-sm text-gray-400">
              <li>Chain ID: 5042002</li>
              <li>
                <a href="https://testnet.arcscan.app" className="hover:text-arc-primary transition-colors">
                  Block Explorer
                </a>
              </li>
              <li>
                <a href="https://rpc.testnet.arc.network" className="hover:text-arc-primary transition-colors">
                  RPC Endpoint
                </a>
              </li>
            </ul>
          </div>
        </div>

        <div className="pt-8 border-t border-gray-800 flex flex-col md:flex-row justify-between items-center gap-4">
          <div className="text-sm text-gray-400">
            Built for Arc Network • Powered by Nova ZK Proofs
          </div>
          <div className="flex items-center gap-4 text-sm text-gray-400">
            <span>MIT License</span>
            <span>•</span>
            <span>Production Ready</span>
            <span>•</span>
            <span className="text-green-400">32/32 Tests Passing ✓</span>
          </div>
        </div>
      </div>
    </footer>
  )
}
