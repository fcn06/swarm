#!/usr/bin/env bash

# ==============================================================================
# Gateway Test: Stateless POST /v1/chat/completions (OpenAI compatible)
# ==============================================================================

set -e

PORT="${PORT:-8080}"
USER_QUERY="${1:-What is the weather like in Boston ?}"
MODEL="${2:-groq/llama-3.3-70b-versatile}"

echo "=========================================================================="
echo "    🌐 Gateway Test: POST /v1/chat/completions                            "
echo "=========================================================================="
echo " URL   : http://localhost:$PORT/v1/chat/completions"
echo " Model : $MODEL"
echo " Query : \"$USER_QUERY\""
echo "=========================================================================="
echo ""

RESPONSE=$(curl -s -X POST "http://localhost:$PORT/v1/chat/completions" \
  -H "Content-Type: application/json" \
  -d "{
    \"model\": \"$MODEL\",
    \"messages\": [
      {
        \"role\": \"system\",
        \"content\": \"You are a helpful assistant.\"
      },
      {
        \"role\": \"user\",
        \"content\": \"$USER_QUERY\"
      }
    ],
    \"stream\": false
  }")

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
