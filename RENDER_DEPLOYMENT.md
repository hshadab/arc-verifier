# Deploying Nova Folding Demo to Render.com

This guide covers deploying the Arc Nova folding proof generation demo to Render.com.

## Prerequisites

- GitHub account with this repository
- Render.com account (free to sign up)
- Pre-generated parameter files in `sonobe/persisted_params/` (commit these to your repo)

## Quick Start

### Option 1: Deploy with render.yaml (Recommended)

1. **Push your code to GitHub** (if not already done):
   ```bash
   git add .
   git commit -m "Add Render deployment configuration"
   git push origin main
   ```

2. **Create New Web Service on Render**:
   - Go to https://dashboard.render.com/
   - Click "New +" → "Web Service"
   - Connect your GitHub repository
   - Render will auto-detect `render.yaml`

3. **Configure the deployment**:
   - Service name: `arc-nova-demo` (auto-filled from render.yaml)
   - Plan: **Pro ($85/mo)** ✅ Required for 8GB RAM
   - Region: Choose closest to your users
   - Click "Create Web Service"

4. **Wait for build** (~10-15 minutes first time):
   - Rust compilation: ~5-8 minutes
   - Parameter loading/generation: ~2-5 minutes
   - Next.js build: ~1-2 minutes

5. **Access your app**:
   - URL: `https://arc-nova-demo.onrender.com` (or your custom domain)
   - Test Nova proof: Click "Generate Nova Proof" button

### Option 2: Manual Configuration

If you prefer manual setup or `render.yaml` doesn't work:

1. **Create New Web Service**:
   - Runtime: **Node**
   - Build Command:
     ```bash
     curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && \
     source $HOME/.cargo/env && \
     rustup default stable && \
     cd sonobe && \
     cargo build --release -p folding-schemes --example compliance_nova_stdio && \
     cargo build --release -p folding-schemes --example compliance_groth16_stdio && \
     cd ../demo && \
     npm install && \
     npm run build
     ```
   - Start Command:
     ```bash
     cd demo && npm start
     ```

2. **Environment Variables**:
   ```
   NODE_ENV=production
   RAYON_NUM_THREADS=4
   MALLOC_ARENA_MAX=2
   PORT=3000
   ```

3. **Select Plan**: Pro ($85/mo) - 8GB RAM, 4 vCPUs

## Option 3: Docker Deployment

For more control, use Docker:

1. **Update render.yaml** to use Docker runtime:
   ```yaml
   services:
     - type: web
       name: arc-nova-demo
       runtime: docker
       plan: pro
       dockerfilePath: ./Dockerfile
       dockerContext: .
   ```

2. **Push and deploy** - Render will build using the Dockerfile

## Performance Expectations

| Phase | Time | Notes |
|-------|------|-------|
| First build | 10-15 min | Includes Rust compilation |
| Subsequent builds | 8-12 min | Cached dependencies |
| Parameter loading | 58s | On each app start |
| Nova folding (3 steps) | 2-3s | Per proof request |
| Decider compression | 50-70s | With 8GB RAM |
| **Total proof time** | ~60-75s | With watchdog fallback |

## Resource Requirements

### Minimum (May timeout):
- **Plan**: Standard ($25/mo)
- **RAM**: 4GB
- **vCPUs**: 2
- **Status**: ⚠️ Borderline - Decider may hit 120s timeout

### Recommended:
- **Plan**: Pro ($85/mo) ✅
- **RAM**: 8GB
- **vCPUs**: 4
- **Status**: ✅ Reliable - Decider completes in 50-70s

### Optimal (Fastest):
- **Plan**: Pro Plus ($250/mo)
- **RAM**: 16GB
- **vCPUs**: 8
- **Status**: ✅ Fast - Decider completes in 30-40s

## Pre-deployment Checklist

### 1. Commit Parameter Files

**IMPORTANT**: You must commit pre-generated parameters to avoid generating them on every deploy (which takes 5+ minutes).

```bash
# Ensure parameters exist
ls -lh sonobe/persisted_params/
# Should show:
# - decider_pp.bin (~87MB)
# - nova_prover_params.bin (~2.1MB)
# - nova_cf_cs_vp.bin (~65KB)
# - nova_cs_vp.bin (~192 bytes)
# - decider_vp.bin (~1.7KB)

# Check if cached proof exists
ls -lh sonobe/composite-proof.calldata
# Should be ~900 bytes

# Commit them
git add sonobe/persisted_params/ sonobe/composite-proof.calldata
git commit -m "Add pre-generated Nova parameters"
git push
```

