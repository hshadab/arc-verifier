import { spawn, ChildProcess } from 'child_process';
import path from 'path';
import readline from 'readline';

interface ServiceCommand {
  command: 'status' | 'generate_proof';
}

interface ServiceResponse {
  status?: string;
  success?: boolean;
  duration_ms?: number;
  verified?: boolean;
  params_loaded?: boolean;
  load_time_ms?: number;
}

class ComplianceService {
  private process: ChildProcess | null = null;
  private ready: boolean = false;
  private pendingCommands: Map<number, {
    resolve: (value: any) => void;
    reject: (error: any) => void;
  }> = new Map();
  private commandId: number = 0;
  private initResolve: ((value: { load_time_ms: number }) => void) | null = null;
  private initReject: ((error: any) => void) | null = null;

  async initialize(): Promise<{ load_time_ms: number }> {
    if (this.ready) {
      return { load_time_ms: 0 }; // Already initialized
    }

    if (this.process) {
      // Already initializing
      return new Promise((resolve, reject) => {
        this.initResolve = resolve;
        this.initReject = reject;
      });
    }

    return new Promise((resolve, reject) => {
      const sonobePath = path.join(process.cwd(), '..', 'sonobe');
      const binaryPath = path.join(sonobePath, 'target', 'release', 'examples', 'compliance_groth16_stdio');

      console.log('🚀 Starting Rust compliance service...');
      console.log('Binary path:', binaryPath);
      console.log('Working directory:', sonobePath);

      this.initResolve = resolve;
      this.initReject = reject;

      this.process = spawn(
        binaryPath,
        [],
        {
          cwd: sonobePath,
          stdio: ['pipe', 'pipe', 'pipe'],
        }
      );

      const rl = readline.createInterface({
        input: this.process.stdout!,
        crlfDelay: Infinity,
      });

      // Listen for stdout (JSON responses)
      rl.on('line', (line) => {
        console.log('[Service stdout]:', line);

        try {
          const response = JSON.parse(line);

          if (response.status === 'ready' && !this.ready) {
            // Service initialized
            this.ready = true;
            console.log(`✅ Service ready! Loaded in ${response.load_time_ms}ms`);

            if (this.initResolve) {
              this.initResolve({ load_time_ms: response.load_time_ms });
              this.initResolve = null;
              this.initReject = null;
            }
          } else {
            // Response to a command
            this.handleResponse(response);
          }
        } catch (e) {
          // Not JSON, probably a log message
          console.log('[Rust log]:', line);
        }
      });

      // Listen for stderr (logs)
      this.process.stderr!.on('data', (data) => {
        const message = data.toString();
        console.log('[Rust stderr]:', message);

        // Check if this is an actual error during initialization (not cargo compilation output)
        if (!this.ready && (message.includes('error:') || message.includes('Error:')) && !message.includes('Compiling')) {
          if (this.initReject) {
            this.initReject(new Error(message));
            this.initReject = null;
            this.initResolve = null;
          }
        }
      });

      // Handle errors
      this.process.on('error', (err) => {
        console.error('Process error:', err);
        if (this.initReject) {
          this.initReject(err);
          this.initReject = null;
          this.initResolve = null;
        }
      });

      this.process.on('exit', (code) => {
        console.log(`Service exited with code ${code}`);
        this.ready = false;
        this.process = null;

        if (this.initReject && code !== 0) {
          this.initReject(new Error(`Service exited with code ${code}`));
          this.initReject = null;
          this.initResolve = null;
        }

        // Reject all pending commands
        const entries = Array.from(this.pendingCommands.entries());
        for (const [id, { reject }] of entries) {
          reject(new Error('Service stopped'));
        }
        this.pendingCommands.clear();
      });
    });
  }

  async sendCommand(command: ServiceCommand): Promise<ServiceResponse> {
    if (!this.ready || !this.process) {
      throw new Error('Service not initialized. Call initialize() first.');
    }

    return new Promise((resolve, reject) => {
      const id = this.commandId++;
      this.pendingCommands.set(id, { resolve, reject });

      // Send plain text command (not JSON) - Rust stdio expects "generate_proof\n"
      const commandStr = command.command + '\n';
      console.log('[Sending command]:', commandStr.trim());
      this.process!.stdin!.write(commandStr);

      // Timeout after 300 seconds (5 minutes for Decider proof generation)
      setTimeout(() => {
        if (this.pendingCommands.has(id)) {
          this.pendingCommands.delete(id);
          reject(new Error('Command timeout (300s)'));
        }
      }, 300000);
    });
  }

  private handleResponse(response: ServiceResponse) {
    console.log('[Received response]:', response);

    // Only resolve on final responses (success or error), not intermediate statuses
    if (response.status === 'generating') {
      console.log('[Intermediate status - waiting for final response]');
      return;
    }

    // Resolve the first pending command (FIFO)
    const entries = Array.from(this.pendingCommands.entries());
    if (entries.length > 0) {
      const [id, { resolve }] = entries[0];
      this.pendingCommands.delete(id);
      resolve(response);
    }
  }

  async generateProof(): Promise<ServiceResponse> {
    return this.sendCommand({ command: 'generate_proof' });
  }

  async getStatus(): Promise<ServiceResponse> {
    return this.sendCommand({ command: 'status' });
  }

  isReady(): boolean {
    return this.ready;
  }

  shutdown() {
    if (this.process) {
      this.process.kill();
      this.process = null;
      this.ready = false;
    }
  }
}

// Singleton instance (persists across Next.js hot reloads)
declare global {
  var complianceServiceInstance: ComplianceService | undefined;
}

export function getComplianceService(): ComplianceService {
  if (!global.complianceServiceInstance) {
    global.complianceServiceInstance = new ComplianceService();
  }
  return global.complianceServiceInstance;
}
