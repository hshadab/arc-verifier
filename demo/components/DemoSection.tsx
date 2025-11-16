'use client'

import { useState } from 'react'

interface Props {
  proofStatus: 'idle' | 'generating' | 'verifying' | 'verified'
  setProofStatus: (status: 'idle' | 'generating' | 'verifying' | 'verified') => void
}

export default function DemoSection({ proofStatus, setProofStatus }: Props) {
  const [currentStep, setCurrentStep] = useState(1)

  const fundData = {
    totalValue: '$100,000,000',
    usdcBalance: '$10,000,000',
    liquidity: '10%',
    minRequired: '10%',
    assets: [
      { name: 'BENJI', value: '$35M', percentage: '35%' },
      { name: 'BUIDL', value: '$30M', percentage: '30%' },
      { name: 'RE Token', value: '$25M', percentage: '25%' },
      { name: 'USDC', value: '$10M', percentage: '10%' },
    ]
  }

  const simulateProofGeneration = () => {
    setProofStatus('generating')
    setCurrentStep(2)

    // Simulate ~22s proof generation (shortened to 5s for demo)
    setTimeout(() => {
      setProofStatus('verifying')
      setCurrentStep(3)

      // Simulate on-chain verification (~20ms in reality)
      setTimeout(() => {
        setProofStatus('verified')
        setCurrentStep(4)
      }, 2000)
    }, 5000)
  }

  return (
    <div className="py-20 px-4 sm:px-6 lg:px-8">
      <div className="max-w-7xl mx-auto">
        <div className="text-center mb-12">
          <h2 className="text-4xl font-bold mb-4">Interactive Demo</h2>
          <p className="text-xl text-gray-400">
            See zero-knowledge compliance in action
          </p>
        </div>

        <div className="grid md:grid-cols-2 gap-8 mb-12">
          {/* Private Data */}
          <div className="glass-effect p-8 rounded-2xl">
            <div className="flex items-center justify-between mb-6">
              <h3 className="text-2xl font-bold">Private Fund Data</h3>
              <span className="px-3 py-1 rounded-full bg-red-500/20 text-red-400 text-sm border border-red-500/30">
                🔒 Hidden On-Chain
              </span>
            </div>

            <div className="space-y-4">
              <div className="p-4 rounded-lg bg-arc-darker/50 border border-gray-700">
                <div className="text-sm text-gray-400 mb-1">Total Portfolio Value</div>
                <div className="text-2xl font-bold blur-sm hover:blur-none transition-all cursor-help" title="Hidden from blockchain">
                  {fundData.totalValue}
                </div>
              </div>

              <div className="p-4 rounded-lg bg-arc-darker/50 border border-gray-700">
                <div className="text-sm text-gray-400 mb-1">USDC Balance</div>
                <div className="text-2xl font-bold blur-sm hover:blur-none transition-all cursor-help" title="Hidden from blockchain">
                  {fundData.usdcBalance}
                </div>
              </div>

              <div className="space-y-2">
                <div className="text-sm text-gray-400 mb-2">Asset Breakdown</div>
                {fundData.assets.map((asset, idx) => (
                  <div key={idx} className="flex justify-between items-center p-3 rounded-lg bg-arc-darker/30">
                    <span className="font-medium">{asset.name}</span>
                    <div className="text-right">
                      <div className="font-bold blur-sm hover:blur-none transition-all cursor-help" title="Hidden from blockchain">
                        {asset.value}
                      </div>
                      <div className="text-sm text-gray-400">{asset.percentage}</div>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          </div>

          {/* Public Proof */}
          <div className="glass-effect p-8 rounded-2xl">
            <div className="flex items-center justify-between mb-6">
              <h3 className="text-2xl font-bold">Public Proof Result</h3>
              <span className="px-3 py-1 rounded-full bg-green-500/20 text-green-400 text-sm border border-green-500/30">
                ✓ Visible On-Chain
              </span>
            </div>

            <div className="space-y-6">
              <div className="p-6 rounded-lg bg-green-500/10 border-2 border-green-500/30">
                <div className="text-sm text-gray-400 mb-2">Compliance Status</div>
                <div className="text-3xl font-bold text-green-400 flex items-center gap-3">
                  <svg className="w-8 h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                  </svg>
                  COMPLIANT
                </div>
              </div>

              <div className="p-4 rounded-lg bg-arc-darker/50 border border-gray-700">
                <div className="text-sm text-gray-400 mb-1">Minimum Liquidity Required</div>
                <div className="text-xl font-bold">{fundData.minRequired}</div>
              </div>

              <div className="p-4 rounded-lg bg-arc-darker/50 border border-gray-700">
                <div className="text-sm text-gray-400 mb-1">Proof Result</div>
                <div className="text-xl font-bold text-green-400">
                  Liquidity ≥ 10% ✓
                </div>
                <div className="text-sm text-gray-400 mt-2">
                  (Exact amounts remain private)
                </div>
              </div>

              <div className="p-4 rounded-lg bg-blue-500/10 border border-blue-500/30">
                <div className="flex items-center justify-between mb-2">
                  <span className="text-sm text-gray-400">On-Chain Cost</span>
                  <span className="text-2xl font-bold text-arc-primary">$0.02</span>
                </div>
                <div className="text-xs text-gray-400">
                  795,738 gas • ~20ms verification
                </div>
              </div>
            </div>
          </div>
        </div>

        {/* Interactive Steps */}
        <div className="glass-effect p-8 rounded-2xl">
          <div className="flex items-center justify-between mb-8">
            {[1, 2, 3, 4].map((step) => (
              <div key={step} className="flex-1 relative">
                <div className={`flex items-center ${step < 4 ? 'pr-4' : ''}`}>
                  <div className={`w-12 h-12 rounded-full flex items-center justify-center font-bold text-lg border-2 transition-all ${
                    currentStep >= step
                      ? 'bg-arc-primary border-arc-primary text-arc-dark'
                      : 'bg-arc-dark border-gray-600 text-gray-500'
                  }`}>
                    {currentStep > step ? '✓' : step}
                  </div>
                  {step < 4 && (
                    <div className={`flex-1 h-1 ml-4 transition-all ${
                      currentStep > step ? 'bg-arc-primary' : 'bg-gray-700'
                    }`} />
                  )}
                </div>
                <div className={`mt-2 text-sm font-medium ${currentStep >= step ? 'text-white' : 'text-gray-500'}`}>
                  {step === 1 && 'View Data'}
                  {step === 2 && 'Generate Proof'}
                  {step === 3 && 'Verify On-Chain'}
                  {step === 4 && 'Confirmed'}
                </div>
              </div>
            ))}
          </div>

          <div className="text-center">
            {proofStatus === 'idle' && (
              <button
                onClick={simulateProofGeneration}
                className="px-8 py-4 rounded-lg bg-arc-gradient font-bold text-lg hover:shadow-lg hover:shadow-arc-primary/50 transition-all"
              >
                🚀 Generate Zero-Knowledge Proof
              </button>
            )}

            {proofStatus === 'generating' && (
              <div className="space-y-4">
                <div className="flex items-center justify-center gap-3">
                  <div className="w-6 h-6 border-4 border-arc-primary border-t-transparent rounded-full animate-spin"></div>
                  <span className="text-lg font-medium">Generating proof... (~22s)</span>
                </div>
                <div className="text-sm text-gray-400">Creating Nova recursive SNARK</div>
              </div>
            )}

            {proofStatus === 'verifying' && (
              <div className="space-y-4">
                <div className="flex items-center justify-center gap-3">
                  <div className="w-6 h-6 border-4 border-green-500 border-t-transparent rounded-full animate-spin"></div>
                  <span className="text-lg font-medium text-green-400">Verifying on Arc testnet...</span>
                </div>
                <div className="text-sm text-gray-400">NovaDecider contract @ {(process.env.NEXT_PUBLIC_NOVA_VERIFIER || '0x076E915833620074669Eccd70aD8836EfA143A7B').slice(0,6)}...{(process.env.NEXT_PUBLIC_NOVA_VERIFIER || '0x076E915833620074669Eccd70aD8836EfA143A7B').slice(-3)}</div>
              </div>
            )}

            {proofStatus === 'verified' && (
              <div className="space-y-4">
                <div className="text-3xl">✅</div>
                <div className="text-2xl font-bold text-green-400">Proof Verified!</div>
                <div className="text-gray-400">Compliance confirmed on Arc testnet</div>
                <div className="flex items-center justify-center gap-4 mt-6">
                  <a
                    href={`${process.env.NEXT_PUBLIC_EXPLORER_URL || 'https://testnet.arcscan.app'}/address/${process.env.NEXT_PUBLIC_NOVA_VERIFIER || '0x076E915833620074669Eccd70aD8836EfA143A7B'}`}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="px-6 py-2 rounded-lg glass-effect hover:glow-effect transition-all"
                  >
                    View on Explorer
                  </a>
                  <button
                    onClick={() => {
                      setProofStatus('idle')
                      setCurrentStep(1)
                    }}
                    className="px-6 py-2 rounded-lg bg-arc-gradient hover:shadow-lg transition-all"
                  >
                    Try Again
                  </button>
                </div>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  )
}
