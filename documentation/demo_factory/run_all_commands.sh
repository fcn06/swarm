#!/bin/bash

echo "**************************************************************************"
echo "THESE COMMANDS HAVE TO BE SENT FROM THE ROOT OF THE REPOSITORY"
echo "Export LLM_A2A_API_KEY beforehand beforehand as API Key related to LLM controlling Domain Agents. Our configuration files are targetting Groq"
echo "Export LLM_JUDGE_API_KEY  beforehand as the API Key related to LLM Controlling Evaluation Service ( LLM as a Judge).Our configurations are targetting GROQ"
echo "So to summarize : "
echo "export LLM_A2A_API_KEY=<YOUR GROQ API KEY>"
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
../swarm_services/target/release/discovery_service  --log-level "warn" &
sleep 3
echo $'\n'

echo "C) Launch Evaluation Service so that agents can have their output evaluated ( LLM as a Judge)"
../swarm_services/target/release/evaluation_service --config-file "documentation/demo_planner_executor_management/agent_judge_config.toml"  --log-level "warn" &
sleep 3
echo $'\n'

echo "C) Launch Memory Service so that agents can have log their conversation and plan"
../swarm_services/target/release/memory_service  --log-level "warn" &
sleep 3
echo $'\n'

echo "D) Launch three MCP server, each with individual tool on three different ports"
./target/release/examples/main-server --port 8000 --log-level "warn" all &
#./target/release/examples/main-server --port 8000 --log-level "info" all &
sleep 4
echo $'\n'

echo "##############################################################"
echo $'\n'

echo "E) Launch factory and one basic domain agents listening on port 8080, with a dedicated mcp config"
echo $'\n'
echo "Factory Agent"
echo $'\n'

./target/release/launch_factory --config-file "documentation/demo_factory/factory_config.toml"  --log-level "warn" 


echo "##############################################################"
echo $'\n'



echo "When it is finished..." 
echo "* You can access registered agents:"
echo "curl http://127.0.0.1:4000/agents"
echo "* You can access evaluation of the outcome of the agents"
echo "curl http://127.0.0.1:7000/evaluations"
echo "Just Try them"
echo $'\n'
