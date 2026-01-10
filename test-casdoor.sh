#!/bin/bash

# Casdoor Configuration Test Script
# This script tests your Casdoor OIDC configuration

set -e

echo "╔═══════════════════════════════════════════════════════════════╗"
echo "║          Casdoor OIDC Configuration Test                      ║"
echo "╚═══════════════════════════════════════════════════════════════╝"
echo ""

# Load environment variables
if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
    echo "✅ Loaded .env file"
else
    echo "❌ .env file not found"
    echo "   Create one from: cp .env.example .env"
    exit 1
fi

echo ""
echo "📋 Configuration:"
echo "   OIDC_ISSUER_URL: $OIDC_ISSUER_URL"
echo "   OIDC_CLIENT_ID: $OIDC_CLIENT_ID"
echo "   OIDC_CLIENT_SECRET: ${OIDC_CLIENT_SECRET:0:10}..."
echo "   OIDC_REDIRECT_URI: $OIDC_REDIRECT_URI"
echo ""

# Test 1: Check Casdoor is reachable
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Test 1: Checking if Casdoor is reachable..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if curl -s --max-time 5 "$OIDC_ISSUER_URL" > /dev/null; then
    echo "✅ Casdoor server is reachable at $OIDC_ISSUER_URL"
else
    echo "❌ Cannot reach Casdoor at $OIDC_ISSUER_URL"
    echo "   Is Casdoor running?"
    exit 1
fi
echo ""

# Test 2: Check OIDC Discovery endpoint
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Test 2: Checking OIDC Discovery endpoint..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

DISCOVERY_URL="$OIDC_ISSUER_URL/.well-known/openid-configuration"
echo "   URL: $DISCOVERY_URL"

if DISCOVERY=$(curl -s --max-time 10 "$DISCOVERY_URL"); then
    if echo "$DISCOVERY" | grep -q "issuer"; then
        echo "✅ OIDC Discovery endpoint is working"
        echo ""
        echo "   Discovered endpoints:"

        if command -v jq &> /dev/null; then
            echo "$DISCOVERY" | jq -r '
                "   • Issuer: \(.issuer)",
                "   • Authorization: \(.authorization_endpoint)",
                "   • Token: \(.token_endpoint)",
                "   • Userinfo: \(.userinfo_endpoint)"
            '
        else
            echo "   (install jq for pretty output)"
            echo "$DISCOVERY" | grep -o '"[^"]*_endpoint":"[^"]*"' | head -3
        fi
    else
        echo "❌ Discovery endpoint returned invalid response"
        echo "   Response: $DISCOVERY"
        exit 1
    fi
else
    echo "❌ Cannot access OIDC Discovery endpoint"
    echo "   URL: $DISCOVERY_URL"
    echo "   Make sure Casdoor is configured for OIDC"
    exit 1
fi
echo ""

# Test 3: Check token endpoint configuration
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Test 3: Checking supported features..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if command -v jq &> /dev/null; then
    GRANT_TYPES=$(echo "$DISCOVERY" | jq -r '.grant_types_supported[]' 2>/dev/null || echo "")
    RESPONSE_TYPES=$(echo "$DISCOVERY" | jq -r '.response_types_supported[]' 2>/dev/null || echo "")

    if echo "$GRANT_TYPES" | grep -q "authorization_code"; then
        echo "✅ Authorization Code grant type supported"
    else
        echo "⚠️  Authorization Code grant type not found"
        echo "   Supported: $GRANT_TYPES"
    fi

    if echo "$DISCOVERY" | jq -e '.code_challenge_methods_supported' > /dev/null 2>&1; then
        PKCE_METHODS=$(echo "$DISCOVERY" | jq -r '.code_challenge_methods_supported[]')
        if echo "$PKCE_METHODS" | grep -q "S256"; then
            echo "✅ PKCE (S256) is supported"
        else
            echo "⚠️  PKCE S256 not found. Supported: $PKCE_METHODS"
        fi
    else
        echo "⚠️  PKCE support not advertised (might still work)"
    fi

    if echo "$RESPONSE_TYPES" | grep -q "code"; then
        echo "✅ Response type 'code' supported"
    else
        echo "❌ Response type 'code' not supported"
        echo "   Supported: $RESPONSE_TYPES"
    fi
else
    echo "⚠️  Install jq for detailed feature checking: brew install jq"
fi
echo ""

# Test 4: Check if application exists (basic test)
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Test 4: Testing client credentials..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# We can't fully test credentials without actually doing OAuth flow
# But we can check if the token endpoint is accessible
TOKEN_ENDPOINT=$(echo "$DISCOVERY" | grep -o '"token_endpoint":"[^"]*"' | cut -d'"' -f4)
echo "   Token endpoint: $TOKEN_ENDPOINT"

if curl -s -X POST "$TOKEN_ENDPOINT" \
    -H "Content-Type: application/x-www-form-urlencoded" \
    -d "grant_type=authorization_code" \
    -d "code=invalid_code_for_testing" \
    -d "client_id=$OIDC_CLIENT_ID" \
    -d "client_secret=$OIDC_CLIENT_SECRET" \
    -d "redirect_uri=$OIDC_REDIRECT_URI" > /tmp/token_test.txt 2>&1; then

    if grep -q "invalid_grant\|invalid_client\|unauthorized_client" /tmp/token_test.txt; then
        if grep -q "invalid_grant" /tmp/token_test.txt; then
            echo "✅ Client credentials appear valid (invalid_grant error is expected)"
        elif grep -q "invalid_client\|unauthorized_client" /tmp/token_test.txt; then
            echo "❌ Client credentials appear INVALID"
            echo "   Response: $(cat /tmp/token_test.txt)"
            echo ""
            echo "   Action needed:"
            echo "   1. Check OIDC_CLIENT_ID in .env"
            echo "   2. Check OIDC_CLIENT_SECRET in .env"
            echo "   3. Verify application exists in Casdoor"
        fi
    else
        echo "⚠️  Unexpected response from token endpoint"
        echo "   Response: $(cat /tmp/token_test.txt | head -3)"
    fi
else
    echo "⚠️  Could not connect to token endpoint"
fi
echo ""

# Test 5: Casdoor application checklist
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Test 5: Casdoor Application Configuration Checklist"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

echo ""
echo "Please verify in your Casdoor admin panel:"
echo ""
echo "  [ ] Application exists with name matching your setup"
echo "  [ ] Client ID matches: $OIDC_CLIENT_ID"
echo "  [ ] Client Secret is correctly set"
echo "  [ ] Redirect URLs includes: $OIDC_REDIRECT_URI"
echo "  [ ] Grant types includes: authorization_code"
echo "  [ ] Response types includes: code"
echo "  [ ] Token format: JWT"
echo "  [ ] PKCE: Enabled (recommended)"
echo "  [ ] Scopes include: openid, profile, email"
echo ""

# Final summary
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Summary"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Next steps:"
echo "  1. Verify all checklist items above in Casdoor"
echo "  2. Make sure redirect URI exactly matches"
echo "  3. Start the server: cargo run"
echo "  4. Try logging in: http://localhost:3000/login"
echo "  5. Check server logs for detailed error messages"
echo ""
echo "If you see 'Failed to parse server response':"
echo "  • Check that token format is set to JWT in Casdoor"
echo "  • Verify the application is properly configured"
echo "  • Make sure you're using the correct organization"
echo ""
echo "For more help, see: OIDC_TROUBLESHOOTING.md"
echo ""

# Cleanup
rm -f /tmp/token_test.txt
