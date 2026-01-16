#!/bin/bash

# Test script for Access Codes API
# This script tests creating, listing, and using access codes

set -e

BASE_URL="http://localhost:3000"
TEST_CODE="test123"
VIDEO_SLUG="lesson1"
IMAGE_SLUG="secret"
COOKIE_FILE="test_cookies.txt"

echo "🧪 Testing Access Codes API"
echo "================================"

# Function to check if server is running
check_server() {
    if ! curl -s "${BASE_URL}/health" > /dev/null; then
        echo "❌ Server not running at ${BASE_URL}"
        echo "   Start the server first: cargo run"
        exit 1
    fi
}

# Function to authenticate using emergency login
authenticate() {
    echo "🔐 Authenticating with emergency login..."

    # Clean up any existing cookie file
    rm -f "$COOKIE_FILE"

    # Attempt emergency login
    RESPONSE=$(curl -s -c "$COOKIE_FILE" -b "$COOKIE_FILE" \
        -d "username=admin&password=testpass123" \
        "${BASE_URL}/login/emergency/auth")

    if echo "$RESPONSE" | grep -q "Emergency Login Successful"; then
        echo "✅ Authentication successful"
        return 0
    else
        echo "❌ Authentication failed"
        echo "Response: $RESPONSE"
        return 1
    fi
}

# Function to create test access code
create_access_code() {
    echo "📝 Creating access code '${TEST_CODE}'..."

    RESPONSE=$(curl -s -b "$COOKIE_FILE" -X POST "${BASE_URL}/api/access-codes" \
        -H "Content-Type: application/json" \
        -d "{
            \"code\": \"${TEST_CODE}\",
            \"description\": \"Test access code\",
            \"expires_at\": \"2027-12-31T23:59:59Z\",
            \"media_items\": [
                {\"media_type\": \"video\", \"media_slug\": \"${VIDEO_SLUG}\"},
                {\"media_type\": \"image\", \"media_slug\": \"${IMAGE_SLUG}\"}
            ]
        }")

    if echo "$RESPONSE" | grep -q "${TEST_CODE}"; then
        echo "✅ Access code created successfully"
        return 0
    else
        echo "❌ Failed to create access code"
        return 1
    fi
}

# Function to list access codes
list_access_codes() {
    echo "📋 Listing access codes..."

    RESPONSE=$(curl -s -b "$COOKIE_FILE" "${BASE_URL}/api/access-codes")

    if echo "$RESPONSE" | grep -q "${TEST_CODE}"; then
        echo "✅ Access code found in list"
        return 0
    else
        echo "❌ Access code not found in list"
        return 1
    fi
}

# Function to test video access with code
test_video_access() {
    echo "🎥 Testing video access with access code..."

    # Test without access code (should fail for private videos)
    RESPONSE_NO_CODE=$(curl -s -w "%{http_code}" -o /dev/null "${BASE_URL}/watch/${VIDEO_SLUG}")

    # Test with access code (should work)
    RESPONSE_WITH_CODE=$(curl -s -w "%{http_code}" -o /dev/null "${BASE_URL}/watch/${VIDEO_SLUG}?access_code=${TEST_CODE}")

    if [ "$RESPONSE_WITH_CODE" = "200" ]; then
        echo "✅ Video accessible with access code"
        return 0
    else
        echo "❌ Video not accessible with access code (HTTP $RESPONSE_WITH_CODE)"
        return 1
    fi
}

# Function to test image access with code
test_image_access() {
    echo "🖼️  Testing image access with access code..."

    # Test with access code (should work)
    RESPONSE=$(curl -s -w "%{http_code}" -o /dev/null "${BASE_URL}/images/${IMAGE_SLUG}?access_code=${TEST_CODE}")

    if [ "$RESPONSE" = "200" ]; then
        echo "✅ Image accessible with access code"
        return 0
    else
        echo "❌ Image not accessible with access code (HTTP $RESPONSE)"
        return 1
    fi
}

# Function to test expired access code
test_expired_code() {
    echo "⏰ Testing expired access code..."

    EXPIRED_CODE="expired456"

    # Create expired access code
    curl -s -b "$COOKIE_FILE" -X POST "${BASE_URL}/api/access-codes" \
        -H "Content-Type: application/json" \
        -d "{
            \"code\": \"${EXPIRED_CODE}\",
            \"description\": \"Expired test code\",
            \"expires_at\": \"2020-01-01T00:00:00Z\",
            \"media_items\": [
                {\"media_type\": \"video\", \"media_slug\": \"${VIDEO_SLUG}\"}
            ]
        }" > /dev/null

    # Test access (should fail)
    RESPONSE=$(curl -s -w "%{http_code}" -o /dev/null "${BASE_URL}/watch/${VIDEO_SLUG}?access_code=${EXPIRED_CODE}")

    if [ "$RESPONSE" = "401" ]; then
        echo "✅ Expired access code correctly rejected"
        return 0
    else
        echo "❌ Expired access code not rejected (HTTP $RESPONSE)"
        return 1
    fi
}

