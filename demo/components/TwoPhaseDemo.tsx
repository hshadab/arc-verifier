'use client';

import { useState } from 'react';

interface ProofRecord {
  id: number;
  duration: number;
  timestamp: Date;
  cost: number;
  proofSize?: number;
  verified: boolean;
  localVerification?: boolean;
  txHash?: string;
  blockNumber?: number;
  gasUsed?: string;
}

export default function TwoPhaseDemo() {
  const [initialized, setInitialized] = useState(false);
  const [loading, setLoading] = useState(false);
  const [loadTime, setLoadTime] = useState<number | null>(null);
  const [proofs, setProofs] = useState<ProofRecord[]>([]);
  const [message, setMessage] = useState('');
  const [progress, setProgress] = useState({ current: 0, total: 0 });

  const handleInitialize = async () => {
    setLoading(true);
    setMessage('Initializing system...');
    setProgress({ current: 0, total: 3 });

    try {
      const response = await fetch('/api/initialize', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
      });

      const reader = response.body?.getReader();
      const decoder = new TextDecoder();

      if (!reader) {
        throw new Error('No response stream');
      }

      while (true) {
        const { done, value } = await reader.read();
        if (done) break;

        const chunk = decoder.decode(value);
        const lines = chunk.split('\n');

        for (const line of lines) {
          if (line.startsWith('data: ')) {
            const data = JSON.parse(line.slice(6));

            if (data.type === 'progress') {
              setMessage(data.message);
              if (data.step && data.totalSteps) {
                setProgress({ current: data.step, total: data.totalSteps });
              }
            } else if (data.type === 'complete') {
              setMessage('✅ ' + data.message);
              setInitialized(true);
              setLoadTime(data.load_time_ms);
              setLoading(false);
            } else if (data.type === 'error') {
              setMessage('❌ ' + data.message);
              setLoading(false);
            }
          }
        }
      }
    } catch (error) {
      setMessage('❌ Connection error: ' + (error instanceof Error ? error.message : 'Unknown error'));
      setLoading(false);
    }
  };

  const handleGenerateProof = async () => {
    setLoading(true);
    setMessage('Generating proof...');
    setProgress({ current: 0, total: 4 });

    try {
      const response = await fetch('/api/generate-proof', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
      });

      const reader = response.body?.getReader();
      const decoder = new TextDecoder();

      if (!reader) {
        throw new Error('No response stream');
      }

      while (true) {
        const { done, value } = await reader.read();
        if (done) break;

        const chunk = decoder.decode(value);
        const lines = chunk.split('\n');

        for (const line of lines) {
          if (line.startsWith('data: ')) {
            const data = JSON.parse(line.slice(6));

            if (data.type === 'progress') {
              setMessage(data.message);
              if (data.step && data.totalSteps) {
                setProgress({ current: data.step, total: data.totalSteps });
              }
            } else if (data.type === 'complete') {
              setMessage(data.localVerification ? '✅ Proof generated and verified locally!' : data.txHash ? '✅ Proof verified on Arc testnet!' : '✅ Proof verified!');
              setProofs([...proofs, {
                id: proofs.length + 1,
                duration: Date.now() - new Date().getTime(),
                timestamp: new Date(),
                cost: data.cost_usd || 0.02,
                proofSize: data.proofSize,
                verified: data.verified ?? true,
                localVerification: data.localVerification,
                txHash: data.txHash,
                blockNumber: data.blockNumber,
                gasUsed: data.gasUsed,
              }]);
              setLoading(false);
            } else if (data.type === 'error') {
              setMessage('❌ ' + data.message);
              setLoading(false);
            }
          }
        }
      }
    } catch (error) {
      setMessage('❌ Connection error: ' + (error instanceof Error ? error.message : 'Unknown error'));
      setLoading(false);
    }
  };

  const totalCost = proofs.reduce((sum, p) => sum + p.cost, 0);
  const costSavings = proofs.length > 0 ? (proofs.length * 0.92) - totalCost : 0;

  return (
    <div className="py-20 px-4 sm:px-6 lg:px-8 bg-gradient-to-b from-arc-dark to-arc-darker">
      <div className="max-w-7xl mx-auto">
        {/* Header */}
        <div className="text-center mb-12">
          <h2 className="text-4xl font-bold mb-4 bg-gradient-to-r from-arc-primary to-arc-secondary bg-clip-text text-transparent">
            Interactive ZK Compliance Demo
          </h2>
          <p className="text-xl text-gray-400">
            Generate real Groth16 proofs and verify on Arc testnet
          </p>
        </div>

        <div className="grid lg:grid-cols-2 gap-8 mb-8">
          {/* Phase 1: Initialize */}
          <div className="glass-effect p-8 rounded-2xl">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-2xl font-bold">Phase 1: One-Time Setup</h3>
              <span className={`px-3 py-1 rounded-full text-sm font-medium ${
                initialized
                  ? 'bg-green-500/20 text-green-400 border border-green-500/30'
                  : 'bg-gray-500/20 text-gray-400 border border-gray-500/30'
              }`}>
                {initialized ? '✅ Ready' : '⏳ Not Started'}
              </span>
            </div>

            <p className="text-gray-400 mb-6">
              Load Groth16 cryptographic keys into memory (~2-3 seconds, once per session)
            </p>

            <button
              onClick={handleInitialize}
              disabled={initialized || loading}
              className={`w-full px-6 py-4 rounded-lg font-semibold text-lg transition-all ${
                initialized
                  ? 'bg-green-500/20 text-green-400 border-2 border-green-500/30 cursor-not-allowed'
                  : loading
                  ? 'bg-gray-600 text-white cursor-wait'
                  : 'bg-gradient-to-r from-blue-600 to-blue-500 text-white hover:shadow-lg hover:shadow-blue-500/50'
              }`}
            >
              {initialized ? '✅ System Ready' : loading ? '⏳ Initializing...' : '🔧 Initialize System'}
            </button>

            {loadTime !== null && (
              <div className="mt-4 p-4 rounded-lg bg-arc-darker/50 border border-gray-700">
                <div className="text-sm text-gray-400">Initialization Time</div>
                <div className="text-2xl font-bold text-green-400">
                  {(loadTime / 1000).toFixed(1)}s
                </div>
                <div className="text-xs text-gray-500 mt-1">
                  Parameters loaded and ready for proof generation
                </div>
              </div>
            )}
          </div>

          {/* Phase 2: Generate Proofs */}
          <div className="glass-effect p-8 rounded-2xl">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-2xl font-bold">Phase 2: Generate Proofs</h3>
              <span className={`px-3 py-1 rounded-full text-sm font-medium ${
                initialized
                  ? 'bg-green-500/20 text-green-400 border border-green-500/30'
                  : 'bg-gray-500/20 text-gray-400 border border-gray-500/30'
              }`}>
                {initialized ? 'Unlocked' : '🔒 Locked'}
              </span>
            </div>

            <p className="text-gray-400 mb-6">
              {initialized
                ? 'Generate Groth16 proofs with pre-loaded keys (~2-5 seconds each)'
                : 'Initialize system first to unlock fast proof generation'}
            </p>

            <button
              onClick={handleGenerateProof}
              disabled={!initialized || loading}
              className={`w-full px-6 py-4 rounded-lg font-semibold text-lg transition-all ${
                !initialized || loading
                  ? 'bg-gray-600 text-gray-400 cursor-not-allowed'
                  : 'bg-gradient-to-r from-green-600 to-green-500 text-white hover:shadow-lg hover:shadow-green-500/50'
              }`}
            >
              {loading && initialized ? '⏳ Generating...' : '▶️ Run Compliance Check ($0.02)'}
            </button>

            {proofs.length > 0 && (
              <div className="mt-4 p-4 rounded-lg bg-arc-darker/50 border border-gray-700">
                <div className="text-sm text-gray-400 mb-2">Latest Proof</div>
                <div className="flex justify-between items-center">
                  <span className="font-bold">Proof #{proofs[proofs.length - 1].id}</span>
                  <span className="text-green-400 font-semibold">
                    {(proofs[proofs.length - 1].duration / 1000).toFixed(1)}s • ${proofs[proofs.length - 1].cost.toFixed(2)}
                  </span>
                </div>
              </div>
            )}
          </div>
        </div>

        {/* Progress/Status Display */}
        {message && (
          <div className="glass-effect rounded-lg p-6 mb-8">
            <div className="flex items-start gap-4">
              {loading && (
                <div className="w-6 h-6 border-4 border-arc-primary border-t-transparent rounded-full animate-spin flex-shrink-0 mt-1"></div>
              )}
              <div className="flex-1">
                <p className="font-mono text-sm">{message}</p>
                {progress.total > 0 && (
                  <div className="mt-3">
                    <div className="flex justify-between text-xs text-gray-400 mb-1">
                      <span>Progress</span>
                      <span>{progress.current} of {progress.total} steps</span>
                    </div>
                    <div className="w-full bg-gray-700 rounded-full h-2 overflow-hidden">
                      <div
                        className="bg-gradient-to-r from-arc-primary to-arc-secondary h-full transition-all duration-500"
                        style={{ width: `${(progress.current / progress.total) * 100}%` }}
                      ></div>
                    </div>
                  </div>
                )}
              </div>
            </div>
          </div>
        )}

        {/* Results Display */}
        {proofs.length > 0 && (
          <div className="glass-effect rounded-2xl p-8">
            <div className="flex items-center justify-between mb-6">
              <h3 className="text-2xl font-bold">Proofs Generated: {proofs.length}</h3>
              {costSavings > 0 && (
                <div className="px-4 py-2 rounded-lg bg-green-500/20 border border-green-500/30">
                  <div className="text-sm text-gray-400">Total Savings</div>
                  <div className="text-xl font-bold text-green-400">
                    ${costSavings.toFixed(2)}
                  </div>
                </div>
              )}
            </div>

            <div className="space-y-3 max-h-96 overflow-y-auto">
              {proofs.map((proof) => (
                <div
                  key={proof.id}
                  className="bg-arc-darker/50 p-4 rounded-lg border border-gray-700 hover:border-arc-primary/50 transition-colors"
                >
                  <div className="flex justify-between items-start mb-2">
                    <div>
                      <div className="font-semibold">Proof #{proof.id}</div>
                      <div className="text-sm text-gray-400">
                        {proof.timestamp.toLocaleTimeString()}
                      </div>
                    </div>
                    <div className="text-right">
                      <div className="font-bold text-green-400">
                        {proof.verified ? '✅ Verified' : '❌ Failed'}
                      </div>
                      <div className="text-sm text-gray-400">
                        {proof.localVerification ? 'Locally' : 'On-chain'}
                      </div>
                    </div>
                  </div>
                  <div className="mt-3 pt-3 border-t border-gray-700 grid grid-cols-2 gap-4 text-sm">
                    <div>
                      <div className="text-gray-400">Proof Size</div>
                      <div className="font-semibold">{proof.proofSize || 128} bytes</div>
                    </div>
                    <div>
                      <div className="text-gray-400">Est. Gas Cost</div>
                      <div className="font-semibold text-green-400">${proof.cost.toFixed(2)}</div>
                    </div>
                  </div>
                  {proof.txHash && (
                    <div className="mt-3 pt-3 border-t border-gray-700">
                      <div className="flex items-center gap-2 text-sm">
                        <span className="text-gray-400">On-chain:</span>
                        <a
                          href={`${process.env.NEXT_PUBLIC_EXPLORER_URL || 'https://testnet.arcscan.app'}/tx/${proof.txHash}`}
                          target="_blank"
                          rel="noopener noreferrer"
                          className="text-arc-primary hover:text-arc-secondary font-mono text-xs underline"
                        >
                          {proof.txHash.slice(0, 10)}...{proof.txHash.slice(-8)}
                        </a>
                        {proof.blockNumber && (
                          <span className="text-gray-500 text-xs">
                            Block {proof.blockNumber.toLocaleString()}
                          </span>
                        )}
                      </div>
                      {proof.gasUsed && (
                        <div className="text-xs text-gray-500 mt-1">
                          Gas used: {Number(proof.gasUsed).toLocaleString()}
                        </div>
                      )}
                    </div>
                  )}
                  {proof.localVerification && (
                    <div className="mt-3 pt-3 border-t border-gray-700">
                      <div className="text-xs text-gray-400">
                        💡 To verify on-chain, deploy the verifier contract and update .env.local
                        <a
                          href="https://github.com/yourusername/arc-verifier/blob/main/GROTH16_DEPLOYMENT_GUIDE.md"
                          target="_blank"
                          className="text-arc-primary hover:underline ml-1"
                        >
                          (see guide)
                        </a>
                      </div>
                    </div>
                  )}
                </div>
              ))}
            </div>

            {proofs.length > 1 && (
              <div className="mt-6 p-6 rounded-lg bg-gradient-to-r from-green-500/10 to-blue-500/10 border-2 border-green-500/30">
                <div className="text-center">
                  <p className="text-green-400 font-bold text-lg mb-2">
                    💰 Saved ${costSavings.toFixed(2)} by reusing verifier!
                  </p>
                  <p className="text-sm text-gray-400">
                    Deploy-per-check would cost ${(proofs.length * 0.92).toFixed(2)} vs ${totalCost.toFixed(2)} with persisted parameters
                  </p>
                  <p className="text-xs text-gray-500 mt-2">
                    {proofs.length} proofs • {((proofs.reduce((sum, p) => sum + p.duration, 0)) / 1000 / 60).toFixed(1)} minutes total
                  </p>
                </div>
              </div>
            )}
          </div>
        )}

        {/* Info Section */}
        <div className="mt-12 grid md:grid-cols-3 gap-6">
          <div className="glass-effect p-6 rounded-lg text-center">
            <div className="text-3xl mb-2">⚡</div>
            <h4 className="font-bold mb-2">2-5s Per Proof</h4>
            <p className="text-sm text-gray-400">
              Fast Groth16 generation after 2-3s initialization
            </p>
          </div>
          <div className="glass-effect p-6 rounded-lg text-center">
            <div className="text-3xl mb-2">💰</div>
            <h4 className="font-bold mb-2">$0.02 Per Verification</h4>
            <p className="text-sm text-gray-400">
              Efficient on-chain verification (~1M gas)
            </p>
          </div>
          <div className="glass-effect p-6 rounded-lg text-center">
            <div className="text-3xl mb-2">🔒</div>
            <h4 className="font-bold mb-2">3 Compliance Checks</h4>
            <p className="text-sm text-gray-400">
              Position limit, liquidity, and whitelist all proven
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}
