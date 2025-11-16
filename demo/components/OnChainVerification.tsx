'use client'

import { useState } from 'react'
import { createPublicClient, createWalletClient, custom, http } from 'viem'
import { ARC_TESTNET, CONTRACTS, NOVA_VERIFIER_ABI } from '@/lib/contracts'

interface ProofData {
  a: [string, string]
  b: [[string, string], [string, string]]
  c: [string, string]
  input: string[]
}

export default function OnChainVerification({ proofCalldata }: { proofCalldata?: string }) {
  const [connected, setConnected] = useState(false)
  const [address, setAddress] = useState<string | null>(null)
  const [verifying, setVerifying] = useState(false)
  const [result, setResult] = useState<{ success: boolean; txHash?: string; error?: string } | null>(null)

  const connectWallet = async () => {
    if (typeof window.ethereum === 'undefined') {
      alert('Please install MetaMask!')
      return
    }

    try {
      const accounts = await window.ethereum.request({ method: 'eth_requestAccounts' })
      setAddress(accounts[0])
      setConnected(true)

      // Switch to Arc testnet
      try {
        await window.ethereum.request({
          method: 'wallet_switchEthereumChain',
          params: [{ chainId: `0x${ARC_TESTNET.id.toString(16)}` }],
        })
      } catch (switchError: any) {
        // Chain not added, add it
        if (switchError.code === 4902) {
          await window.ethereum.request({
            method: 'wallet_addEthereumChain',
            params: [{
              chainId: `0x${ARC_TESTNET.id.toString(16)}`,
              chainName: ARC_TESTNET.name,
              nativeCurrency: ARC_TESTNET.nativeCurrency,
              rpcUrls: [ARC_TESTNET.rpcUrls.default.http[0]],
              blockExplorerUrls: [ARC_TESTNET.blockExplorers.default.url],
            }],
          })
        }
      }
    } catch (error) {
      console.error('Failed to connect wallet:', error)
      alert('Failed to connect wallet')
    }
  }

  const verifyOnChain = async () => {
    if (!proofCalldata) {
      alert('No proof data available. Generate a proof first!')
      return
    }

    setVerifying(true)
    setResult(null)

    try {
      // Parse proof calldata (hex string) into components
      // Format: 0x + (a[0] 32 bytes) + (a[1] 32 bytes) + (b[0][0] 32 bytes) + ... + (c[0] 32 bytes) + (c[1] 32 bytes) + (input[0] 32 bytes) + ...
      const data = proofCalldata.startsWith('0x') ? proofCalldata.slice(2) : proofCalldata

      // Each element is 32 bytes (64 hex chars)
      const parseElement = (offset: number) => '0x' + data.slice(offset, offset + 64)

      let offset = 0
      const proof: ProofData = {
        a: [parseElement(offset), parseElement(offset + 64)],
        b: [
          [parseElement(offset + 128), parseElement(offset + 192)],
          [parseElement(offset + 256), parseElement(offset + 320)]
        ],
        c: [parseElement(offset + 384), parseElement(offset + 448)],
        input: []
      }
      offset += 512 // Skip a, b, c (8 * 64)

      // Parse 38 public inputs
      for (let i = 0; i < 38; i++) {
        proof.input.push(parseElement(offset + i * 64))
      }

      // Create viem clients
      const publicClient = createPublicClient({
        chain: ARC_TESTNET,
        transport: http(),
      })

      const walletClient = createWalletClient({
        chain: ARC_TESTNET,
        transport: custom(window.ethereum!),
      })

      // Call verifyProof (view function - free)
      const isValid = await publicClient.readContract({
        address: CONTRACTS.NovaDeciderVerifier as `0x${string}`,
        abi: NOVA_VERIFIER_ABI,
        functionName: 'verifyProof',
        args: [proof.a, proof.b, proof.c, proof.input],
      })

      setResult({
        success: isValid as boolean,
      })

      setVerifying(false)
    } catch (error: any) {
      console.error('Verification failed:', error)
      setResult({
        success: false,
        error: error.message || 'Unknown error',
      })
      setVerifying(false)
    }
  }

  return (
    <div className="space-y-4">
      {!connected ? (
        <button
          onClick={connectWallet}
          className="w-full px-6 py-4 rounded-xl font-bold bg-gradient-to-r from-blue-600 to-purple-600 text-white hover:shadow-2xl hover:shadow-blue-500/50 transition-all"
        >
          🔌 Connect Wallet to Verify On-Chain
        </button>
      ) : (
        <div className="space-y-4">
          <div className="p-4 rounded-lg bg-green-500/20 border border-green-500/50">
            <div className="text-sm text-green-400">
              ✅ Connected: {address?.slice(0, 6)}...{address?.slice(-4)}
            </div>
            <div className="text-xs text-gray-400 mt-1">
              Network: {ARC_TESTNET.name}
            </div>
          </div>

          <button
            onClick={verifyOnChain}
            disabled={verifying || !proofCalldata}
            className={`w-full px-6 py-4 rounded-xl font-bold transition-all ${
              verifying || !proofCalldata
                ? 'bg-gray-600 text-gray-400 cursor-not-allowed'
                : 'bg-gradient-to-r from-green-600 to-blue-600 text-white hover:shadow-2xl hover:shadow-green-500/50'
            }`}
          >
            {verifying ? (
              <span className="flex items-center justify-center gap-2">
                <div className="w-5 h-5 border-3 border-white border-t-transparent rounded-full animate-spin"></div>
                Verifying On-Chain...
              </span>
            ) : (
              '🔗 Verify Proof on Arc Testnet'
            )}
          </button>

          {result && (
            <div className={`p-6 rounded-xl border-2 ${result.success ? 'bg-green-500/20 border-green-500' : 'bg-red-500/20 border-red-500'}`}>
              <div className="text-center">
                <div className="text-4xl mb-2">{result.success ? '✅' : '❌'}</div>
                <div className={`text-xl font-bold ${result.success ? 'text-green-400' : 'text-red-400'}`}>
                  {result.success ? 'Proof Verified On-Chain!' : 'Verification Failed'}
                </div>
                {result.error && (
                  <div className="mt-2 text-sm text-red-300">
                    Error: {result.error}
                  </div>
                )}
                {result.success && (
                  <div className="mt-4 text-sm text-gray-300">
                    <div>Contract: {CONTRACTS.NovaDeciderVerifier}</div>
                    <div className="mt-2">
                      <a
                        href={`${ARC_TESTNET.blockExplorers.default.url}/address/${CONTRACTS.NovaDeciderVerifier}`}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="text-blue-400 hover:underline"
                      >
                        View on Arc Explorer →
                      </a>
                    </div>
                  </div>
                )}
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  )
}

// TypeScript declaration for window.ethereum
declare global {
  interface Window {
    ethereum?: any
  }
}
