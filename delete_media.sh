#!/bin/bash
#
# Interactive Media Deletion Script
# Authenticates with the server and allows deletion of media items
#

set -e

SERVER="http://localhost:3000"
COOKIE_FILE="/tmp/media_cookies.txt"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🗑️  Media Deletion Tool${NC}"
echo "================================"
echo ""

# Step 1: Authentication
echo -e "${YELLOW}🔐 Step 1: Authentication${NC}"
echo "Choose authentication method:"
echo "  1. API Key (recommended)"
echo "  2. Emergency Login (username/password)"
echo ""
read -p "Selection [1]: " AUTH_METHOD
AUTH_METHOD=${AUTH_METHOD:-1}

# Authentication headers variable
AUTH_HEADERS=""

if [ "$AUTH_METHOD" = "1" ]; then
    # API Key authentication
    if [ -n "$MEDIA_API_KEY" ]; then
        echo "Using API key from MEDIA_API_KEY environment variable"
        API_KEY="$MEDIA_API_KEY"
    else
        read -sp "Enter your API key: " API_KEY
        echo ""
    fi

    if [ -z "$API_KEY" ]; then
        echo -e "${RED}✗ No API key provided${NC}"
        exit 1
    fi

    # Test API key
    echo -n "Validating API key... "
    TEST_RESPONSE=$(curl -s -H "Authorization: Bearer $API_KEY" "$SERVER/api/media" -w "\n%{http_code}")
    HTTP_CODE=$(echo "$TEST_RESPONSE" | tail -n 1)

    if [ "$HTTP_CODE" = "200" ]; then
        echo -e "${GREEN}✓ API key valid${NC}"
        AUTH_HEADERS="-H \"Authorization: Bearer $API_KEY\""
    else
        echo -e "${RED}✗ API key invalid (HTTP $HTTP_CODE)${NC}"
        exit 1
    fi
else
    # Emergency login (legacy method)
    read -p "Username [admin]: " USERNAME
    USERNAME=${USERNAME:-admin}

    read -sp "Password [testpass123]: " PASSWORD
    PASSWORD=${PASSWORD:-testpass123}
    echo ""

    echo -n "Logging in... "
    LOGIN_RESPONSE=$(curl -s -c "$COOKIE_FILE" -X POST "$SERVER/login/emergency/auth" \
      -H "Content-Type: application/x-www-form-urlencoded" \
      -d "username=$USERNAME&password=$PASSWORD" \
      -w "\n%{http_code}")

    HTTP_CODE=$(echo "$LOGIN_RESPONSE" | tail -n 1)

    if [ "$HTTP_CODE" = "200" ] || [ "$HTTP_CODE" = "303" ]; then
        echo -e "${GREEN}✓ Login successful${NC}"
        AUTH_HEADERS="-b $COOKIE_FILE"
    else
        echo -e "${RED}✗ Login failed (HTTP $HTTP_CODE)${NC}"
        rm -f "$COOKIE_FILE"
        exit 1
    fi
fi

echo ""

# Step 2: Fetch media items
echo -e "${YELLOW}📋 Step 2: Fetching your media items${NC}"

# Get all media from unified API (using appropriate auth method)
MEDIA_JSON=$(eval curl -s $AUTH_HEADERS "$SERVER/api/media")

# Parse media items
echo ""
echo "Your Media (All Types):"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Store IDs and info in arrays
IDS=()
SLUGS=()
TITLES=()
TYPES=()

# Check if jq is available for better JSON parsing
if command -v jq &> /dev/null; then
    # Use jq for robust parsing
    # Store in temporary variable to avoid process substitution issues
    JQ_OUTPUT=$(echo "$MEDIA_JSON" | jq -r '.items[] | [.type, .data.id, .data.slug, .data.title] | @tsv')
    while IFS=$'\t' read -r type id slug title; do
        TYPES+=("$type")
        IDS+=("$id")
        SLUGS+=("$slug")
        TITLES+=("$title")
    done <<< "$JQ_OUTPUT"
