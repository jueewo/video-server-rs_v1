#!/bin/sh

# Simulate what the script does
SERVER="http://localhost:3000"

printf "Testing...\n"

# Get the JSON
MEDIA_JSON=$(curl -s -H "Authorization: Bearer test" "$SERVER/api/media")

# Try to parse it
echo "$MEDIA_JSON" | jq -r '.items[] | [.type, .data.id, .data.slug, .data.title] | @tsv'
