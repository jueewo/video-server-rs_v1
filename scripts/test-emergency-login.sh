#!/bin/bash

# Emergency Login Test Script
# This script tests the emergency login feature with various scenarios

set -e

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}================================================${NC}"
echo -e "${BLUE}   Emergency Login Feature Test${NC}"
echo -e "${BLUE}================================================${NC}"
echo ""

BASE_URL="http://localhost:3000"

# Function to check if server is running
check_server() {
    if ! curl -s -o /dev/null -w "%{http_code}" "${BASE_URL}/health" > /dev/null 2>&1; then
        echo -e "${RED}✗ Server is not running at ${BASE_URL}${NC}"
        echo -e "${YELLOW}  Please start the server first: cargo run${NC}"
        exit 1
    fi
    echo -e "${GREEN}✓ Server is running${NC}"
    echo ""
}

# Test 1: Check if emergency login is enabled
test_emergency_enabled() {
    echo -e "${BLUE}Test 1: Check if emergency login route exists${NC}"

    HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" "${BASE_URL}/login/emergency")

    if [ "$HTTP_CODE" = "200" ]; then
        echo -e "${GREEN}✓ Emergency login is ENABLED (route exists)${NC}"
        echo -e "${YELLOW}  Note: This should be disabled in production!${NC}"
        return 0
    elif [ "$HTTP_CODE" = "404" ]; then
        echo -e "${YELLOW}⚠ Emergency login is DISABLED (route not found)${NC}"
        echo -e "${YELLOW}  To enable: Set ENABLE_EMERGENCY_LOGIN=true in .env and restart${NC}"
        return 1
    else
        echo -e "${RED}✗ Unexpected response code: ${HTTP_CODE}${NC}"
        return 1
    fi
    echo ""
}

# Test 2: Check if login page shows emergency button
test_login_page() {
    echo -e "${BLUE}Test 2: Check login page for emergency button${NC}"

    PAGE_CONTENT=$(curl -s "${BASE_URL}/login")

    if echo "$PAGE_CONTENT" | grep -q "Emergency Login"; then
        echo -e "${GREEN}✓ Emergency login button visible on login page${NC}"
    else
        echo -e "${YELLOW}⚠ Emergency login button not visible${NC}"
        echo -e "${YELLOW}  This is expected if ENABLE_EMERGENCY_LOGIN=false${NC}"
    fi
    echo ""
}

# Test 3: Check emergency login form
test_emergency_form() {
    echo -e "${BLUE}Test 3: Check emergency login form${NC}"

    FORM_CONTENT=$(curl -s "${BASE_URL}/login/emergency")

    if echo "$FORM_CONTENT" | grep -q "username"; then
        echo -e "${GREEN}✓ Emergency login form contains username field${NC}"
    else
        echo -e "${RED}✗ Username field not found in form${NC}"
    fi

    if echo "$FORM_CONTENT" | grep -q "password"; then
        echo -e "${GREEN}✓ Emergency login form contains password field${NC}"
    else
        echo -e "${RED}✗ Password field not found in form${NC}"
    fi

    if echo "$FORM_CONTENT" | grep -q "POST"; then
        echo -e "${GREEN}✓ Form uses POST method${NC}"
    else
        echo -e "${RED}✗ Form does not use POST method${NC}"
    fi
    echo ""
}

# Test 4: Test invalid credentials
test_invalid_credentials() {
    echo -e "${BLUE}Test 4: Test with invalid credentials${NC}"

    RESPONSE=$(curl -s -X POST "${BASE_URL}/login/emergency/auth" \
        -H "Content-Type: application/x-www-form-urlencoded" \
        -d "username=invalid&password=wrong")

    if echo "$RESPONSE" | grep -q "Login Failed\|Invalid credentials"; then
        echo -e "${GREEN}✓ Invalid credentials rejected properly${NC}"
    else
        echo -e "${RED}✗ Unexpected response to invalid credentials${NC}"
    fi
    echo ""
}

# Test 5: Configuration check
test_configuration() {
    echo -e "${BLUE}Test 5: Check .env configuration${NC}"

    if [ -f ".env" ]; then
        echo -e "${GREEN}✓ .env file exists${NC}"

        if grep -q "ENABLE_EMERGENCY_LOGIN" .env; then
            ENABLED=$(grep "ENABLE_EMERGENCY_LOGIN" .env | cut -d'=' -f2)
            echo -e "${GREEN}✓ ENABLE_EMERGENCY_LOGIN is set to: ${ENABLED}${NC}"
        else
            echo -e "${YELLOW}⚠ ENABLE_EMERGENCY_LOGIN not found in .env${NC}"
        fi

        if grep -q "SU_USER" .env; then
            echo -e "${GREEN}✓ SU_USER is configured${NC}"
        else
            echo -e "${YELLOW}⚠ SU_USER not found in .env${NC}"
        fi

        if grep -q "SU_PWD" .env; then
            echo -e "${GREEN}✓ SU_PWD is configured${NC}"
        else
            echo -e "${YELLOW}⚠ SU_PWD not found in .env${NC}"
        fi
    else
        echo -e "${YELLOW}⚠ .env file not found${NC}"
        echo -e "${YELLOW}  Copy .env.example to .env and configure${NC}"
    fi
    echo ""
}

# Main test execution
main() {
    check_server

    test_configuration

    if test_emergency_enabled; then
        test_login_page
        test_emergency_form
        test_invalid_credentials

        echo -e "${BLUE}================================================${NC}"
        echo -e "${GREEN}✓ Emergency login tests completed${NC}"
        echo -e "${BLUE}================================================${NC}"
        echo ""
        echo -e "${YELLOW}Manual Testing:${NC}"
        echo -e "1. Visit: ${BASE_URL}/login"
        echo -e "2. Click 'Emergency Login' button"
        echo -e "3. Enter valid SU_USER and SU_PWD credentials"
        echo -e "4. Verify successful login"
        echo ""
        echo -e "${YELLOW}Security Reminder:${NC}"
        echo -e "⚠  Emergency login is currently ENABLED"
        echo -e "   This should be DISABLED in production!"
        echo -e "   Set ENABLE_EMERGENCY_LOGIN=false in .env"
    else
        echo -e "${BLUE}================================================${NC}"
        echo -e "${YELLOW}⚠ Emergency login is disabled${NC}"
        echo -e "${BLUE}================================================${NC}"
        echo ""
        echo -e "To test emergency login functionality:"
        echo -e "1. Edit .env and set: ENABLE_EMERGENCY_LOGIN=true"
        echo -e "2. Set SU_USER and SU_PWD to test credentials"
        echo -e "3. Restart the server: cargo run"
        echo -e "4. Run this test script again"
        echo ""
        echo -e "${GREEN}✓ This is the correct state for production!${NC}"
    fi
}

# Run tests
main
