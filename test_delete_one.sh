#!/bin/bash
# Test deleting document ID 33

API_KEY="${MEDIA_API_KEY}"
if [ -z "$API_KEY" ]; then
    echo "Error: MEDIA_API_KEY not set"
    exit 1
fi

echo "Testing DELETE for document ID 33..."
curl -v -X DELETE "http://localhost:3000/api/documents/33" \
    -H "Authorization: Bearer $API_KEY" \
    -H "Content-Type: application/json"

echo ""
echo "Checking if file still exists..."
ls -la storage/vaults/vault-90b0d507/documents/ai-b2b-solutions.md 2>&1
