#!/bin/bash

# WebDAV Integration Tests
# Run with: ./tests/webdav_tests.sh

WEBDAV_URL="${WEBDAV_URL:-http://localhost:3001}"
WORKSPACE_ID="${WORKSPACE_ID:-workspace-195978c3}"
USERNAME="${USERNAME:-jueewo}"
PASSWORD="${PASSWORD:-test}"

echo "=========================================="
echo "WebDAV Integration Tests"
echo "=========================================="
echo "URL: $WEBDAV_URL"
echo "Workspace: $WORKSPACE_ID"
echo "Username: $USERNAME"
echo "=========================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

# Test counter
TESTS_PASSED=0
TESTS_FAILED=0

# Test file name (unique per run)
TEST_FILE="test_$(date +%s).txt"
TEST_CONTENT="WebDAV Test Content"

echo "Starting WebDAV server in background..."
cd /Users/juergen/MyDev/MyProjects/video-server-rs_v1
DATABASE_URL=sqlite:media.db STORAGE_DIR=./storage cargo run --package webdav > /tmp/webdav.log 2>&1 &
WEBDAV_PID=$!

cleanup() {
    echo ""
    echo "Stopping WebDAV server (PID: $WEBDAV_PID)..."
    kill $WEBDAV_PID 2>/dev/null || true
    wait $WEBDAV_PID 2>/dev/null || true
}
trap cleanup EXIT

# Wait for server to start
sleep 4

# Check if server is running
if ! curl -s -o /dev/null "$WEBDAV_URL/" 2>/dev/null; then
    echo -e "${RED}ERROR: WebDAV server failed to start${NC}"
    cat /tmp/webdav.log
    exit 1
fi

echo "Server started (PID: $WEBDAV_PID)"
echo ""

# Test function
test_http() {
    local name="$1"
    local expected="$2"
    shift 2
    local actual
    actual=$(curl -s -o /dev/null -w "%{http_code}" "$@")
    if [ "$actual" = "$expected" ]; then
        echo -e "Test: $name ... ${GREEN}PASS${NC} (HTTP $actual)"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo -e "Test: $name ... ${RED}FAIL${NC} (expected $expected, got $actual)"
        TESTS_FAILED=$((TESTS_FAILED + 1))
    fi
}

echo "Running tests..."
echo ""

# Test 1: Unauthorized access
test_http "Unauthorized access" "401" \
    "$WEBDAV_URL/dav/$WORKSPACE_ID/workspace.yaml"

# Test 2: GET file (authorized)
test_http "GET file (authorized)" "200" \
    -u "$USERNAME:$PASSWORD" "$WEBDAV_URL/dav/$WORKSPACE_ID/workspace.yaml"

# Test 3: PUT file (create new file)
test_http "PUT file (create)" "201" \
    -u "$USERNAME:$PASSWORD" -X PUT -d "$TEST_CONTENT" \
    "$WEBDAV_URL/dav/$WORKSPACE_ID/$TEST_FILE"

# Test 4: Verify file content
CONTENT=$(curl -s -u "$USERNAME:$PASSWORD" "$WEBDAV_URL/dav/$WORKSPACE_ID/$TEST_FILE")
if [ "$CONTENT" = "$TEST_CONTENT" ]; then
    echo -e "Test: Verify file content ... ${GREEN}PASS${NC}"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "Test: Verify file content ... ${RED}FAIL${NC} (content mismatch)"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi

# Test 5: DELETE file
test_http "DELETE file" "204" \
    -u "$USERNAME:$PASSWORD" -X DELETE \
    "$WEBDAV_URL/dav/$WORKSPACE_ID/$TEST_FILE"

# Test 6: Access non-owned workspace
test_http "Access non-owned workspace" "403" \
    -u "$USERNAME:$PASSWORD" "$WEBDAV_URL/dav/nonexistent-workspace/test.txt"

# Test 7: GET directory (PROPFIND returns 207)
test_http "GET directory (PROPFIND)" "207" \
    -u "$USERNAME:$PASSWORD" -X PROPFIND -H "Depth: 1" \
    "$WEBDAV_URL/dav/$WORKSPACE_ID"

# Test 8: PUT in subdirectory
test_http "PUT in subdirectory" "201" \
    -u "$USERNAME:$PASSWORD" -X PUT -d "subdir test" \
    "$WEBDAV_URL/dav/$WORKSPACE_ID/subdir/test.txt"

# Cleanup subdirectory
curl -s -u "$USERNAME:$PASSWORD" -X DELETE "$WEBDAV_URL/dav/$WORKSPACE_ID/subdir/test.txt" 2>/dev/null || true
rmdir /Users/juergen/MyDev/MyProjects/video-server-rs_v1/storage/workspaces/$WORKSPACE_ID/subdir 2>/dev/null || true

# Summary
echo ""
echo "=========================================="
echo "Test Summary"
echo "=========================================="
echo -e "Passed: ${GREEN}$TESTS_PASSED${NC}"
echo -e "Failed: ${RED}$TESTS_FAILED${NC}"
echo "=========================================="

if [ $TESTS_FAILED -gt 0 ]; then
    exit 1
else
    exit 0
fi