# Function to test ownership validation
test_ownership_validation() {
    echo "🔒 Testing ownership validation..."

    # Use timestamp to ensure unique code
    UNAUTHORIZED_CODE="unauthorized$(date +%s)"

    # Try to create access code for non-existent media (should fail with 403)
    RESPONSE=$(curl -s -w "%{http_code}" -o /dev/null -b "$COOKIE_FILE" -X POST "${BASE_URL}/api/access-codes" \
        -H "Content-Type: application/json" \
        -d "{
            \"code\": \"${UNAUTHORIZED_CODE}\",
            \"description\": \"Should fail - media not owned\",
            \"expires_at\": \"2027-12-31T23:59:59Z\",
            \"media_items\": [
                {\"media_type\": \"video\", \"media_slug\": \"nonexistent-video\"}
            ]
        }")

    if [ "$RESPONSE" = "403" ]; then
        echo "✅ Ownership validation working - access denied for unowned media"
        return 0
    else
        echo "❌ Ownership validation failed (HTTP $RESPONSE)"
        return 1
    fi
}

# Function to delete test access code
cleanup() {
    echo "🧹 Cleaning up test access codes..."

    curl -s -b "$COOKIE_FILE" -X DELETE "${BASE_URL}/api/access-codes/${TEST_CODE}" > /dev/null
    curl -s -b "$COOKIE_FILE" -X DELETE "${BASE_URL}/api/access-codes/expired456" > /dev/null

    # Clean up cookie file
    rm -f "$COOKIE_FILE"

    echo "✅ Cleanup completed"
}

# Main test execution
main() {
    echo "🔍 Checking server status..."
    check_server

    echo ""

    # Authenticate first
    if ! authenticate; then
        echo "❌ Cannot proceed without authentication"
        exit 1
    fi

    echo ""
    echo "🧪 Running Access Codes Tests"
    echo "================================"

    local tests_passed=0
    local total_tests=0

    # Test 1: Create access code
    ((total_tests++))
    if create_access_code; then
        ((tests_passed++))
    fi

    echo ""

    # Test 2: List access codes
    ((total_tests++))
    if list_access_codes; then
        ((tests_passed++))
    fi

    echo ""

    # Test 3: Video access
    ((total_tests++))
    if test_video_access; then
        ((tests_passed++))
    fi

    echo ""

    # Test 4: Image access
    ((total_tests++))
    if test_image_access; then
        ((tests_passed++))
    fi

    echo ""

    # Test 5: Expired code
    ((total_tests++))
    if test_expired_code; then
        ((tests_passed++))
    fi

    echo ""

    # Test 6: Ownership validation
    ((total_tests++))
    if test_ownership_validation; then
        ((tests_passed++))
    fi

    echo ""
    echo "📊 Test Results: $tests_passed/$total_tests passed"

    if [ "$tests_passed" -eq "$total_tests" ]; then
        echo "🎉 All tests passed!"
    else
        echo "❌ Some tests failed"
        exit 1
    fi

    echo ""
    cleanup
}

# Run main function
main "$@"
