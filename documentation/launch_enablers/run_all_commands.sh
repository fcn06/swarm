#!/bin/bash

echo "**************************************************************************"
echo "THESE COMMANDS HAVE TO BE SENT FROM THE ROOT OF THE REPOSITORY"
echo "Export LLM_JUDGE_API_KEY  beforehand as the API Key related to LLM Controlling Evaluation Service ( LLM as a Judge).Our configurations are targetting GROQ"
echo "export LLM_JUDGE_API_KEY=<YOUR GROQ API KEY>"
echo "**************************************************************************"

read -n 1 -s -r -p "Press any key to start..."

echo $'\n'
echo "A) Compile the project"
cargo build --release 
echo $'\n'

echo "B) Compile the mcp server example"
cargo build --release --example main-server
echo $'\n'

echo "##############################################################"
echo $'\n'

echo "C) Launch discovery Service so that agents can self register"
./target/release/discovery_service  --log-level "warn" &
sleep 3
echo $'\n'

echo "C) Launch Evaluation Service so that agents can have their output evaluated ( LLM as a Judge)"
./target/release/evaluation_service --config-file "documentation/demo_planner_executor_management/agent_judge_config.toml"  --log-level "warn" &
sleep 3
echo $'\n'

echo "C) Launch Memory Service so that agents can have log their conversation and plan"
./target/release/memory_service  --log-level "warn" &
sleep 3
echo $'\n'

echo "D) Launch three MCP server, each with individual tool on three different ports"
./target/release/examples/main-server --port 8000 --log-level "warn" all &
sleep 4
echo $'\n'

echo "##############################################################"
echo $'\n'

