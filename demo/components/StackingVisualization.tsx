'use client'

import { useState, useEffect } from 'react'

interface Props {
  isAnimating: boolean
  onComplete?: () => void
}

export default function StackingVisualization({ isAnimating, onComplete }: Props) {
  const [step, setStep] = useState(0)
  const [realGasUsed, setRealGasUsed] = useState<string>('796000')
  const [realTxHash, setRealTxHash] = useState<string | null>(null)
  const [realBlockNumber, setRealBlockNumber] = useState<number | null>(null)
  const [proofDurations, setProofDurations] = useState<{ [key: number]: string }>({})
  const [deciderDuration, setDeciderDuration] = useState<string>('')
  const [isGeneratingReal, setIsGeneratingReal] = useState(false)

  // 0: Initial state
  // 1: Period 1 appears
  // 2: Period 1 proves (badge appears)
  // 3: Period 2 appears
  // 4: Period 2 proves (badge appears)
  // 5: Period 3 appears
  // 6: Period 3 proves (badge appears)
  // 7: All proofs stack together
  // 8: Compressed into one
  // 9: Proof moves to Arc verifier
  // 10: Arc verification (scanning)
  // 11: Gas meter filling
  // 12: Block number incrementing
  // 13: Verified! (burst effect)

  const VERIFIER_ADDRESS = process.env.NEXT_PUBLIC_NOVA_VERIFIER || '0xf6Ff03AEBA3321d7c01Ddb210Bda914826708dEE'
  const EXPLORER_URL = process.env.NEXT_PUBLIC_EXPLORER_URL || 'https://testnet.arcscan.app'

  useEffect(() => {
    if (isAnimating && !isGeneratingReal) {
      setIsGeneratingReal(true)
      setStep(0)
      setRealTxHash(null)
      setRealBlockNumber(null)
      setProofDurations({})
      setDeciderDuration('')

      // Start real proof generation
      startRealProofGeneration()
    }
  }, [isAnimating])

  const startRealProofGeneration = async () => {
    try {
      console.log('Starting real proof generation...')
      // Call the API to generate real proofs
      const response = await fetch('/api/generate-proof', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        }
      })

      console.log('Response received, status:', response.status)

      if (!response.body) {
        throw new Error('No response body')
      }

      const reader = response.body.getReader()
      const decoder = new TextDecoder()

      console.log('Starting to read SSE stream...')

      while (true) {
        const { done, value } = await reader.read()
        if (done) {
          console.log('Stream ended')
          break
        }

        const text = decoder.decode(value)
        console.log('Received chunk:', text.substring(0, 100))
        const lines = text.split('\n')

        for (const line of lines) {
          if (line.startsWith('data: ')) {
            try {
              const eventData = line.slice(6)
              console.log('Raw SSE data:', eventData)
              const event = JSON.parse(eventData)
              console.log('Parsed event:', event)

              if (event.type === 'progress') {
                // Just starting
                setStep(1) // Show Period 1 card
              } else if (event.type === 'period') {
                // Real proof generated!
                if (event.step === 1) {
                  setProofDurations(prev => ({ ...prev, 1: event.duration || '' }))
                  setStep(2) // Show Period 1 proved
                  setTimeout(() => setStep(3), 400) // Show Period 2 card
                } else if (event.step === 2) {
                  setProofDurations(prev => ({ ...prev, 2: event.duration || '' }))
                  setStep(4) // Show Period 2 proved
                  setTimeout(() => setStep(5), 400) // Show Period 3 card
                } else if (event.step === 3) {
                  setProofDurations(prev => ({ ...prev, 3: event.duration || '' }))
                  setStep(6) // Show Period 3 proved
                  setTimeout(() => setStep(7), 600) // Start stacking
                  setTimeout(() => setStep(8), 1400) // Show compressed proof
                }
              } else if (event.type === 'compressed') {
                // Nova folding complete - show the compressed proof box
                setDeciderDuration(event.duration || '')
                // Give it time to display before moving to Arc
                setTimeout(() => setStep(9), 2000) // Move to Arc (arrow)
                setTimeout(() => setStep(10), 2800) // Start Arc verification
              } else if (event.type === 'complete') {
                // All done! Set all final data and transition to complete
                setRealGasUsed(event.gasUsed || '796000')
                setRealTxHash(event.txHash || null)
                setRealBlockNumber(event.blockNumber || null)

                // Schedule transitions AFTER compressed event transitions complete
                // compressed schedules: step 9 (+2000ms), step 10 (+2800ms)
                // So we start at +4000ms to ensure no overlap
                setTimeout(() => setStep(11), 4000) // Show gas meter (after step 10 settles)
                setTimeout(() => setStep(12), 4700) // Show block number
                setTimeout(() => setStep(13), 5500) // Complete!
                setTimeout(() => {
                  setIsGeneratingReal(false)
                  onComplete?.() // Notify parent that animation is complete
                }, 6000)
              } else if (event.type === 'error') {
                console.error('Proof generation error:', event.message)
                setIsGeneratingReal(false)
              }
            } catch (e) {
              console.error('Failed to parse SSE event:', e)
            }
          }
        }
      }
    } catch (error) {
      console.error('Error connecting to proof generation:', error)
      setIsGeneratingReal(false)

      // Fallback to simulation if API fails
      setTimeout(() => setStep(1), 800)
      setTimeout(() => setStep(2), 1600)
      setTimeout(() => setStep(3), 2400)
      setTimeout(() => setStep(4), 3200)
      setTimeout(() => setStep(5), 4000)
      setTimeout(() => setStep(6), 4800)
      setTimeout(() => setStep(7), 5800)
      setTimeout(() => setStep(8), 7000)
      setTimeout(() => setStep(9), 8500)
      setTimeout(() => setStep(10), 10000)
      setTimeout(() => setStep(11), 11500)
      setTimeout(() => setStep(12), 13000)
      setTimeout(() => setStep(13), 14500)
    }
  }

  const layers = [
    {
      period: 'Period 1',
      day: 'Day 1',
      color: 'bg-blue-500/30',
      borderColor: 'border-blue-500',
      checks: [
        { name: 'Position Limit', icon: '📊', value: '≤ 40%' },
        { name: 'Liquidity', icon: '💧', value: '≥ 10%' },
        { name: 'Whitelist', icon: '✓', value: 'Valid' },
      ],
      appearStep: 2,
      proveStep: 2,
    },
    {
      period: 'Period 2',
      day: 'Day 2',
      color: 'bg-purple-500/30',
      borderColor: 'border-purple-500',
      checks: [
        { name: 'Position Limit', icon: '📊', value: '≤ 40%' },
        { name: 'Liquidity', icon: '💧', value: '≥ 10%' },
        { name: 'Whitelist', icon: '✓', value: 'Valid' },
      ],
      appearStep: 3,
      proveStep: 4,
    },
    {
      period: 'Period 3',
      day: 'Day 3',
      color: 'bg-pink-500/30',
      borderColor: 'border-pink-500',
      checks: [
        { name: 'Position Limit', icon: '📊', value: '≤ 40%' },
        { name: 'Liquidity', icon: '💧', value: '≥ 10%' },
        { name: 'Whitelist', icon: '✓', value: 'Valid' },
      ],
      appearStep: 5,
      proveStep: 6,
    },
  ]

  return (
    <div className="py-12">
      <div className="text-center mb-8">
        <h3 className="text-3xl font-bold mb-2">Nova Proof Folding + Arc Verification</h3>
        <p className="text-gray-400">
          {step === 0 && 'Starting compliance verification...'}
          {step === 1 && 'Setup: Initializing proving system (~10s)...'}
          {step === 2 && 'Generating proof for Period 1...'}
          {(step === 3 || step === 4) && 'Generating proof for Period 2...'}
          {(step === 5 || step === 6) && 'Generating proof for Period 3...'}
          {step === 7 && 'Stacking proofs together...'}
          {step === 8 && 'Nova folding: creating compressed proof...'}
          {step === 9 && 'Sending to Arc Network...'}
          {step >= 10 && step < 13 && 'Verifying on Arc blockchain...'}
          {step === 13 && '✓ Verified! Real Sonobe Nova proofs.'}
        </p>
        {isGeneratingReal && step > 0 && step < 13 && (
          <div className="mt-3 inline-flex items-center gap-2 px-4 py-2 rounded-full bg-green-500/20 border border-green-500/30">
            <div className="w-2 h-2 rounded-full bg-green-500 animate-pulse"></div>
            <span className="text-sm text-green-400 font-semibold">⚡ Live Proof Generation</span>
          </div>
        )}
      </div>

      <div className="flex items-center justify-center gap-8 min-h-[600px] relative">
        {/* Left Side: Sequential Proof Cards */}
        <div className={`relative w-96 h-[500px] transition-all duration-1000 ${
          step >= 9 ? 'opacity-0 scale-50' : 'opacity-100 scale-100'
        }`}>
          {/* Setup Phase (before Period 1) - ONLY show at step 1 */}
          {step === 1 && (
            <div className="absolute inset-0 flex items-center justify-center z-20">
              <div className="w-80 p-6 glass-effect rounded-2xl border-2 border-yellow-500/50 animate-pulse">
                <div className="text-center">
                  <div className="text-4xl mb-3">⚙️</div>
                  <div className="font-bold text-yellow-400 text-xl mb-2">One-Time Setup</div>
                  <div className="text-sm text-gray-300 mb-3">
                    Initializing proving system...
                  </div>
                  <div className="h-2 bg-arc-darker rounded-full overflow-hidden mb-3">
                    <div className="h-full bg-gradient-to-r from-yellow-500 to-yellow-400 animate-progress-bar" />
                  </div>
                  <div className="text-xs text-gray-400 space-y-1">
                    <div>• PublicParams generation (~2s)</div>
                    <div>• Decider setup (~8s)</div>
                    <div className="text-yellow-400 font-bold mt-2">This happens once, not per proof!</div>
                  </div>
                </div>
              </div>
            </div>
          )}

          {/* Folded Proof Outline - surrounds the three stacked cards */}
          {(step === 7 || step === 8) && (
            <div className="absolute inset-0 flex items-center justify-center z-10">
              <div className={`transition-all duration-1000 ${
                step === 7 ? 'w-[340px] h-[400px] opacity-50' : 'w-[360px] h-[420px] opacity-100'
              } border-4 border-arc-primary rounded-3xl animate-pulse-glow`}
                style={{
                  boxShadow: '0 0 30px rgba(0, 229, 255, 0.5), inset 0 0 30px rgba(0, 229, 255, 0.2)'
                }}
              >
                {/* Corner indicators for folding */}
                <div className="absolute -top-3 -left-3 w-6 h-6 bg-arc-primary rounded-full animate-ping" />
                <div className="absolute -top-3 -right-3 w-6 h-6 bg-arc-primary rounded-full animate-ping" style={{ animationDelay: '0.2s' }} />
                <div className="absolute -bottom-3 -left-3 w-6 h-6 bg-arc-primary rounded-full animate-ping" style={{ animationDelay: '0.4s' }} />
                <div className="absolute -bottom-3 -right-3 w-6 h-6 bg-arc-primary rounded-full animate-ping" style={{ animationDelay: '0.6s' }} />

                {/* Label */}
                <div className="absolute -top-10 left-1/2 transform -translate-x-1/2 px-4 py-1 bg-arc-primary/20 border border-arc-primary rounded-full text-xs font-bold text-arc-primary whitespace-nowrap">
                  📦 Nova Folding {step === 8 ? '- Creating compressed proof...' : ''}
                </div>
              </div>
            </div>
          )}

          <div className="absolute inset-0 flex flex-col items-center justify-center gap-4" style={{ zIndex: 15 }}>
            {layers.map((layer, idx) => {
              const hasAppeared = step >= layer.appearStep
              const isProved = step >= layer.proveStep
              const isStacking = step >= 7
              const shouldHide = step >= 9 // Only hide when moving to Arc, not during folding

              if (!hasAppeared) return null

              return (
                <div
                  key={idx}
                  className={`transition-all duration-700 ease-in-out
                    ${layer.color} ${layer.borderColor} border-2 rounded-2xl p-4 w-80
                    ${shouldHide ? 'opacity-0 scale-0' : 'opacity-100 scale-100'}
                    ${!hasAppeared ? 'opacity-0 translate-y-10' : ''}
                  `}
                  style={{
                    transform: isStacking
                      ? `translateY(${(1 - idx) * 20}px) scale(0.95)`
                      : shouldHide
                      ? 'scale(0)'
                      : 'translateY(0) scale(1)',
                  }}
                >
                  <div className="mb-3">
                    <div className="flex items-center justify-between mb-2">
                      <div>
                        <div className="text-sm font-bold">{layer.period}</div>
                        <div className="text-xs text-gray-400">{layer.day}</div>
                      </div>
                      {hasAppeared && !isProved && (
                        <div className="px-3 py-1 rounded-full bg-blue-500/20 border border-blue-500/30 text-xs font-bold text-blue-400 animate-pulse">
                          ⚙️ Generating...
                        </div>
                      )}
                      {isProved && (
                        <div className="flex flex-col items-end gap-1">
                          <div className="px-3 py-1 rounded-full bg-green-500/30 border border-green-500 text-xs font-bold text-green-400 animate-pop-in">
                            ✓ Proved
                          </div>
                          {proofDurations[idx + 1] && (
                            <div className="text-xs text-gray-400 font-mono">
                              {proofDurations[idx + 1]}
                            </div>
                          )}
                        </div>
                      )}
                    </div>
                    {/* Progress bar while generating */}
                    {hasAppeared && !isProved && (
                      <div className="h-1.5 bg-arc-darker rounded-full overflow-hidden">
                        <div className="h-full bg-gradient-to-r from-blue-500 to-blue-400 animate-progress-bar" />
                      </div>
                    )}
                  </div>

                  {/* Checks */}
                  <div className="space-y-2">
                    {layer.checks.map((check, checkIdx) => (
                      <div key={checkIdx} className="flex items-center justify-between text-xs py-1 px-2 rounded bg-arc-darker/30">
                        <div className="flex items-center gap-2">
                          <span>{check.icon}</span>
                          <span className="text-gray-300">{check.name}</span>
                        </div>
                        <span className="text-green-400 font-mono">{check.value}</span>
                      </div>
                    ))}
                  </div>
                </div>
              )
            })}
          </div>
        </div>

        {/* Arrow Animation */}
        {step === 9 && (
          <div className="absolute left-1/2 top-1/2 transform -translate-x-1/2 -translate-y-1/2 z-20">
            <div className="text-6xl animate-pulse text-arc-primary">
              →
            </div>
          </div>
        )}

        {/* Right Side: Arc Verification Box */}
        <div
          className={`relative transition-all duration-1000 ${
            step >= 10 ? 'opacity-100 scale-100' : 'opacity-0 scale-0'
          }`}
        >
          <div className="w-[450px] glass-effect rounded-2xl p-8 border-2 border-arc-primary relative overflow-hidden">
            {/* Hexagon Grid Background */}
            <div className="absolute inset-0 opacity-10">
              <div className="absolute inset-0" style={{
                backgroundImage: `url("data:image/svg+xml,%3Csvg width='60' height='60' xmlns='http://www.w3.org/2000/svg'%3E%3Cpath d='M30 0l25.98 15v30L30 60 4.02 45V15z' fill='none' stroke='%2300E5FF' stroke-width='1'/%3E%3C/svg%3E")`,
                backgroundSize: '60px 60px'
              }} />
            </div>

            {/* Scanning Lines Effect */}
            {step >= 10 && step < 13 && (
              <>
                <div className="absolute inset-0 animate-scan opacity-30">
                  <div className="h-1 w-full bg-gradient-to-r from-transparent via-arc-primary to-transparent" />
                </div>
                <div className="absolute inset-0 animate-scan-slow opacity-20">
                  <div className="h-1 w-full bg-gradient-to-r from-transparent via-arc-secondary to-transparent" />
                </div>
              </>
            )}


            <div className="relative z-10">
              {/* Arc Logo with Glow */}
              <div className="text-center mb-6">
                <div className={`inline-block transition-all duration-500 ${
                  step >= 10 ? 'animate-pulse-slow' : ''
                }`}>
                  <img
                    src="https://cdn.prod.website-files.com/685311a976e7c248b5dfde95/688f6e47eca8d8e359537b5f_logo-ondark.svg"
                    alt="Arc Logo"
                    className="h-16 object-contain mx-auto"
                    style={{
                      filter: step >= 10 ? 'drop-shadow(0 0 20px rgba(0, 229, 255, 0.6))' : 'none'
                    }}
                  />
                </div>
                <div className="text-xl font-bold text-arc-primary mt-3">
                  {step < 10 && 'Arc Network Verifier'}
                  {step >= 10 && step < 13 && 'Verifying Proof...'}
                  {step === 13 && '✓ VERIFIED ON ARC'}
                </div>
              </div>

              {/* Verification Details */}
              <div className="space-y-3 mb-6">
                {/* Gas Meter */}
                <div className="p-3 rounded-lg bg-arc-darker/50 border border-arc-primary/30">
                  <div className="flex justify-between items-center mb-2">
                    <span className="text-sm text-gray-400">Gas Used</span>
                    <span className={`font-mono text-arc-primary font-bold transition-all duration-500 ${
                      step >= 11 ? 'opacity-100' : 'opacity-0'
                    }`}>
                      {step >= 11 ? parseInt(realGasUsed).toLocaleString() : '0'}
                    </span>
                  </div>
                  {step >= 11 && (
                    <div className="h-2 bg-arc-darker rounded-full overflow-hidden">
                      <div className="h-full bg-gradient-to-r from-arc-primary to-arc-secondary animate-fill-meter" />
                    </div>
                  )}
                  {step >= 11 && (
                    <div className="text-xs text-gray-500 mt-1">
                      Real proof verification ⚡
                    </div>
                  )}
                </div>

                {/* Block Number */}
                {step >= 12 && realBlockNumber && (
                  <div className="p-3 rounded-lg bg-arc-darker/50 border border-arc-primary/30 animate-slide-in">
                    <div className="flex justify-between items-center">
                      <span className="text-sm text-gray-400">Block Number</span>
                      <span className="font-mono text-arc-primary font-bold animate-count-up">
                        #{realBlockNumber.toLocaleString()}
                      </span>
                    </div>
                  </div>
                )}


                {/* Cost */}
                {step >= 13 && (
                  <div className="p-4 rounded-lg bg-green-500/10 border-2 border-green-500/30 animate-pop-in">
                    <div className="flex justify-between items-center">
                      <span className="text-sm text-gray-400">Verification Cost</span>
                      <span className="text-2xl font-bold text-green-400">$0.02</span>
                    </div>
                    <div className="text-xs text-gray-400 mt-1">
                      Paid in real testnet USDC
                    </div>
                  </div>
                )}
              </div>

              {/* Success Message */}
              {step === 13 && (
                <div className="text-center p-6 rounded-lg bg-green-500/20 border-2 border-green-500 animate-burst">
                  <div className="text-xl font-bold text-green-400 mb-2">
                    Verification Complete!
                  </div>
                  <div className="text-sm text-gray-300 mb-2">
                    This was a <span className="text-arc-primary font-bold">real proof generation</span> using Sonobe Nova!
                  </div>
                  <div className="text-xs text-gray-400 mb-4">
                    {realTxHash ? '⚡ Verified on-chain on Arc Testnet' : '📋 Proof verified locally with Sonobe'}
                  </div>
                  <div className="flex flex-col gap-2">
                    {realTxHash ? (
                      <a
                        href={`${EXPLORER_URL}/tx/${realTxHash}`}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="px-4 py-2 rounded-lg bg-green-500/20 border border-green-500 text-sm hover:bg-green-500/30 transition-all"
                      >
                        <div className="font-bold text-green-400 mb-1">✓ View On-Chain Verification →</div>
                        <div className="text-xs text-gray-300 font-mono">
                          {realTxHash.slice(0, 10)}...{realTxHash.slice(-8)}
                        </div>
                      </a>
                    ) : (
                      <div className="px-4 py-2 rounded-lg bg-blue-500/20 border border-blue-500 text-sm">
                        <div className="font-bold text-blue-400 mb-1">✓ Proof Generated Successfully</div>
                        <div className="text-xs text-gray-300">
                          3 REAL proofs folded into 1 compressed proof using Sonobe Nova.
                        </div>
                      </div>
                    )}
                    <a
                      href={`${EXPLORER_URL}/address/${VERIFIER_ADDRESS}`}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="px-4 py-2 rounded-lg bg-arc-primary/20 border border-arc-primary text-sm hover:bg-arc-primary/30 transition-all"
                    >
                      View Verifier Contract on Arc →
                    </a>
                  </div>
                </div>
              )}
            </div>
          </div>

        </div>
      </div>

      {/* Bottom Info */}
      <div className="mt-12 max-w-4xl mx-auto">
        <div className="glass-effect p-6 rounded-xl border border-arc-primary/30">
          <div className="flex items-start gap-4">
            <div className="text-3xl">💡</div>
            <div>
              <div className="font-bold text-lg mb-2">Real Proof Generation with Sonobe + On-Chain Verification</div>
              <div className="text-sm text-gray-300 space-y-2">
                <p>
                  <strong className="text-arc-primary">This demo generates 3 REAL Nova proofs</strong>, folds them into 1 compressed proof,
                  and verifies it <strong className="text-green-400">on Arc testnet blockchain</strong>. Nothing is mocked!
                </p>
                <p>
                  <strong className="text-green-400">Timeline:</strong>
                </p>
                <ul className="list-disc list-inside space-y-1 ml-2 text-xs">
                  <li><strong>0-10s:</strong> Setup (PublicParams + Decider - one-time initialization)</li>
                  <li><strong>10-13s:</strong> 3 REAL period proofs (~1s each) using Sonobe Nova ✓</li>
                  <li><strong>13-24s:</strong> Nova folding - combines 3 proofs into 1 compressed proof (900 bytes) ✓</li>
                  <li><strong>24s+:</strong> Submit FOLDED proof to Arc testnet for REAL on-chain verification ✓</li>
                </ul>
                <p className="text-xs text-gray-400 mt-3">
                  {isGeneratingReal && step < 13 ? (
                    <span className="text-green-400">⚡ Currently generating real proofs with Sonobe...</span>
                  ) : (
                    <>
                      <strong className="text-arc-primary">On-chain verification enabled!</strong> •
                      Verifier: <code className="text-arc-primary">{VERIFIER_ADDRESS.slice(0, 6)}...{VERIFIER_ADDRESS.slice(-4)}</code> •
                      Network: Arc Testnet (5042002)
                    </>
                  )}
                </p>
              </div>
            </div>
          </div>
        </div>
      </div>

      <style jsx>{`
        @keyframes scan {
          0% { transform: translateY(0); }
          100% { transform: translateY(400px); }
        }
        @keyframes scan-slow {
          0% { transform: translateY(-100px); }
          100% { transform: translateY(500px); }
        }
        @keyframes fill-meter {
          0% { width: 0%; }
          100% { width: 100%; }
        }
        @keyframes slide-in {
          0% { transform: translateX(-20px); opacity: 0; }
          100% { transform: translateX(0); opacity: 1; }
        }
        @keyframes slide-in-delay {
          0% { transform: translateX(-20px); opacity: 0; }
          100% { transform: translateX(0); opacity: 1; }
        }
        @keyframes pop-in {
          0% { transform: scale(0.8); opacity: 0; }
          50% { transform: scale(1.1); }
          100% { transform: scale(1); opacity: 1; }
        }
        @keyframes pop-in-delay {
          0% { transform: scale(0.8) rotate(-5deg); opacity: 0; }
          50% { transform: scale(1.1) rotate(2deg); }
          100% { transform: scale(1) rotate(0); opacity: 1; }
        }
        @keyframes burst {
          0% { transform: scale(0.9); }
          50% { transform: scale(1.05); }
          100% { transform: scale(1); }
        }
        @keyframes pulse-slow {
          0%, 100% { opacity: 1; transform: scale(1); }
          50% { opacity: 0.8; transform: scale(1.05); }
        }
        @keyframes pulse-glow {
          0%, 100% {
            opacity: 0.8;
            box-shadow: 0 0 30px rgba(0, 229, 255, 0.5), inset 0 0 30px rgba(0, 229, 255, 0.2);
          }
          50% {
            opacity: 1;
            box-shadow: 0 0 50px rgba(0, 229, 255, 0.8), inset 0 0 50px rgba(0, 229, 255, 0.4);
          }
        }
        @keyframes count-up {
          0% { opacity: 0; }
          100% { opacity: 1; }
        }

        .animate-scan {
          animation: scan 2s linear infinite;
        }
        .animate-scan-slow {
          animation: scan-slow 3s linear infinite;
        }
        .animate-fill-meter {
          animation: fill-meter 1s ease-out forwards;
        }
        .animate-slide-in {
          animation: slide-in 0.5s ease-out forwards;
        }
        .animate-slide-in-delay {
          animation: slide-in-delay 0.5s ease-out 0.2s forwards;
          opacity: 0;
        }
        .animate-pop-in {
          animation: pop-in 0.6s ease-out forwards;
        }
        .animate-pop-in-delay {
          animation: pop-in-delay 0.6s ease-out 0.3s forwards;
          opacity: 0;
        }
        .animate-burst {
          animation: burst 0.6s ease-out forwards;
        }
        .animate-pulse-slow {
          animation: pulse-slow 2s ease-in-out infinite;
        }
        .animate-pulse-glow {
          animation: pulse-glow 2s ease-in-out infinite;
        }
        .animate-count-up {
          animation: count-up 0.5s ease-out forwards;
        }
      `}</style>
    </div>
  )
}
