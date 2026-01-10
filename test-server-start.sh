#!/bin/bash

# Quick server startup test script
set -e

echo "🔧 Testing Server Startup"
echo "========================="
echo ""

# Check if port 3000 is in use
if lsof -ti:3000 > /dev/null 2>&1; then
    echo "⚠️  Port 3000 is already in use. Stopping existing process..."
    lsof -ti:3000 | xargs kill -9 2>/dev/null || true
    sleep 2
fi

# Remove old database
if [ -f "video.db" ]; then
    echo "🗑️  Removing old database..."
    rm video.db
fi

# Ensure directories exist
echo "📁 Creating storage directories..."
mkdir -p storage/images/public
mkdir -p storage/images/private
mkdir -p storage/public
mkdir -p storage/private

# Build the project
echo ""
echo "🔨 Building project..."
cargo build --release

# Start server in background
echo ""
echo "🚀 Starting server..."
cargo run --release > /tmp/video-server.log 2>&1 &
SERVER_PID=$!

# Wait for server to start
echo "⏳ Waiting for server to be ready..."
sleep 3

# Check if server is running
if ! kill -0 $SERVER_PID 2>/dev/null; then
    echo "❌ Server failed to start. Check logs:"
    cat /tmp/video-server.log
    exit 1
fi

# Test health endpoint
echo ""
echo "🧪 Testing health endpoint..."
if curl -s -f http://localhost:3000/health > /dev/null; then
    echo "✅ Server is running and healthy!"
else
    echo "❌ Health check failed"
    kill $SERVER_PID 2>/dev/null
    cat /tmp/video-server.log
    exit 1
fi

# Check database
echo ""
echo "🗄️  Checking database..."
if [ -f "video.db" ]; then
    echo "✅ Database created"

    # Check tables
    TABLES=$(sqlite3 video.db "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE '_sqlx%';")
    echo "   Tables: $TABLES"

    if echo "$TABLES" | grep -q "videos"; then
        echo "   ✅ Videos table exists"
    else
        echo "   ❌ Videos table missing"
    fi

    if echo "$TABLES" | grep -q "images"; then
        echo "   ✅ Images table exists"
    else
        echo "   ❌ Images table missing"
    fi

    # Count records
    VIDEO_COUNT=$(sqlite3 video.db "SELECT COUNT(*) FROM videos;" 2>/dev/null || echo "0")
    IMAGE_COUNT=$(sqlite3 video.db "SELECT COUNT(*) FROM images;" 2>/dev/null || echo "0")
    echo "   Videos: $VIDEO_COUNT records"
    echo "   Images: $IMAGE_COUNT records"
else
    echo "❌ Database not created"
fi

# Test endpoints
echo ""
echo "🌐 Testing endpoints..."

# Test home page
if curl -s -f http://localhost:3000/ > /dev/null; then
    echo "✅ Home page (/) works"
else
    echo "❌ Home page (/) failed"
fi

# Test images gallery
if curl -s -f http://localhost:3000/images > /dev/null; then
    echo "✅ Images gallery (/images) works"
else
    echo "❌ Images gallery (/images) failed"
fi

# Test login
if curl -s -f http://localhost:3000/login > /dev/null; then
    echo "✅ Login (/login) works"
else
    echo "❌ Login (/login) failed"
fi

echo ""
echo "========================="
echo "✅ Server is running successfully!"
echo ""
echo "Server PID: $SERVER_PID"
echo "Log file: /tmp/video-server.log"
echo ""
echo "Available endpoints:"
echo "  • Home:    http://localhost:3000/"
echo "  • Images:  http://localhost:3000/images"
echo "  • Login:   http://localhost:3000/login"
echo "  • Upload:  http://localhost:3000/upload"
echo "  • Health:  http://localhost:3000/health"
echo ""
echo "To stop the server: kill $SERVER_PID"
echo "To view logs: tail -f /tmp/video-server.log"
echo ""