else
    # Fallback to basic regex parsing
    # Extract each item block
    echo "$MEDIA_JSON" | grep -o '"type":"[^"]*","data":{"id":[^}]*}' | while read -r item; do
        if [[ $item =~ \"type\":\"([^\"]+)\" ]]; then
            TYPE="${BASH_REMATCH[1]}"
            TYPES+=("$TYPE")
        fi
        if [[ $item =~ \"id\":([0-9]+) ]]; then
            ID="${BASH_REMATCH[1]}"
            IDS+=("$ID")
        fi
        if [[ $item =~ \"slug\":\"([^\"]+)\" ]]; then
            SLUG="${BASH_REMATCH[1]}"
            SLUGS+=("$SLUG")
        fi
        if [[ $item =~ \"title\":\"([^\"]+)\" ]]; then
            TITLE="${BASH_REMATCH[1]}"
            TITLES+=("$TITLE")
        fi
    done
fi

# Display items
if [ ${#IDS[@]} -eq 0 ]; then
    echo -e "${YELLOW}No media found or unable to fetch list.${NC}"
    echo ""
    echo "You can still delete by ID manually. Check the database:"
    echo "  sqlite3 media.db 'SELECT id, slug, title, media_type FROM media_items;'"
    rm -f "$COOKIE_FILE"
    exit 0
fi

for i in "${!IDS[@]}"; do
    # Get type icon
    case "${TYPES[$i]}" in
        "Image") ICON="🖼️ " ;;
        "Video") ICON="🎥" ;;
        "Document") ICON="📄" ;;
        *) ICON="📦" ;;
    esac

    printf "${BLUE}%2d.${NC} %s %-8s ID: %-4s Slug: %-25s Title: %s\n" \
        "$((i+1))" "$ICON" "${TYPES[$i]}" "${IDS[$i]}" "${SLUGS[$i]}" "${TITLES[$i]}"
done

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Step 3: Select items to delete
echo -e "${YELLOW}🎯 Step 3: Select items to delete${NC}"
echo "Enter item numbers to delete (comma-separated, e.g., 1,3,5)"
echo "Or enter 'all' to delete all items"
echo "Or enter 'q' to quit"
echo ""
read -p "Selection: " SELECTION

if [ "$SELECTION" = "q" ]; then
    echo "Cancelled."
    rm -f "$COOKIE_FILE"
    exit 0
fi

# Parse selection
TO_DELETE=()
if [ "$SELECTION" = "all" ]; then
    TO_DELETE=("${IDS[@]}")
else
    IFS=',' read -ra INDICES <<< "$SELECTION"
    for idx in "${INDICES[@]}"; do
        # Trim whitespace
        idx=$(echo "$idx" | xargs)
        # Convert to array index (1-based to 0-based)
        arr_idx=$((idx - 1))
        if [ $arr_idx -ge 0 ] && [ $arr_idx -lt ${#IDS[@]} ]; then
            TO_DELETE+=("${IDS[$arr_idx]}")
        else
            echo -e "${RED}Invalid selection: $idx${NC}"
        fi
    done
fi

if [ ${#TO_DELETE[@]} -eq 0 ]; then
    echo "No items selected."
    rm -f "$COOKIE_FILE"
    exit 0
fi

echo ""
echo -e "${RED}⚠️  WARNING: You are about to delete ${#TO_DELETE[@]} item(s)${NC}"
echo ""
read -p "Are you sure? (yes/no): " CONFIRM

if [ "$CONFIRM" != "yes" ]; then
    echo "Cancelled."
    rm -f "$COOKIE_FILE"
    exit 0
fi

# Step 4: Delete items
echo ""
echo -e "${YELLOW}🗑️  Step 4: Deleting items${NC}"

SUCCESS_COUNT=0
FAIL_COUNT=0

for idx in "${!TO_DELETE[@]}"; do
    id="${TO_DELETE[$idx]}"

    # Find the index in original arrays to get type
    original_idx=-1
    for i in "${!IDS[@]}"; do
        if [ "${IDS[$i]}" = "$id" ]; then
            original_idx=$i
            break
        fi
    done

    if [ $original_idx -ge 0 ]; then
        type="${TYPES[$original_idx]}"
        slug="${SLUGS[$original_idx]}"
        echo -n "Deleting $type ID $id ($slug)... "

        # Determine the correct API endpoint based on type
        case "$type" in
            "Image")
                ENDPOINT="$SERVER/api/images/$id"
                ;;
            "Video")
                ENDPOINT="$SERVER/api/videos/$id"
                ;;
            "Document")
                ENDPOINT="$SERVER/api/documents/$id"
                ;;
            *)
                echo -e "${RED}✗ Unknown type${NC}"
                ((FAIL_COUNT++))
                continue
                ;;
        esac

        RESPONSE=$(eval curl -s $AUTH_HEADERS -X DELETE "$ENDPOINT" \
            -H "Content-Type: application/json" \
            -w "\n%{http_code}")

        HTTP_CODE=$(echo "$RESPONSE" | tail -n 1)
        BODY=$(echo "$RESPONSE" | sed '$d')

        if [ "$HTTP_CODE" = "200" ]; then
            echo -e "${GREEN}✓ Deleted${NC}"
            ((SUCCESS_COUNT++))
        else
            echo -e "${RED}✗ Failed (HTTP $HTTP_CODE)${NC}"
            if [ ! -z "$BODY" ]; then
                echo "  Response: $BODY"
            fi
            ((FAIL_COUNT++))
        fi
    else
        echo -e "${RED}✗ Could not find type for ID $id${NC}"
        ((FAIL_COUNT++))
    fi
done

# Cleanup
rm -f "$COOKIE_FILE"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${GREEN}✓ Deleted: $SUCCESS_COUNT${NC}"
if [ $FAIL_COUNT -gt 0 ]; then
    echo -e "${RED}✗ Failed: $FAIL_COUNT${NC}"
fi
echo ""
echo -e "${BLUE}Done!${NC}"
