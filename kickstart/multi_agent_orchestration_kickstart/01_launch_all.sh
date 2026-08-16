#!/usr/bin/env bash

# ==============================================================================
# Swarm Multi-Agent & MCP Full Stack Launch Script
# Uses local config files in kickstart/config_files/
# ==============================================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SWARM_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
WORKSPACE_ROOT="$(cd "$SWARM_ROOT/.." && pwd)"
SWARM_SERVICES_ROOT="$WORKSPACE_ROOT/swarm_services"
CONFIG_DIR="$SCRIPT_DIR/config_files"

cd "$SWARM_ROOT"

# Auto-load .env from SWARM_ROOT or WORKSPACE_ROOT if present
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
echo "          🌐 fcn06/swarm Multi-Agent & MCP Tools Launch                  "
echo "=========================================================================="

echo $'\n'
echo "--------------------------------------------------------------------------"
echo " 1. Checking GROQ API Key Environment Variables"
echo "--------------------------------------------------------------------------"

if [ -n "$GROQ_API_KEY" ]; then
    echo "✔ Found GROQ_API_KEY in environment."
    export LLM_A2A_API_KEY="${LLM_A2A_API_KEY:-$GROQ_API_KEY}"
    export LLM_MCP_API_KEY="${LLM_MCP_API_KEY:-$GROQ_API_KEY}"
    export LLM_PLANNER_API_KEY="${LLM_PLANNER_API_KEY:-$GROQ_API_KEY}"
    export LLM_JUDGE_API_KEY="${LLM_JUDGE_API_KEY:-$GROQ_API_KEY}"
elif [ -n "$LLM_A2A_API_KEY" ]; then
    export GROQ_API_KEY="${GROQ_API_KEY:-$LLM_A2A_API_KEY}"
    export LLM_MCP_API_KEY="${LLM_MCP_API_KEY:-$LLM_A2A_API_KEY}"
    export LLM_PLANNER_API_KEY="${LLM_PLANNER_API_KEY:-$LLM_A2A_API_KEY}"
    export LLM_JUDGE_API_KEY="${LLM_JUDGE_API_KEY:-$LLM_A2A_API_KEY}"
fi

echo "Summary of GROQ API Key Variables:"
echo " • LLM_A2A_API_KEY    : ${LLM_A2A_API_KEY:+[SET]}${LLM_A2A_API_KEY:-[MISSING - for Domain / Basic Agent with MCP tools]}"
echo " • LLM_MCP_API_KEY    : ${LLM_MCP_API_KEY:+[SET]}${LLM_MCP_API_KEY:-[MISSING - for MCP Runtime Tool Loop]}"
echo " • LLM_PLANNER_API_KEY: ${LLM_PLANNER_API_KEY:+[SET]}${LLM_PLANNER_API_KEY:-[MISSING - for Planner Agent Orchestrator]}"
echo " • LLM_JUDGE_API_KEY  : ${LLM_JUDGE_API_KEY:+[SET]}${LLM_JUDGE_API_KEY:-[MISSING - for Evaluation / Judge Service]}"

if [ -z "$LLM_A2A_API_KEY" ]; then
    echo $'\n'
    echo "⚠️  WARNING: No Groq API Key was detected in your environment."
    read -p "Enter your GROQ API key now: " USER_KEY
    if [ -n "$USER_KEY" ]; then
        export GROQ_API_KEY="$USER_KEY"
        export LLM_A2A_API_KEY="$USER_KEY"
        export LLM_MCP_API_KEY="$USER_KEY"
        export LLM_PLANNER_API_KEY="$USER_KEY"
        export LLM_JUDGE_API_KEY="$USER_KEY"
        
        # Persist to .env for future runs
        cat <<EOF > "$SWARM_ROOT/.env"
GROQ_API_KEY=$USER_KEY
LLM_A2A_API_KEY=$USER_KEY
LLM_MCP_API_KEY=$USER_KEY
LLM_PLANNER_API_KEY=$USER_KEY
LLM_JUDGE_API_KEY=$USER_KEY
EOF
        echo "✔ GROQ keys configured and saved to $SWARM_ROOT/.env"
    fi
