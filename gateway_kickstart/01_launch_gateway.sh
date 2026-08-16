#!/usr/bin/env bash

# ==============================================================================
# Swarm Standalone Gateway Server Launch Script
# Uses configuration from gateway_kickstart/config_files/
# ==============================================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SWARM_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CONFIG_FILE="$SCRIPT_DIR/config_files/gateway_config.toml"

cd "$SWARM_ROOT"

PORT="${PORT:-8080}"
BIND_ADDRESS="${BIND_ADDRESS:-0.0.0.0:$PORT}"
LOG_LEVEL="${LOG_LEVEL:-info}"

echo "=========================================================================="
echo "          🌐 fcn06/swarm Standalone Gateway Server Launch                "
echo "=========================================================================="
echo " Config File : gateway_kickstart/config_files/gateway_config.toml"
echo " Bind Address: $BIND_ADDRESS"
echo " Log Level   : $LOG_LEVEL"
echo "=========================================================================="

echo $'\n'
echo "--------------------------------------------------------------------------"
echo " 1. Checking LLM Provider API Keys"
echo "--------------------------------------------------------------------------"

echo "Provider Keys Status:"
echo " • GROQ_API_KEY   : ${GROQ_API_KEY:+[SET]}${GROQ_API_KEY:-[NOT SET - needed for Groq models]}"
echo " • GEMINI_API_KEY : ${GEMINI_API_KEY:+[SET]}${GEMINI_API_KEY:-[NOT SET - needed for Gemini models]}"
echo " • OPENAI_API_KEY : ${OPENAI_API_KEY:+[SET]}${OPENAI_API_KEY:-[NOT SET - needed for OpenAI models]}"

if [ -z "$GROQ_API_KEY" ] && [ -z "$GEMINI_API_KEY" ] && [ -z "$OPENAI_API_KEY" ]; then
    echo $'\n'
    echo "⚠️  WARNING: No LLM API keys were detected in your environment."
    echo "   Without an API key, the gateway will operate in local Echo mode."
    read -p "Enter your GROQ API key (or press enter to skip): " USER_KEY
    if [ -n "$USER_KEY" ]; then
        export GROQ_API_KEY="$USER_KEY"
        echo "✔ GROQ_API_KEY set."
    fi
fi

echo $'\n'
echo "--------------------------------------------------------------------------"
echo " 2. Building Gateway Server (Release Mode)"
echo "--------------------------------------------------------------------------"

cargo build --release --bin swarm_server

echo "✔ Binary compiled successfully."

echo $'\n'
echo "--------------------------------------------------------------------------"
echo " 3. Launching Gateway Server on $BIND_ADDRESS"
echo "--------------------------------------------------------------------------"

# Terminate any existing swarm_server process
pkill -f "target/release/swarm_server" || true
sleep 1

./target/release/swarm_server --config-file "$CONFIG_FILE" --bind-address "$BIND_ADDRESS" --log-level "$LOG_LEVEL" &
GATEWAY_PID=$!
sleep 2

echo $'\n'
echo "=========================================================================="
echo "          🎉 Swarm Gateway Server is RUNNING (PID: $GATEWAY_PID)!        "
echo "=========================================================================="
echo " • Open Responses Route:   POST http://127.0.0.1:$PORT/v1/responses"
echo " • Chat Completions Route: POST http://127.0.0.1:$PORT/v1/chat/completions"
echo " • Config Directory:       gateway_kickstart/config_files/"
echo "=========================================================================="
echo $'\n'
echo "👉 Test Chat Completions: ./gateway_kickstart/02_test_chat_completions.sh"
echo "👉 Test Open Responses:   ./gateway_kickstart/03_test_open_responses.sh"
echo "👉 Stop Gateway:          ./gateway_kickstart/04_terminate_gateway.sh"
