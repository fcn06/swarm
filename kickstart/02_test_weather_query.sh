#!/usr/bin/env bash

# ==============================================================================
# Sample MCP Agent Request Script: Weather in Boston (Call & Output)
# ==============================================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SWARM_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$SWARM_ROOT"

USER_QUERY="${1:-What is the current weather in Boston ?}"

echo "=========================================================================="
echo "    🌤️  Executing Multi-Agent Query with MCP Tool: Weather in Boston     "
echo "=========================================================================="
echo " Query  : \"$USER_QUERY\""
echo " Route  : Planner Agent (8280) -> Domain Agent (8080) -> MCP Server (8000)"
echo " Config : kickstart/config_files/"
echo "=========================================================================="
echo ""

./target/release/simple_workflow_agent_client \
  --port 8280 \
  --log-level "warn" \
  --generation-type "dynamic_generation" \
  --graph-file "kickstart/config_files/mix_agent_tools_workflow.json" \
  --user-query "$USER_QUERY"

echo ""
echo "=========================================================================="
echo "✔ Query completed!"
echo "=========================================================================="
