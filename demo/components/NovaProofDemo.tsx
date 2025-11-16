'use client';

import { useState } from 'react';

interface NovaEvent {
  type: 'progress' | 'complete' | 'error';
  message: string;
  step?: number;
  totalSteps?: number;
  timestamp?: number;
  proofSize?: number;
  verified?: boolean;
  periodsProven?: number;
}

export default function NovaProofDemo() {
  const [loading, setLoading] = useState(false);
  const [messages, setMessages] = useState<string[]>([]);
  const [progress, setProgress] = useState({ current: 0, total: 0 });
  const [result, setResult] = useState<any>(null);
  const [startTime, setStartTime] = useState<number | null>(null);
  const [duration, setDuration] = useState<number | null>(null);
  const [proofCalldata, setProofCalldata] = useState<string | null>(null);

  const handleGenerateNova = async () => {
    setLoading(true);
    setMessages([]);
    setResult(null);
    setDuration(null);
    setProgress({ current: 0, total: 5 });
    const start = Date.now();
    setStartTime(start);

    try {
      const response = await fetch('/api/generate-nova-proof', {
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
            try {
              const data: NovaEvent = JSON.parse(line.slice(6));

              if (data.type === 'progress') {
                setMessages(prev => [...prev, `${new Date().toLocaleTimeString()}: ${data.message}`]);
                if (data.step && data.totalSteps) {
                  setProgress({ current: data.step, total: data.totalSteps });
                }
              } else if (data.type === 'complete') {
                const elapsed = Date.now() - start;
                setDuration(elapsed);
                setMessages(prev => [...prev, `✅ ${data.message}`]);
                setResult({
                  proofSize: data.proofSize,
                  verified: data.verified,
                  periodsProven: data.periodsProven,
                  timestamp: data.timestamp,
                });
                setLoading(false);
                setProgress({ current: data.totalSteps || 5, total: data.totalSteps || 5 });
              } else if (data.type === 'error') {
                setMessages(prev => [...prev, `❌ ${data.message}`]);
                setLoading(false);
              }
            } catch (e) {
              console.error('Failed to parse SSE data:', e);
            }
          }
        }
      }
    } catch (error) {
      setMessages(prev => [...prev, `❌ Connection error: ${error instanceof Error ? error.message : 'Unknown error'}`]);
      setLoading(false);
    }
  };

  return (
    <div className="py-20 px-4 sm:px-6 lg:px-8 bg-gradient-to-b from-arc-darker to-arc-dark">
      <div className="max-w-7xl mx-auto">
        {/* Header */}
        <div className="text-center mb-12">
          <div className="inline-block px-4 py-2 rounded-full bg-purple-500/20 border border-purple-500/30 text-purple-400 text-sm font-semibold mb-4">
            🌟 Nova Recursive Proofs
          </div>
          <h2 className="text-4xl font-bold mb-4 bg-gradient-to-r from-purple-400 to-pink-400 bg-clip-text text-transparent">
            Live Nova Proof Generation
          </h2>
          <p className="text-xl text-gray-400 max-w-3xl mx-auto">
            Generate real recursive Nova proofs with Groth16 compression. Watch the entire process happen in real-time.
          </p>
        </div>

        {/* Main Demo Card */}
        <div className="glass-effect rounded-2xl p-8 mb-8">
          <div className="grid lg:grid-cols-2 gap-8">
            {/* Left: Control Panel */}
            <div>
              <h3 className="text-2xl font-bold mb-4">Proof Generator</h3>
              <p className="text-gray-400 mb-6">
                Click below to see a real Nova proof with recursive folding and Groth16 compression.
                On Render.com, this uses a pre-generated cached proof (instant). Local development generates live proofs (60-75s).
              </p>

              <button
                onClick={handleGenerateNova}
                disabled={loading}
                className={`w-full px-8 py-6 rounded-xl font-bold text-xl transition-all transform ${
                  loading
                    ? 'bg-gray-600 text-gray-400 cursor-wait'
                    : 'bg-gradient-to-r from-purple-600 to-pink-600 text-white hover:shadow-2xl hover:shadow-purple-500/50 hover:scale-105 active:scale-95'
                }`}
              >
                {loading ? (
                  <span className="flex items-center justify-center gap-3">
                    <div className="w-6 h-6 border-4 border-white border-t-transparent rounded-full animate-spin"></div>
                    Generating Nova Proof...
                  </span>
                ) : (
                  <span className="flex items-center justify-center gap-3">
                    🚀 Generate Nova Proof
                  </span>
                )}
              </button>

              {/* Expected Timeline */}
              <div className="mt-6 p-4 rounded-lg bg-arc-darker/50 border border-gray-700">
                <div className="text-sm font-semibold text-gray-300 mb-3">Deployment Info:</div>
                <div className="space-y-2 text-sm text-gray-400">
                  <div className="flex justify-between">
                    <span>🌐 Render.com (Demo)</span>
                    <span className="font-mono text-green-400">Instant</span>
                  </div>
                  <div className="flex justify-between">
                    <span>💻 Local Development</span>
                    <span className="font-mono">~60-75s</span>
                  </div>
                  <div className="flex justify-between">
                    <span>🏢 Production (16GB+)</span>
                    <span className="font-mono">~60-75s</span>
                  </div>
                  <div className="pt-2 mt-2 border-t border-gray-700 text-xs text-gray-500">
                    Render uses cached proof due to 8GB RAM limit. Real proof generation requires 16GB+ for Decider compression.
                  </div>
                </div>
              </div>

              {/* Result Display */}
              {result && duration && (
                <div className="mt-6 p-6 rounded-xl bg-gradient-to-r from-green-500/20 to-blue-500/20 border-2 border-green-500/50">
                  <div className="text-center mb-4">
                    <div className="text-4xl mb-2">✅</div>
                    <div className="text-2xl font-bold text-green-400 mb-1">Proof Generated!</div>
                    <div className="text-lg text-gray-300">
                      Completed in <span className="font-mono font-bold">{(duration / 1000).toFixed(1)}s</span>
                    </div>
                  </div>
                  <div className="grid grid-cols-2 gap-4 mt-4 pt-4 border-t border-gray-600">
                    <div className="text-center">
                      <div className="text-xs text-gray-400">Proof Size</div>
                      <div className="text-lg font-bold text-purple-400">{result.proofSize || 900}B</div>
                    </div>
                    <div className="text-center">
                      <div className="text-xs text-gray-400">Periods Proven</div>
                      <div className="text-lg font-bold text-blue-400">{result.periodsProven || 3}</div>
                    </div>
                  </div>
                  {result.verified && (
                    <div className="mt-4 text-center text-sm text-green-400">
                      ✓ Proof verification successful
                    </div>
                  )}
                </div>
              )}
            </div>

            {/* Right: Live Progress */}
            <div>
              <h3 className="text-2xl font-bold mb-4">Live Progress</h3>

              {/* Progress Bar */}
              {progress.total > 0 && (
                <div className="mb-4">
                  <div className="flex justify-between text-sm text-gray-400 mb-2">
                    <span>Step {progress.current} of {progress.total}</span>
                    <span>{Math.round((progress.current / progress.total) * 100)}%</span>
                  </div>
                  <div className="w-full bg-gray-700 rounded-full h-3 overflow-hidden">
                    <div
                      className="bg-gradient-to-r from-purple-500 to-pink-500 h-full transition-all duration-500"
                      style={{ width: `${(progress.current / progress.total) * 100}%` }}
                    ></div>
                  </div>
                </div>
              )}

              {/* Message Log */}
              <div className="bg-arc-darker rounded-lg p-4 font-mono text-sm h-[400px] overflow-y-auto border border-gray-700">
                {messages.length === 0 ? (
                  <div className="text-gray-500 text-center py-8">
                    Click "Generate Nova Proof" to start.<br/>
                    Live updates will appear here.
                  </div>
                ) : (
                  <div className="space-y-1">
                    {messages.map((msg, i) => (
                      <div
                        key={i}
                        className={`${msg.includes('✅') ? 'text-green-400' : msg.includes('❌') ? 'text-red-400' : 'text-gray-300'}`}
                      >
                        {msg}
                      </div>
                    ))}
                    {loading && (
                      <div className="text-purple-400 animate-pulse">
                        ● Processing...
                      </div>
                    )}
                  </div>
                )}
              </div>

              {startTime && loading && (
                <div className="mt-4 text-center text-sm text-gray-400">
                  Elapsed: <span className="font-mono font-semibold text-purple-400">
                    {Math.floor((Date.now() - startTime) / 1000)}s
                  </span>
                </div>
              )}
            </div>
          </div>
        </div>

        {/* Technical Details */}
        <div className="grid md:grid-cols-3 gap-6">
          <div className="glass-effect p-6 rounded-lg text-center">
            <div className="text-3xl mb-2">🔄</div>
            <h4 className="font-bold mb-2">Recursive Folding</h4>
            <p className="text-sm text-gray-400">
              Nova incrementally verifiable computation with 3 fold steps
            </p>
          </div>
          <div className="glass-effect p-6 rounded-lg text-center">
            <div className="text-3xl mb-2">🗜️</div>
            <h4 className="font-bold mb-2">Groth16 Compression</h4>
            <p className="text-sm text-gray-400">
              Final proof compressed to constant 900 bytes for on-chain verification
            </p>
          </div>
          <div className="glass-effect p-6 rounded-lg text-center">
            <div className="text-3xl mb-2">⚡</div>
            <h4 className="font-bold mb-2">Production Ready</h4>
            <p className="text-sm text-gray-400">
              Real cryptographic proofs, not simulated or mocked
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}
