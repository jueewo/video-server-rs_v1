#!/bin/bash

SERVER="http://localhost:3000"

echo "Fetching media and checking for control characters..."

# Get your API key from environment or prompt
if [ -z "$MEDIA_API_KEY" ]; then
    read -sp "Enter your API key: " API_KEY
    echo ""
else
    API_KEY="$MEDIA_API_KEY"
fi

# Fetch the JSON
curl -s -H "Authorization: Bearer $API_KEY" "$SERVER/api/media" > /tmp/media_debug.json

echo ""
echo "=== Checking for control characters in the response ==="
# Look for control characters and show context
od -A x -t x1z -v /tmp/media_debug.json | grep -E "0[0-9a-f] " | head -20

echo ""
echo "=== Trying to identify the problematic item ==="
# Try to parse each item individually
python3 << 'PYTHON'
import json
import sys

try:
    with open('/tmp/media_debug.json', 'r') as f:
        content = f.read()
    
    # Check for control characters
    for i, char in enumerate(content):
        if ord(char) < 32 and char not in '\n\r\t':
            print(f"Found control character at position {i}: {repr(char)} (0x{ord(char):02x})")
            # Show context
            start = max(0, i - 50)
            end = min(len(content), i + 50)
            print(f"Context: ...{repr(content[start:end])}...")
            print()
    
    # Try to parse JSON
    data = json.loads(content)
    print(f"\nTotal items: {len(data.get('items', []))}")
    
    # Check each item for control characters in fields
    for idx, item in enumerate(data.get('items', [])):
        item_data = item.get('data', {})
        for field in ['title', 'description', 'filename', 'slug']:
            value = item_data.get(field, '')
            if isinstance(value, str):
                for char in value:
                    if ord(char) < 32 and char not in '\n\r\t':
                        print(f"Item #{idx} (ID: {item_data.get('id')}): Field '{field}' contains control character: {repr(char)}")
                        print(f"  Value: {repr(value)}")

except Exception as e:
    print(f"Error: {e}")
    import traceback
    traceback.print_exc()
PYTHON

echo ""
echo "=== Raw JSON saved to /tmp/media_debug.json ==="
echo "You can inspect it with: cat -A /tmp/media_debug.json | less"
