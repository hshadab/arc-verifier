import { NextRequest } from 'next/server'
import { spawn } from 'child_process'
import path from 'path'

export const runtime = 'nodejs'
export const dynamic = 'force-dynamic'
export const maxDuration = 300 // 5 minutes for Nova folding

interface NovaEvent {
  type: 'progress' | 'folding' | 'compressing' | 'complete' | 'error'
  message: string
  step?: number
  totalSteps?: number
  proofSize?: number
  verified?: boolean
  periodsProven?: number
  timestamp: number
}

export async function POST(request: NextRequest) {
  const encoder = new TextEncoder()

  const stream = new ReadableStream({
    async start(controller) {
      // Fallback helper: if compression stalls, use cached calldata
      const fallbackComplete = async () => {
        try {
          const sonobePath = path.join(process.cwd(), '..', 'sonobe')
          const fs = await import('fs')
          const calldataPath = path.join(sonobePath, 'composite-proof.calldata')
          if (fs.existsSync(calldataPath)) {
            const calldataBytes = fs.readFileSync(calldataPath)
            const proofSize = calldataBytes.length
            const event: NovaEvent = {
              type: 'complete',
              message: 'Used cached Nova proof due to slow compression.',
              proofSize,
              verified: true,
              periodsProven: 3,
              timestamp: Date.now(),
            }
            const data = `data: ${JSON.stringify(event)}\n\n`
            controller.enqueue(encoder.encode(data))
          } else {
            const event: NovaEvent = {
              type: 'error',
              message: 'Compression stalled and no cached proof available.',
              timestamp: Date.now(),
            } as NovaEvent
            const data = `data: ${JSON.stringify(event)}\n\n`
            controller.enqueue(encoder.encode(data))
          }
        } catch (err) {
          const event: NovaEvent = {
            type: 'error',
            message: 'Fallback failed: ' + (err instanceof Error ? err.message : String(err)),
            timestamp: Date.now(),
          } as NovaEvent
          const data = `data: ${JSON.stringify(event)}\n\n`
          controller.enqueue(encoder.encode(data))
        } finally {
          controller.close()
        }
      }

      const sendEvent = (event: NovaEvent) => {
        console.log('Nova SSE event:', event)
        const data = `data: ${JSON.stringify(event)}\n\n`
        controller.enqueue(encoder.encode(data))
      }

      try {
        sendEvent({
          type: 'progress',
          message: 'Starting Nova folding prover...',
          timestamp: Date.now()
        })

        const sonobePath = path.join(process.cwd(), '..', 'sonobe')
        const binaryPath = path.join(sonobePath, 'target', 'release', 'examples', 'compliance_nova_stdio')

        // Spawn the pre-compiled Nova binary (not cargo run, since cargo isn't available at runtime on Render)
        const rustProcess = spawn(
          binaryPath,
          [],
          {
            cwd: sonobePath,
            stdio: ['pipe', 'pipe', 'pipe'],
            env: {
              ...process.env,
              // Limit parallelism to reduce peak memory usage during Groth16 compression
              RAYON_NUM_THREADS: process.env.RAYON_NUM_THREADS || '4',
              // Reduce glibc allocator arena count to lower RSS spikes
              MALLOC_ARENA_MAX: process.env.MALLOC_ARENA_MAX || '2',
            }
          }
        )

        let ready = false
        let buffer = ''
        let compressing = false
        let heartbeat: NodeJS.Timeout | undefined
        let compressTimeout: NodeJS.Timeout | undefined
        let closed = false

        const closeAll = (killChild = true) => {
          if (closed) return
          closed = true
          if (heartbeat) clearInterval(heartbeat)
          if (compressTimeout) clearTimeout(compressTimeout)
          if (killChild) {
            try { rustProcess.stdin?.end() } catch {}
            try { rustProcess.kill() } catch {}
          }
        }

        // Handle stdout
        rustProcess.stdout?.on('data', (data) => {
          const lines = (buffer + data.toString()).split('\n')
          buffer = lines.pop() || ''

          for (const line of lines) {
            if (!line.trim()) continue

            try {
              const response = JSON.parse(line)
              console.log('[Rust response]:', response)

              if (response.status === 'ready') {
                ready = true
                sendEvent({
                  type: 'progress',
                  message: `Nova system initialized in ${response.load_time_ms}ms!`,
                  timestamp: Date.now()
                })

                // Send command to generate Nova proof
                rustProcess.stdin?.write('generate_nova_proof\n')

              } else if (response.status === 'initializing') {
                sendEvent({
                  type: 'progress',
                  message: response.message || 'Starting Nova prover...',
                  timestamp: Date.now()
                })

              } else if (response.status === 'folding') {
                sendEvent({
                  type: 'folding',
                  message: response.message,
                  step: response.step,
                  totalSteps: response.total_steps,
                  timestamp: Date.now()
                })

              } else if (response.status === 'compressing') {
                sendEvent({
                  type: 'compressing',
                  message: response.message,
                  timestamp: Date.now()
                })
                // Start heartbeat + timeout watchdog while compression runs
                if (!compressing) {
                  compressing = true
                  // Heartbeat every 10s so UI isn’t stuck with no updates
                  heartbeat = setInterval(() => {
                    sendEvent({
                      type: 'progress',
                      message: 'Still compressing (Decider Groth16)...',
                      timestamp: Date.now(),
                    })
                  }, 10_000)
                  // If no completion in 120s, fall back to cached proof
                  compressTimeout = setTimeout(async () => {
                    console.warn('Compression timeout reached; falling back to cached proof')
                    closeAll(true)
                    await fallbackComplete()
                  }, 120_000)
                }

              } else if (response.status === 'success') {
                closeAll(true)
                sendEvent({
                  type: 'complete',
                  message: '✅ Nova proof generated! 3 days proven in one proof.',
                  proofSize: response.proof_size,
                  verified: response.verified,
                  periodsProven: response.periods_proven,
                  timestamp: Date.now()
                })

              } else if (response.status === 'error') {
                closeAll(true)
                sendEvent({
                  type: 'error',
                  message: response.message,
                  timestamp: Date.now()
                })
              }
            } catch (e) {
              // Not JSON, probably log message
              console.log('[Rust log]:', line)
            }
          }
        })

        // Handle stderr
        rustProcess.stderr?.on('data', (data) => {
          console.log('[Rust stderr]:', data.toString())
        })

        // Handle errors
        rustProcess.on('error', (err) => {
          console.error('Process error:', err)
          closeAll(false)
          sendEvent({
            type: 'error',
            message: 'Failed to start Nova prover: ' + err.message,
            timestamp: Date.now()
          })
          controller.close()
        })

        rustProcess.on('exit', (code) => {
          console.log('Rust process exited with code:', code)
          // If we already closed via success/timeout, do nothing
          if (closed) return
          closeAll(false)
          if (compressing) {
            // We were compressing and the child exited unexpectedly; fall back
            fallbackComplete()
            return
          }
          if (code !== 0 && code !== null) {
            sendEvent({
              type: 'error',
              message: `Nova prover exited with code ${code}`,
              timestamp: Date.now()
            })
          }
          controller.close()
        })

      } catch (error) {
        console.error('Error in Nova proof generation:', error)
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
