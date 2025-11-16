import { NextRequest, NextResponse } from 'next/server';
import { getComplianceService } from '@/lib/compliance-service';

export const runtime = 'nodejs';
export const dynamic = 'force-dynamic';

export async function GET(request: NextRequest) {
  try {
    const service = getComplianceService();
    const isReady = service.isReady();

    if (!isReady) {
      return NextResponse.json({
        status: 'not_initialized',
        ready: false,
        message: 'Service not initialized. Call POST /api/initialize first.',
      });
    }

    // Try to get status from service
    try {
      const statusResponse = await service.getStatus();
      return NextResponse.json({
        status: 'ready',
        ready: true,
        params_loaded: statusResponse.params_loaded ?? true,
        message: 'Service ready for proof generation',
      });
    } catch (error) {
      // If command fails, service might not be fully ready
      return NextResponse.json({
        status: 'initializing',
        ready: false,
        message: 'Service is starting up...',
      });
    }
  } catch (error) {
    console.error('Status check error:', error);
    return NextResponse.json(
      {
        status: 'error',
        ready: false,
        message: error instanceof Error ? error.message : 'Unknown error',
      },
      { status: 500 }
    );
  }
}
