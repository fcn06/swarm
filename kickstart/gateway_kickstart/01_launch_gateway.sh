#!/usr/bin/env bash

# ==============================================================================
# Swarm Standalone Gateway Server Launch Script
# Uses configuration from gateway_kickstart/config_files/
# ==============================================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SWARM_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
CONFIG_FILE="$SCRIPT_DIR/config_files/gateway_config.toml"

cd "$SWARM_ROOT"

PORT="${PORT:-8080}"
BIND_ADDRESS="${BIND_ADDRESS:-0.0.0.0:$PORT}"
LOG_LEVEL="${LOG_LEVEL:-info}"

# Auto-load .env from SWARM_ROOT or WORKSPACE_ROOT if present
WORKSPACE_ROOT="$(cd "$SWARM_ROOT/.." && pwd)"
if [ -f "$SWARM_ROOT/.env" ]; then
    # shellcheck disable=SC1091
    set -a
    source "$SWARM_ROOT/.env"
    set +a
elif [ -f "$WORKSPACE_ROOT/.env" ]; then
    # shellcheck disable=SC1091
    set -a
    source "$WORKSPACE_ROOT/.env"
    set +a
fi

echo "=========================================================================="
echo "          🌐 fcn06/swarm Standalone Gateway Server Launch                "
echo "=========================================================================="
echo " Config File : kickstart/gateway_kickstart/config_files/gateway_config.toml"
echo " Bind Address: $BIND_ADDRESS"
echo " Log Level   : $LOG_LEVEL"
echo "=========================================================================="

echo $'\n'
echo "--------------------------------------------------------------------------"
echo " 1. Checking LLM Provider Configuration"
echo "--------------------------------------------------------------------------"

# Check if local Ollama daemon is active
OLLAMA_ACTIVE=false
if curl -s --connect-timeout 1 http://localhost:11434/api/tags >/dev/null 2>&1; then
    OLLAMA_ACTIVE=true
fi

echo "Provider Status:"
echo " • GROQ_API_KEY   : ${GROQ_API_KEY:+[SET - Groq active]}${GROQ_API_KEY:-[NOT SET]}"
echo " • GEMINI_API_KEY : ${GEMINI_API_KEY:+[SET - Gemini active]}${GEMINI_API_KEY:-[NOT SET]}"
echo " • OPENAI_API_KEY : ${OPENAI_API_KEY:+[SET - OpenAI active]}${OPENAI_API_KEY:-[NOT SET]}"
echo " • SWARM_LLM_URL  : ${SWARM_LLM_URL:+[SET - $SWARM_LLM_URL]}${SWARM_LLM_URL:-[NOT SET]}"
if [ "$OLLAMA_ACTIVE" = true ]; then
    echo " • Local Ollama   : [DETECTED at http://localhost:11434]"
else
    echo " • Local Ollama   : [NOT DETECTED on port 11434]"
fi

# If no provider is configured, present interactive choice
if [ -z "$GROQ_API_KEY" ] && [ -z "$GEMINI_API_KEY" ] && [ -z "$OPENAI_API_KEY" ] && [ -z "$SWARM_LLM_URL" ]; then
    echo $'\n'
    echo "No LLM provider keys or custom endpoints are currently active."
    echo "Select an option below:"
    echo "  1) Groq (Cloud - ultra fast Llama 3.3, GPT-OSS)"
    echo "  2) Google Gemini (Cloud - Gemini 2.0 Flash / Pro)"
    echo "  3) OpenAI (Cloud - GPT-4o, GPT-4o-mini)"
    echo "  4) Local Ollama (http://localhost:11434)"
    echo "  5) Local Echo Mode (No LLM, echoes requests for testing)"
    echo ""
    read -p "Choose provider [1-5] (default 1): " PROVIDER_CHOICE
    PROVIDER_CHOICE="${PROVIDER_CHOICE:-1}"

    case "$PROVIDER_CHOICE" in
        1)
            read -p "Enter your GROQ API key: " USER_KEY
            if [ -n "$USER_KEY" ]; then
                export GROQ_API_KEY="$USER_KEY"
                echo "GROQ_API_KEY=$USER_KEY" >> "$SWARM_ROOT/.env"
                echo "✔ GROQ_API_KEY set and saved to .env."
            fi
            ;;
        2)
            read -p "Enter your Google GEMINI API key: " USER_KEY
            if [ -n "$USER_KEY" ]; then
                export GEMINI_API_KEY="$USER_KEY"
                echo "GEMINI_API_KEY=$USER_KEY" >> "$SWARM_ROOT/.env"
                echo "✔ GEMINI_API_KEY set and saved to .env."
            fi
            ;;
        3)
            read -p "Enter your OPENAI API key: " USER_KEY
            if [ -n "$USER_KEY" ]; then
                export OPENAI_API_KEY="$USER_KEY"
                echo "OPENAI_API_KEY=$USER_KEY" >> "$SWARM_ROOT/.env"
                echo "✔ OPENAI_API_KEY set and saved to .env."
            fi
            ;;
        4)
            read -p "Enter Ollama URL [http://localhost:11434/v1/chat/completions]: " USER_URL
            USER_URL="${USER_URL:-http://localhost:11434/v1/chat/completions}"
            export SWARM_LLM_URL="$USER_URL"
            export OPENAI_API_KEY="ollama"
            cat <<EOF >> "$SWARM_ROOT/.env"
SWARM_LLM_URL=$USER_URL
OPENAI_API_KEY=ollama
EOF
            echo "✔ Configured for local Ollama ($USER_URL) and saved to .env."
            ;;
        5)
            echo "✔ Operating in Echo mode."
            ;;
        *)
            echo "✔ Defaulting to Echo mode."
            ;;
    esac
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
echo " • Config Directory:       kickstart/gateway_kickstart/config_files/"
echo "=========================================================================="
echo $'\n'
echo "👉 Test Chat Completions: ./kickstart/gateway_kickstart/02_test_chat_completions.sh"
echo "👉 Test Open Responses:   ./kickstart/gateway_kickstart/03_test_open_responses.sh"
echo "👉 Stop Gateway:          ./kickstart/gateway_kickstart/04_terminate_gateway.sh"
