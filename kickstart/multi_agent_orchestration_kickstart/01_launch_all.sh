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
elif [ -n "$LLM_A2A_API_KEY" ]; then
    export GROQ_API_KEY="${GROQ_API_KEY:-$LLM_A2A_API_KEY}"
    export LLM_MCP_API_KEY="${LLM_MCP_API_KEY:-$LLM_A2A_API_KEY}"
    export LLM_PLANNER_API_KEY="${LLM_PLANNER_API_KEY:-$LLM_A2A_API_KEY}"
fi

echo "Summary of GROQ API Key Variables:"
echo " • LLM_A2A_API_KEY    : ${LLM_A2A_API_KEY:+[SET]}${LLM_A2A_API_KEY:-[MISSING - for Domain / Basic Agent with MCP tools]}"
echo " • LLM_MCP_API_KEY    : ${LLM_MCP_API_KEY:+[SET]}${LLM_MCP_API_KEY:-[MISSING - for MCP Runtime Tool Loop]}"
echo " • LLM_PLANNER_API_KEY: ${LLM_PLANNER_API_KEY:+[SET]}${LLM_PLANNER_API_KEY:-[MISSING - for Planner Agent Orchestrator]}"

if [ -z "$LLM_A2A_API_KEY" ]; then
    echo $'\n'
    echo "⚠️  WARNING: No Groq API Key was detected in your environment."
    read -p "Enter your GROQ API key now: " USER_KEY
    if [ -n "$USER_KEY" ]; then
        export GROQ_API_KEY="$USER_KEY"
        export LLM_A2A_API_KEY="$USER_KEY"
        export LLM_MCP_API_KEY="$USER_KEY"
        export LLM_PLANNER_API_KEY="$USER_KEY"
        echo "✔ GROQ keys configured."
    fi
fi

echo $'\n'
echo "--------------------------------------------------------------------------"
echo " 2. Building Binaries (Release Mode)"
echo "--------------------------------------------------------------------------"

echo "• Compiling discovery and memory services..."
cd "$SWARM_SERVICES_ROOT"
cargo build --release --bin discovery_service --bin memory_service

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
pkill -f "basic_agent_launch" || true
pkill -f "launch_planner_agent" || true
pkill -f "launch_executor_agent" || true
sleep 1

echo "• [1/5] Starting Discovery Service (Port 4000)..."
"$SWARM_SERVICES_ROOT/target/release/discovery_service" --log-level "warn" &
sleep 2

echo "• [2/5] Starting Memory Service (Port 5000)..."
"$SWARM_SERVICES_ROOT/target/release/memory_service" --log-level "warn" &
sleep 2

echo "• [3/5] Starting MCP Tools Server (Port 8000 with Weather, Customer, Scrape, Search)..."
"$SWARM_ROOT/target/release/examples/main-server" --port 8000 --log-level "warn" all &
sleep 3

echo "• [4/5] Starting Basic Domain Agent with MCP Tools (Port 8080)..."
"$SWARM_ROOT/target/release/basic_agent_launch" --config-file "kickstart/multi_agent_orchestration_kickstart/config_files/agent_basic_config.toml" --log-level "warn" &
sleep 4

echo "• [5/5] Starting Planner & Executor Agents (Ports 8280 & 9580)..."
"$SWARM_ROOT/target/release/launch_planner_agent" --config-file "kickstart/multi_agent_orchestration_kickstart/config_files/agent_planner_config.toml" --log-level "warn" &
sleep 3
"$SWARM_ROOT/target/release/launch_executor_agent" --config-file "kickstart/multi_agent_orchestration_kickstart/config_files/agent_executor_config.toml" --log-level "warn" &
sleep 3

echo $'\n'
echo "=========================================================================="
echo "          🎉 All Agents & MCP Server are Running!                         "
echo "=========================================================================="
echo " • Discovery Service:      http://127.0.0.1:4000/agents"
echo " • Memory Service:         http://127.0.0.1:5000"
echo " • MCP Tools Server:       http://127.0.0.1:8000/sse"
echo " • Domain Agent (MCP):     http://127.0.0.1:8080"
echo " • Planner Orchestrator:   http://127.0.0.1:8280"
echo " • Executor Agent:         http://127.0.0.1:9580"
echo "=========================================================================="
echo $'\n'
echo "👉 Run: ./kickstart/multi_agent_orchestration_kickstart/02_test_weather_query.sh"
echo "👉 Stop: ./kickstart/multi_agent_orchestration_kickstart/03_terminate_all.sh"
