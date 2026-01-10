#!/bin/bash

# Casdoor Token Format Diagnostic Script
# This script tests what format Casdoor returns from the token endpoint

set -e

echo "🔍 Casdoor Token Format Diagnostic"
echo "=================================="
echo ""

# Load .env if exists
if [ -f .env ]; then
    echo "📝 Loading configuration from .env..."
    export $(cat .env | grep -v '^#' | xargs)
else
    echo "⚠️  No .env file found, using defaults"
fi

# Configuration
ISSUER_URL="${OIDC_ISSUER_URL:-http://localhost:8088}"
CLIENT_ID="${OIDC_CLIENT_ID}"
CLIENT_SECRET="${OIDC_CLIENT_SECRET}"
REDIRECT_URI="${OIDC_REDIRECT_URI:-http://localhost:3000/oidc/callback}"

echo ""
echo "Configuration:"
echo "  Issuer: $ISSUER_URL"
echo "  Client ID: ${CLIENT_ID:0:20}..."
echo "  Redirect URI: $REDIRECT_URI"
echo ""

# Check if credentials are set
if [ -z "$CLIENT_ID" ] || [ -z "$CLIENT_SECRET" ]; then
    echo "❌ Error: OIDC_CLIENT_ID and OIDC_CLIENT_SECRET must be set in .env"
    echo ""
    echo "Please create a .env file with:"
    echo "  OIDC_ISSUER_URL=http://localhost:8088"
    echo "  OIDC_CLIENT_ID=your-client-id"
    echo "  OIDC_CLIENT_SECRET=your-client-secret"
    echo "  OIDC_REDIRECT_URI=http://localhost:3000/oidc/callback"
    exit 1
fi

# Step 1: Check Casdoor is reachable
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Step 1: Checking Casdoor Discovery Endpoint"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

DISCOVERY_URL="$ISSUER_URL/.well-known/openid-configuration"
echo "Fetching: $DISCOVERY_URL"
echo ""

if ! DISCOVERY=$(curl -s "$DISCOVERY_URL" 2>&1); then
    echo "❌ Failed to connect to Casdoor"
    echo "   Make sure Casdoor is running on $ISSUER_URL"
    exit 1
fi

# Parse discovery document
TOKEN_ENDPOINT=$(echo "$DISCOVERY" | grep -o '"token_endpoint":"[^"]*"' | cut -d'"' -f4)
AUTH_ENDPOINT=$(echo "$DISCOVERY" | grep -o '"authorization_endpoint":"[^"]*"' | cut -d'"' -f4)

if [ -z "$TOKEN_ENDPOINT" ]; then
    echo "❌ Invalid discovery response"
    echo "Response:"
    echo "$DISCOVERY" | head -20
    exit 1
fi

echo "✅ Discovery successful"
echo "   Authorization: $AUTH_ENDPOINT"
echo "   Token: $TOKEN_ENDPOINT"
echo ""

# Step 2: Check application configuration via token endpoint
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Step 2: Testing Token Endpoint (Invalid Grant Expected)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Testing with invalid code to see response format..."
echo ""

# Test token endpoint with invalid code
RESPONSE=$(curl -s -w "\nHTTP_STATUS:%{http_code}" -X POST "$TOKEN_ENDPOINT" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "grant_type=authorization_code" \
  -d "code=test_invalid_code_12345" \
  -d "client_id=$CLIENT_ID" \
  -d "client_secret=$CLIENT_SECRET" \
  -d "redirect_uri=$REDIRECT_URI" 2>&1)

HTTP_STATUS=$(echo "$RESPONSE" | grep "HTTP_STATUS:" | cut -d':' -f2)
BODY=$(echo "$RESPONSE" | sed '/HTTP_STATUS:/d')

echo "HTTP Status: $HTTP_STATUS"
echo ""
echo "Response Body:"
echo "$BODY"
echo ""

# Check if response is JSON
if echo "$BODY" | jq . >/dev/null 2>&1; then
    echo "✅ Response is valid JSON"
    echo ""

    # Parse error response
    ERROR_TYPE=$(echo "$BODY" | jq -r '.error // empty')
    ERROR_DESC=$(echo "$BODY" | jq -r '.error_description // empty')

    if [ -n "$ERROR_TYPE" ]; then
        echo "Error Type: $ERROR_TYPE"
        echo "Error Description: $ERROR_DESC"
    fi

    # Check what fields are present
    echo ""
    echo "Response structure:"
    echo "$BODY" | jq 'keys'
else
    echo "❌ Response is NOT valid JSON!"
    echo "   This is likely the problem!"
    echo ""
    echo "Raw response (first 500 chars):"
    echo "$BODY" | head -c 500
    echo ""
fi

# Step 3: Test with client credentials grant (if you want actual tokens)
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Step 3: Checking Token Format Configuration"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "⚠️  We can't test actual token format without a real authorization code."
echo "    You need to complete a full login flow to get a real code."
echo ""

# Step 4: Provide diagnostic information
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Diagnostic Summary"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

if echo "$BODY" | jq . >/dev/null 2>&1; then
    echo "✅ Casdoor is returning JSON responses"
    echo ""
    echo "Next steps to debug 'Failed to parse server response':"
    echo ""
    echo "1. Check Token Format in Casdoor:"
    echo "   - Open Casdoor Admin → Applications → Your App"
    echo "   - Look for 'Token Format' setting"
    echo "   - Make sure it's set to: JWT (not Opaque)"
    echo ""
    echo "2. Complete a real login and check server logs:"
    echo "   - cargo run"
    echo "   - Visit http://localhost:3000/login"
    echo "   - Login with Casdoor"
    echo "   - Check the detailed error in server console"
    echo ""
    echo "3. If server logs show 'Failed to parse', the issue is likely:"
    echo "   - Token Format = Opaque (change to JWT)"
    echo "   - Response missing 'id_token' field (required for OIDC)"
    echo "   - Response has unexpected structure"
else
    echo "❌ Casdoor is NOT returning valid JSON!"
    echo ""
    echo "This is definitely the problem. Possible causes:"
    echo ""
    echo "1. Wrong token endpoint URL"
    echo "   Current: $TOKEN_ENDPOINT"
    echo ""
    echo "2. Casdoor misconfiguration"
    echo "   - Check Casdoor is properly installed"
    echo "   - Check Casdoor logs for errors"
    echo ""
    echo "3. Network/proxy issues"
    echo "   - Check firewall settings"
    echo "   - Check if proxy is interfering"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "To get more detailed diagnostics:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Run the server with debug logging:"
echo "  RUST_LOG=debug cargo run"
echo ""
echo "Then try logging in and look for these log lines:"
echo "  🔍 Exchanging authorization code for tokens..."
echo "  ❌ Token exchange failed: ..."
echo "  Error details: ..."
echo ""
echo "The error details will tell us exactly what's wrong with the response."
echo ""