fi

echo $'\n'
echo "--------------------------------------------------------------------------"
echo " 2. Building Binaries (Release Mode)"
echo "--------------------------------------------------------------------------"

echo "• Compiling discovery, memory, and evaluation services..."
cd "$SWARM_SERVICES_ROOT"
cargo build --release --bin discovery_service --bin memory_service --bin evaluation_service

echo "• Compiling swarm agents and MCP server..."
cd "$SWARM_ROOT"
cargo build --release --example main-server
cargo build --release --bin basic_agent_launch --bin launch_planner_agent --bin launch_executor_agent --bin simple_workflow_agent_client

echo "✔ Binaries ready."

echo $'\n'
echo "--------------------------------------------------------------------------"
echo " 3. Launching Services in Background"
echo "--------------------------------------------------------------------------"

# Terminate any leftover processes on these ports first
pkill -f "examples/main-server" || true
pkill -f "discovery_service" || true
pkill -f "memory_service" || true
pkill -f "evaluation_service" || true
pkill -f "basic_agent_launch" || true
pkill -f "launch_planner_agent" || true
pkill -f "launch_executor_agent" || true
sleep 1

echo "• [1/6] Starting Discovery Service (Port 4000)..."
setsid "$SWARM_SERVICES_ROOT/target/release/discovery_service" --log-level "warn" < /dev/null > /dev/null 2>&1 &
sleep 2

echo "• [2/6] Starting Memory Service (Port 5000)..."
setsid "$SWARM_SERVICES_ROOT/target/release/memory_service" --log-level "warn" < /dev/null > /dev/null 2>&1 &
sleep 2

echo "• [3/6] Starting Evaluation Service (Port 7000)..."
setsid "$SWARM_SERVICES_ROOT/target/release/evaluation_service" --config-file "$WORKSPACE_ROOT/swarm_commons/configuration/agent_judge_config.toml" --log-level "warn" < /dev/null > /dev/null 2>&1 &
sleep 2

echo "• [4/6] Starting MCP Tools Server (Port 8000 with Weather, Customer, Scrape, Search)..."
setsid "$SWARM_ROOT/target/release/examples/main-server" --port 8000 --log-level "warn" all < /dev/null > /dev/null 2>&1 &
sleep 3

echo "• [5/6] Starting Basic Domain Agent with MCP Tools (Port 8180)..."
setsid "$SWARM_ROOT/target/release/basic_agent_launch" --config-file "kickstart/multi_agent_orchestration_kickstart/config_files/agent_basic_config.toml" --log-level "warn" < /dev/null > /dev/null 2>&1 &
sleep 4

echo "• [6/6] Starting Planner & Executor Agents (Ports 8280 & 9580)..."
setsid "$SWARM_ROOT/target/release/launch_planner_agent" --config-file "kickstart/multi_agent_orchestration_kickstart/config_files/agent_planner_config.toml" --log-level "warn" < /dev/null > /dev/null 2>&1 &
sleep 3
setsid "$SWARM_ROOT/target/release/launch_executor_agent" --config-file "kickstart/multi_agent_orchestration_kickstart/config_files/agent_executor_config.toml" --log-level "warn" < /dev/null > /dev/null 2>&1 &
sleep 3

echo $'\n'
echo "=========================================================================="
echo "          🎉 All Agents & MCP Server are Running!                         "
echo "=========================================================================="
echo " • Discovery Service:      http://127.0.0.1:4000/agents"
echo " • Memory Service:         http://127.0.0.1:5000"
echo " • Evaluation Service:     http://127.0.0.1:7000"
echo " • MCP Tools Server:       http://127.0.0.1:8000/sse"
echo " • Domain Agent (MCP):     http://127.0.0.1:8180"
echo " • Planner Orchestrator:   http://127.0.0.1:8280"
echo " • Executor Agent:         http://127.0.0.1:9580"
echo "=========================================================================="
echo $'\n'
echo "👉 Run: ./kickstart/multi_agent_orchestration_kickstart/02_test_weather_query.sh"
echo "👉 Stop: ./kickstart/multi_agent_orchestration_kickstart/03_terminate_all.sh"
