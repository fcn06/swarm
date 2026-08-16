#!/usr/bin/env bash

# ==============================================================================
# Gateway Test: Stateless POST /v1/chat/completions (OpenAI compatible)
# Uses template from gateway_kickstart/config_files/chat_completions_request.json
# ==============================================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SWARM_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
CONFIG_DIR="$SCRIPT_DIR/config_files"

cd "$SWARM_ROOT"

PORT="${PORT:-8080}"
USER_QUERY="${1:-}"
MODEL="${2:-groq/llama-3.3-70b-versatile}"

echo "=========================================================================="
echo "    🌐 Gateway Test: POST /v1/chat/completions                            "
echo "=========================================================================="
echo " URL    : http://localhost:$PORT/v1/chat/completions"
echo " Config : kickstart/gateway_kickstart/config_files/chat_completions_request.json"
echo " Model  : $MODEL"
if [ -n "$USER_QUERY" ]; then
    echo " Query  : \"$USER_QUERY\""
fi
echo "=========================================================================="
echo ""

if [ -n "$USER_QUERY" ]; then
    PAYLOAD=$(jq -n \
      --arg model "$MODEL" \
      --arg content "$USER_QUERY" \
      '{model: $model, messages: [{role: "system", content: "You are a helpful assistant."}, {role: "user", content: $content}], stream: false}')
else
    PAYLOAD=$(cat "$CONFIG_DIR/chat_completions_request.json")
fi

RESPONSE=$(curl -s -X POST "http://localhost:$PORT/v1/chat/completions" \
  -H "Content-Type: application/json" \
  -d "$PAYLOAD")

echo "Response from Gateway:"
if command -v jq &> /dev/null; then
    echo "$RESPONSE" | jq .
else
    echo "$RESPONSE"
fi

echo ""
echo "=========================================================================="
echo "✔ Chat completions test completed!"
echo "=========================================================================="
