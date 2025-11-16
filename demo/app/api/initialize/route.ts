import { NextRequest, NextResponse } from 'next/server';
import { getComplianceService } from '@/lib/compliance-service';

export const runtime = 'nodejs';
export const dynamic = 'force-dynamic';

export async function POST(request: NextRequest) {
  const encoder = new TextEncoder();

  const stream = new ReadableStream({
    async start(controller) {
      try {
        const service = getComplianceService();

        if (service.isReady()) {
          // Already initialized
          controller.enqueue(
            encoder.encode(`data: ${JSON.stringify({
              type: 'complete',
              message: 'System already initialized',
              already_ready: true,
              load_time_ms: 0,
            })}\n\n`)
          );
          controller.close();
          return;
        }

        // Send progress events
        controller.enqueue(
          encoder.encode(`data: ${JSON.stringify({
            type: 'progress',
            message: 'Starting Groth16 compliance service...',
            step: 1,
            totalSteps: 3,
          })}\n\n`)
        );

        controller.enqueue(
          encoder.encode(`data: ${JSON.stringify({
            type: 'progress',
            message: 'Loading Groth16 proving and verifying keys from disk...',
            step: 2,
            totalSteps: 3,
          })}\n\n`)
        );

        controller.enqueue(
          encoder.encode(`data: ${JSON.stringify({
            type: 'progress',
            message: 'Loading keys into memory (~2-3 seconds)...',
            step: 3,
            totalSteps: 3,
          })}\n\n`)
        );

        // Initialize (takes ~83s)
        const startTime = Date.now();
        const result = await service.initialize();
        const actualDuration = Date.now() - startTime;

        controller.enqueue(
          encoder.encode(`data: ${JSON.stringify({
            type: 'complete',
            message: 'System initialized! Ready for proof generation.',
            load_time_ms: result.load_time_ms || actualDuration,
            already_ready: false,
          })}\n\n`)
        );

        controller.close();
      } catch (error) {
        console.error('Initialization error:', error);
        controller.enqueue(
          encoder.encode(`data: ${JSON.stringify({
            type: 'error',
            message: error instanceof Error ? error.message : 'Unknown error during initialization',
          })}\n\n`)
        );
        controller.close();
      }
    },
  });

  return new Response(stream, {
    headers: {
      'Content-Type': 'text/event-stream',
      'Cache-Control': 'no-cache',
      'Connection': 'keep-alive',
    },
  });
}
