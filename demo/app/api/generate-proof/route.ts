import { NextRequest } from 'next/server'
import { getComplianceService } from '@/lib/compliance-service'
import { ethers } from 'ethers'
import fs from 'fs'
import path from 'path'

export const runtime = 'nodejs'
export const dynamic = 'force-dynamic'

interface ProofEvent {
  type: 'progress' | 'complete' | 'error'
  message: string
  proofSize?: number
  verified?: boolean
  localVerification?: boolean
  gasUsed?: string
  txHash?: string
  blockNumber?: number
  timestamp: number
}

export async function POST(request: NextRequest) {
  const encoder = new TextEncoder()

  const stream = new ReadableStream({
    async start(controller) {
      const sendEvent = (event: ProofEvent) => {
        console.log('Sending SSE event:', event)
        const data = `data: ${JSON.stringify(event)}\n\n`
        controller.enqueue(encoder.encode(data))
      }

      try {
        // Get the running service
        const service = getComplianceService()

        if (!service.isReady()) {
          sendEvent({
            type: 'error',
            message: 'Service not initialized. Call /api/initialize first.',
            timestamp: Date.now()
          })
          controller.close()
          return
        }

        sendEvent({
          type: 'progress',
          message: 'Generating Groth16 proof (checking 3 compliance rules)...',
          timestamp: Date.now()
        })

        // Generate proof using the service
        const result = await service.generateProof()

        if (result.status !== 'success' || !result.verified) {
          sendEvent({
            type: 'error',
            message: result.status === 'error'
              ? 'Proof generation failed'
              : 'Proof verification failed',
            timestamp: Date.now()
          })
          controller.close()
          return
        }

        sendEvent({
          type: 'progress',
          message: 'Proof generated and verified locally!',
          timestamp: Date.now()
        })

        // Read the calldata file
        const sonobePath = path.join(process.cwd(), '..', 'sonobe')
        const calldataPath = path.join(sonobePath, 'compliance-proof.calldata')

        if (!fs.existsSync(calldataPath)) {
          sendEvent({
            type: 'error',
            message: 'Proof calldata file not found',
            timestamp: Date.now()
          })
          controller.close()
          return
        }

        const calldataBytes = fs.readFileSync(calldataPath)
        const proofSize = calldataBytes.length

        // Submit to Arc testnet
        let txHash: string | undefined
        let blockNumber: number | undefined
        let gasUsed: string | undefined

        const rpcUrl = process.env.ARC_TESTNET_RPC_URL || 'https://rpc.testnet.arc.network'
        const privateKey = process.env.PRIVATE_KEY
        const verifierAddress = process.env.NEXT_PUBLIC_GROTH16_VERIFIER

        console.log('On-chain submission config:', {
          hasPrivateKey: !!privateKey,
          rpcUrl,
          verifierAddress,
          proofSize
        })

        if (!privateKey || !verifierAddress) {
          // Fall back to local verification if keys not configured
          sendEvent({
            type: 'complete',
            message: '✅ Proof verified locally (on-chain keys not configured)',
            proofSize,
            verified: true,
            localVerification: true,
            timestamp: Date.now()
          })
          controller.close()
          return
        }

        try {
          sendEvent({
            type: 'progress',
            message: 'Submitting Groth16 proof to Arc testnet...',
            timestamp: Date.now()
          })

          const provider = new ethers.JsonRpcProvider(rpcUrl)
          const wallet = new ethers.Wallet(privateKey, provider)

          // Build calldata: function selector + proof
          const calldataHex = '0x' + calldataBytes.toString('hex')

          console.log('Submitting proof:', {
            wallet: wallet.address,
            verifier: verifierAddress,
            calldataLength: calldataHex.length
          })

          // Submit the transaction
          const tx = await wallet.sendTransaction({
            to: verifierAddress,
            data: calldataHex,
            gasLimit: 1500000 // 1.5M gas limit for Groth16
          })

          console.log('Transaction sent:', tx.hash)

          sendEvent({
            type: 'progress',
            message: `Waiting for confirmation: ${tx.hash.slice(0, 10)}...`,
            timestamp: Date.now()
          })

          // Wait for confirmation
          const receipt = await tx.wait()

          if (receipt) {
            txHash = receipt.hash
            blockNumber = receipt.blockNumber
            gasUsed = receipt.gasUsed.toString()
            const status = receipt.status

            console.log('✅ Transaction confirmed!', {
              txHash,
              blockNumber,
              status,
              gasUsed
            })

            if (status === 0) {
              sendEvent({
                type: 'error',
                message: 'Transaction reverted - proof verification failed on-chain',
                timestamp: Date.now()
              })
            } else {
              sendEvent({
                type: 'complete',
                message: '✅ Proof verified on Arc testnet!',
                proofSize,
                verified: true,
                localVerification: false,
                gasUsed,
                txHash,
                blockNumber,
                timestamp: Date.now()
              })
            }
          }
        } catch (error) {
          console.error('On-chain submission error:', error)
          sendEvent({
            type: 'error',
            message: 'Failed to submit proof on-chain: ' + (error instanceof Error ? error.message : 'Unknown error'),
            timestamp: Date.now()
          })
        }

        controller.close()

      } catch (error) {
        console.error('Error in proof generation:', error)
        sendEvent({
          type: 'error',
          message: error instanceof Error ? error.message : 'Unknown error occurred',
          timestamp: Date.now()
        })
        controller.close()
      }
    }
  })

  return new Response(stream, {
    headers: {
      'Content-Type': 'text/event-stream',
      'Cache-Control': 'no-cache',
      'Connection': 'keep-alive',
    },
  })
}
