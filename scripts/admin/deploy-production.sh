#!/bin/bash
set -e

echo "🚀 Production Deployment Script"
echo "================================"

# Configuration
PROD_SERVER="media.appkask.com"
PROD_USER="root"
PROD_DIR="/root/data/rust-apps/video-server-rs_v1"
BINARY_PATH="/usr/local/bin/video-server-rs"

# Step 1: Build frontend bundle
echo ""
echo "📦 Step 1: Building frontend bundle..."
cd crates/3d-gallery/frontend
npm run build
cd ../../..

# Step 2: Build Rust binary (release mode)
echo ""
echo "🦀 Step 2: Building Rust binary (release mode)..."
cargo build --release

# Step 3: Copy static files to production
echo ""
echo "📤 Step 3: Copying static files to production..."
ssh ${PROD_USER}@${PROD_SERVER} "mkdir -p ${PROD_DIR}/crates/3d-gallery/static"
scp crates/3d-gallery/static/bundle.js ${PROD_USER}@${PROD_SERVER}:${PROD_DIR}/crates/3d-gallery/static/
scp crates/3d-gallery/static/bundle.js.map ${PROD_USER}@${PROD_SERVER}:${PROD_DIR}/crates/3d-gallery/static/

# Step 4: Copy binary to production
echo ""
echo "📤 Step 4: Copying binary to production..."
scp target/release/video-server-rs ${PROD_USER}@${PROD_SERVER}:${BINARY_PATH}

# Step 5: Restart service
echo ""
echo "🔄 Step 5: Restarting service..."
ssh ${PROD_USER}@${PROD_SERVER} "systemctl restart video-server-rs_v1"

# Step 6: Check status
echo ""
echo "✅ Step 6: Checking service status..."
ssh ${PROD_USER}@${PROD_SERVER} "systemctl status video-server-rs_v1 --no-pager | head -20"

echo ""
echo "🎉 Deployment complete!"
echo "🌐 Visit: https://media.appkask.com/3d?code=gallery3d"
