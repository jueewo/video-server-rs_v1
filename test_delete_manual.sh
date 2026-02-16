#!/bin/bash
read -sp "Enter API key: " API_KEY
echo ""

echo "Testing DELETE for document ID 33..."
curl -s -X DELETE "http://localhost:3000/api/documents/33" \
    -H "Authorization: Bearer $API_KEY" \
    -H "Content-Type: application/json" | jq .

echo ""
echo "Checking if file still exists..."
ls -la storage/vaults/vault-90b0d507/documents/ai-b2b-solutions.md 2>&1

echo ""
echo "=== Server logs (last 30 lines) ==="
tail -30 /tmp/server.log | grep -A10 -B10 "Deleting document\|delete.*file"
