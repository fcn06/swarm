#!/usr/bin/env bash

# ==============================================================================
# Gateway Test: Stateful POST /v1/responses (Open Responses standard)
# ==============================================================================

set -e

PORT="${PORT:-8080}"
MODEL="${1:-groq/llama-3.3-70b-versatile}"

echo "=========================================================================="
echo "    🌐 Gateway Test: POST /v1/responses (Multi-Turn Stateful Session)    "
echo "=========================================================================="
echo " URL   : http://localhost:$PORT/v1/responses"
echo " Model : $MODEL"
echo "=========================================================================="
echo ""

echo "--------------------------------------------------------------------------"
echo " Turn 1: Initial user query"
echo " Input  : \"My favorite city is Boston.\""
echo "--------------------------------------------------------------------------"

TURN1_RESP=$(curl -s -X POST "http://localhost:$PORT/v1/responses" \
  -H "Content-Type: application/json" \
  -d "{
    \"model\": \"$MODEL\",
    \"input\": \"My favorite city is Boston.\",
    \"stream\": false
  }")

if command -v jq &> /dev/null; then
    echo "$TURN1_RESP" | jq .
else
    echo "$TURN1_RESP"
fi

# Extract the response ID to chain session state in Turn 2
if command -v jq &> /dev/null; then
    PREV_ID=$(echo "$TURN1_RESP" | jq -r '.id // empty')
else
    PREV_ID=""
fi

echo ""
echo "--------------------------------------------------------------------------"
echo " Turn 2: Follow-up query chaining with previous_response_id: $PREV_ID"
echo " Input  : \"What city did I say was my favorite?\""
echo "--------------------------------------------------------------------------"

if [ -n "$PREV_ID" ] && [ "$PREV_ID" != "null" ]; then
    TURN2_RESP=$(curl -s -X POST "http://localhost:$PORT/v1/responses" \
      -H "Content-Type: application/json" \
      -d "{
        \"model\": \"$MODEL\",
        \"input\": \"What city did I say was my favorite?\",
        \"previous_response_id\": \"$PREV_ID\",
        \"stream\": false
      }")
else
    TURN2_RESP=$(curl -s -X POST "http://localhost:$PORT/v1/responses" \
      -H "Content-Type: application/json" \
      -d "{
        \"model\": \"$MODEL\",
        \"input\": \"What is the weather like in Boston ?\",
        \"stream\": false
      }")
fi

if command -v jq &> /dev/null; then
    echo "$TURN2_RESP" | jq .
else
    echo "$TURN2_RESP"
fi

echo ""
echo "=========================================================================="
echo "✔ Open Responses stateful test completed!"
echo "=========================================================================="
