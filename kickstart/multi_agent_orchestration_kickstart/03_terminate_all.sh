#!/usr/bin/env bash

# ==============================================================================
# Terminate All Swarm, MCP and Service Processes
# ==============================================================================

echo "=========================================================================="
echo "          🛑 Terminating all Swarm & MCP processes...                    "
echo "=========================================================================="

echo "• Stopping Planner Agent..."
pkill -f "launch_planner_agent" || true

echo "• Stopping Executor Agent..."
pkill -f "launch_executor_agent" || true

echo "• Stopping Basic Domain Agent (MCP Runtime)..."
pkill -f "basic_agent_launch" || true

echo "• Stopping MCP Tools Server (main-server)..."
pkill -f "examples/main-server" || pkill -f "main-server" || true

echo "• Stopping Memory Service..."
pkill -f "memory_service" || true

echo "• Stopping Discovery Service..."
pkill -f "discovery_service" || true

echo "• Stopping Evaluation Service (if running)..."
pkill -f "evaluation_service" || true

echo "• Stopping Swarm Gateway Server (if running)..."
pkill -f "swarm_server" || true

echo "• Stopping Factory Service (if running)..."
pkill -f "launch_factory" || true

echo $'\n'
echo "✔ All Swarm processes stopped."
