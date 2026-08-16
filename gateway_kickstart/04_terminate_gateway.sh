#!/usr/bin/env bash

# ==============================================================================
# Terminate Swarm Standalone Gateway Server
# ==============================================================================

echo "=========================================================================="
echo "          🛑 Stopping Swarm Standalone Gateway Server...                 "
echo "=========================================================================="

pkill -f "swarm_server" || true

echo "✔ Gateway server stopped."
