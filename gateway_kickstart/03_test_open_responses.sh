#!/usr/bin/env bash

# ==============================================================================
# Gateway Test: Stateful POST /v1/responses (Open Responses standard)
# Uses templates from gateway_kickstart/config_files/
# ==============================================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SWARM_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CONFIG_DIR="$SCRIPT_DIR/config_files"

cd "$SWARM_ROOT"

PORT="${PORT:-8080}"
MODEL="${1:-groq/llama-3.3-70b-versatile}"

echo "=========================================================================="
echo "    🌐 Gateway Test: POST /v1/responses (Multi-Turn Stateful Session)    "
echo "=========================================================================="
echo " URL    : http://localhost:$PORT/v1/responses"
echo " Config : gateway_kickstart/config_files/"
echo " Model  : $MODEL"
echo "=========================================================================="
echo ""

echo "--------------------------------------------------------------------------"
echo " Turn 1: Initial query from config_files/open_responses_request.json"
echo " Input  : \"My favorite city is Boston.\""
echo "--------------------------------------------------------------------------"

TURN1_PAYLOAD=$(cat "$CONFIG_DIR/open_responses_request.json")
if [ -n "$MODEL" ]; then
    TURN1_PAYLOAD=$(echo "$TURN1_PAYLOAD" | jq --arg model "$MODEL" '.model = $model')
fi

TURN1_RESP=$(curl -s -X POST "http://localhost:$PORT/v1/responses" \
  -H "Content-Type: application/json" \
  -d "$TURN1_PAYLOAD")

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
    TURN2_PAYLOAD=$(cat "$CONFIG_DIR/open_responses_followup_request.json" | \
      jq --arg model "$MODEL" --arg pid "$PREV_ID" '.model = $model | .previous_response_id = $pid')
else
    TURN2_PAYLOAD=$(cat "$CONFIG_DIR/open_responses_followup_request.json" | \
      jq --arg model "$MODEL" '.model = $model | del(.previous_response_id)')
fi

TURN2_RESP=$(curl -s -X POST "http://localhost:$PORT/v1/responses" \
  -H "Content-Type: application/json" \
  -d "$TURN2_PAYLOAD")

if command -v jq &> /dev/null; then
    echo "$TURN2_RESP" | jq .
else
    echo "$TURN2_RESP"
fi

echo ""
echo "=========================================================================="
echo "✔ Open Responses stateful test completed!"
echo "=========================================================================="
