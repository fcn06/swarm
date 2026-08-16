#!/bin/bash

echo "**************************************************************************"
echo "THESE COMMANDS HAVE TO BE SENT FROM THE ROOT OF THE REPOSITORY"
echo "Export LLM_A2A_API_KEY beforehand as API Key related to LLM controlling Domain Agents."
echo "Export LLM_MCP_API_KEY beforehand as API Key related to LLM Controlling MCP Runtime."
echo "Export LLM_JUDGE_API_KEY beforehand as the API Key related to LLM Controlling Evaluation Service (LLM as a Judge)."
echo "Export LLM_PLANNER_API_KEY beforehand as the API Key related to LLM controlling the planner agent interaction."
echo "So to summarize : "
echo "export LLM_A2A_API_KEY=<YOUR GROQ API KEY>"
echo "export LLM_MCP_API_KEY=<YOUR GROQ API KEY>"
echo "export LLM_JUDGE_API_KEY=<YOUR GROQ API KEY>"
echo "export LLM_PLANNER_API_KEY=<YOUR GROQ API KEY>"
echo "**************************************************************************"

printf "Press Enter to start..."
read -r _

echo ""
echo "A) Compile the project"
cargo build --release 
echo ""

echo "B) Compile the mcp server example"
cargo build --release --example main-server
echo ""

echo "##############################################################"
echo ""

echo "C) Launch discovery Service so that agents can self register"
../swarm_services/target/release/discovery_service --log-level "warn" &
sleep 3
echo ""

echo "C) Launch Evaluation Service so that agents can have their output evaluated (LLM as a Judge)"
../swarm_services/target/release/evaluation_service --config-file "documentation/demo_planner_executor_management/agent_judge_config.toml" --log-level "warn" &
sleep 3
echo ""

echo "C) Launch Memory Service so that agents can log their conversation and plan"
../swarm_services/target/release/memory_service --log-level "warn" &
sleep 3
echo ""

echo "D) Launch MCP server on port 8000 with all tools"
./target/release/examples/main-server --port 8000 --log-level "warn" all &
sleep 4
echo ""

echo "##############################################################"
echo ""

echo "E) Launch basic domain agent listening on port 8080"
echo "Basic Domain Agent"
echo ""

./target/release/basic_agent_launch --config-file "documentation/demo_planner_executor_management/agent_basic_config.toml" --log-level "warn" &
sleep 5
echo ""

echo "##############################################################"
echo ""

echo "G) Launch Planner Management Agent listening on port 8280"
echo "Planner Agent"
echo ""
./target/release/launch_planner_agent --config-file "documentation/demo_planner_executor_management/agent_planner_config.toml" --log-level "warn" &
sleep 5
echo ""

echo "##############################################################"
echo ""

echo "F) Launch Executor Management Agent listening on port 9580"
echo "Executor Agent"
echo ""
./target/release/launch_executor_agent --config-file "documentation/demo_planner_executor_management/agent_executor_config.toml" --log-level "warn" &
sleep 5
echo ""

echo "##############################################################"
echo ""

echo "F) Launch the agent_client with complex requests."
echo "The outcome of the agents will be evaluated by the LLM as a Judge."
echo ""
printf "Press Enter to continue..."
read -r _
echo ""

echo "Ask the questions to the orchestrator..."
echo ""

CLIENT_COMMAND="./target/release/simple_workflow_agent_client --port 8280 --log-level \"warn\""
GENERATION_TYPE_ARG=""

if [ "$1" = "--dynamic-generation" ]; then
    GENERATION_TYPE_ARG="--generation-type \"dynamic_generation\""
elif [ "$1" = "--high-level-plan" ]; then
    GENERATION_TYPE_ARG="--generation-type \"high_level_plan\""
fi

GRAPH_FILE_ARG=""
if [ -n "$2" ]; then
    GRAPH_FILE_ARG="--graph-file \"$2\""
fi

eval "$CLIENT_COMMAND $GENERATION_TYPE_ARG $GRAPH_FILE_ARG"

echo ""
echo "When it is finished..." 
echo "* You can access registered agents:"
echo "curl http://127.0.0.1:4000/agents"
echo "* You can access evaluation of the outcome of the agents:"
echo "curl http://127.0.0.1:7000/evaluations"
echo ""
