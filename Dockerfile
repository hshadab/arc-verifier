# Multi-stage build for Arc Nova Demo
# Stage 1: Build Rust binaries and generate parameters
FROM rust:1.75-slim-bookworm AS rust-builder

WORKDIR /build

# Install build dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy Rust workspace
COPY sonobe ./sonobe
COPY arecibo ./arecibo
COPY Cargo.toml Cargo.lock ./

# Build Rust binaries in release mode
WORKDIR /build/sonobe
RUN cargo build --release -p folding-schemes --example compliance_nova_stdio
RUN cargo build --release -p folding-schemes --example compliance_groth16_stdio

# Generate parameters if they don't exist
RUN if [ ! -d "persisted_params" ]; then \
        echo "Generating parameters (~5 minutes)..."; \
        timeout 600 ../target/release/examples/compliance_nova_stdio <<EOF || true; \
    fi

# Stage 2: Build Next.js application
FROM node:20-slim AS node-builder

WORKDIR /app

# Copy package files
COPY demo/package*.json ./

# Install dependencies
RUN npm ci --only=production

# Copy demo source
COPY demo ./

# Build Next.js
RUN npm run build

# Stage 3: Production runtime
FROM node:20-slim

WORKDIR /app

# Install runtime dependencies for Rust binaries
RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy Next.js build from node-builder
COPY --from=node-builder /app ./demo
COPY --from=node-builder /app/node_modules ./demo/node_modules

# Copy Rust binaries and parameters from rust-builder
COPY --from=rust-builder /build/target/release/examples/compliance_nova_stdio ./sonobe/target/release/examples/
COPY --from=rust-builder /build/target/release/examples/compliance_groth16_stdio ./sonobe/target/release/examples/
COPY --from=rust-builder /build/sonobe/persisted_params ./sonobe/persisted_params
COPY --from=rust-builder /build/sonobe/composite-proof.calldata ./sonobe/composite-proof.calldata

# Set environment variables
ENV NODE_ENV=production
ENV RAYON_NUM_THREADS=4
ENV MALLOC_ARENA_MAX=2
ENV PORT=3000

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD node -e "require('http').get('http://localhost:3000', (r) => {process.exit(r.statusCode === 200 ? 0 : 1)})"

# Start Next.js server
WORKDIR /app/demo
CMD ["npm", "start"]