### 2. Update .gitignore

Make sure these files are NOT ignored:

```gitignore
# Allow parameter files (needed for deployment)
!sonobe/persisted_params/*.bin
!sonobe/composite-proof.calldata
```

### 3. Test Locally

Before deploying, test that everything works:

```bash
# Build Rust binaries
cd sonobe
cargo build --release -p folding-schemes --example compliance_nova_stdio

# Test Next.js server
cd ../demo
npm install
npm run build
npm start

# In another terminal, test the API
curl -X POST http://localhost:3000/api/generate-nova-proof
```

## Troubleshooting

### Build fails: "Rust not found"

The build command installs Rust automatically. If it fails:
- Check Render build logs for errors
- Try updating rustup installation command

### Build times out

Render has a 20-minute build timeout. If exceeded:
- Ensure you've committed `persisted_params/` to avoid regeneration
- Consider using Docker for faster builds with layer caching

### App crashes: "Out of memory"

Decider proof requires 4-8GB RAM:
- Upgrade to Pro plan (8GB)
- Check environment variables are set: `RAYON_NUM_THREADS=4`, `MALLOC_ARENA_MAX=2`

### Watchdog timeout triggers every time

If Decider always hits 120s timeout:
- Check RAM allocation (need at least 8GB)
- Monitor Render metrics during proof generation
- Consider upgrading to Pro Plus (16GB)

### Parameters not found

If logs show "Generating parameters...":
- Verify `sonobe/persisted_params/*.bin` files are committed
- Check `.gitignore` isn't excluding them
- Ensure files are in the deployed build (check Render shell)

## Monitoring

### Render Dashboard

Monitor your deployment:
- **Metrics**: CPU, Memory, Response times
- **Logs**: Real-time application logs
- **Events**: Deployments, crashes, restarts

### Key Metrics to Watch

- **Memory usage during Decider**: Should stay under 7GB on Pro plan
- **Response time for `/api/generate-nova-proof`**: 60-75s typical
- **CPU usage**: Spikes to 100% during Decider (normal)

### Logs to Check

```bash
# Via Render dashboard or CLI
render logs arc-nova-demo

# Look for:
# - "Nova system initialized in XXXXms"
# - "Folding compliance check 1/2/3"
# - "Compressing proof with Decider"
# - "Still compressing..." (heartbeat)
# - "✅ Nova proof generated!" or "Used cached Nova proof"
```

## Cost Optimization

### Development

For testing/development, use:
- **Free tier**: Not recommended (512MB RAM - will fail)
- **Starter ($7/mo)**: Not enough RAM (512MB)
- **Standard ($25/mo)**: Borderline (4GB RAM)

### Production

For production/demo:
- **Pro ($85/mo)**: ✅ Best balance of cost and performance
- **Pro Plus ($250/mo)**: Only if you need sub-40s proof times

### Cost per Proof

Assuming Pro plan ($85/mo):
- Fixed cost (not usage-based)
- Unlimited proof generations
- **Cost per proof**: Effectively $0 (flat monthly rate)

Compare to AWS EC2 r7i.2xlarge:
- **Render Pro**: $85/mo flat
- **AWS**: $320/mo + data transfer

**Render saves ~$235/mo** vs comparable AWS instance.

## Next Steps

After deploying:

1. **Test the live app**:
   ```bash
   curl -X POST https://your-app.onrender.com/api/generate-nova-proof
   ```

2. **Set up custom domain** (optional):
   - Go to Settings → Custom Domain
   - Add your domain and configure DNS

3. **Enable auto-deploy**:
   - Settings → Auto-Deploy
   - Deploy on every push to `main` branch

4. **Set up monitoring**:
   - Consider adding Sentry for error tracking
   - Set up uptime monitoring (Render has built-in health checks)

## Support

- Render docs: https://render.com/docs
- Render community: https://community.render.com
- This repo issues: https://github.com/your-repo/issues

---

**Estimated deployment time**: 15-20 minutes (first deploy)
**Recommended plan**: Pro ($85/mo)
**Expected proof time**: 60-75 seconds
