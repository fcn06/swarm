#!/bin/bash

echo "**************************************************************************"
echo "THE BELOW EXAMPLES ARE CONFIGURED WITH GEMENI, for workflow agents"
echo "You should get an API key"
echo "This is the url that we will be targetting"
echo "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions"
echo "**************************************************************************"
echo $'\n'
echo "**************************************************************************"
echo "BUT FOR LLM as a Judge, we are using a different model. In our case, we will use openai/gpt-oss-20b, exposed from GROQ"
echo "You should get an API key"
echo "This is the url that we will be targetting"
echo "https://api.groq.com/openai/v1/chat/completions"
echo "**************************************************************************"

echo "**************************************************************************"
echo "THESE COMMANDS HAVE TO BE SENT FROM THE ROOT OF THE REPOSITORY"
echo "Export LLM_A2A_API_KEY beforehand beforehand as API Key related to LLM controlling Domain Agents. Our configuration files are targetting Groq"
echo "Export LLM_MCP_API_KEY beforehand beforehand as API Key related to LLM Controlling MCP Runtime. Our configurations are targetting Groq"
echo "Export LLM_JUDGE_API_KEY  beforehand as the API Key related to LLM Controlling Evaluation Service ( LLM as a Judge).Our configurations are targetting GROQ"
echo "Export LLM_PLANNER_API_KEY  beforehand as the API Key related to LLM controlling the planner agent interaction.Our configurations are targetting GROQ"
echo "Export LLM_EXECUTOR_API_KEY  beforehand as the API Key related to LLM controlling the executor agent interaction.Our configurations are targetting GROQ"
echo "So to summarize : "
echo "export LLM_A2A_API_KEY=<YOUR GROQ API KEY>"
echo "export LLM_MCP_API_KEY=<YOUR GROQ API KEY>"
echo "export LLM_JUDGE_API_KEY=<YOUR GROQ API KEY>"
echo "export LLM_PLANNER_API_KEY=<YOUR GROQ API KEY>"
echo "export LLM_EXECUTOR_API_KEY=<YOUR GROQ API KEY>"
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

echo "E) Launch one basic domain agents listening on port 8080, with a dedicated mcp config"
echo $'\n'
echo "Basic Domain Agent"
echo $'\n'

./target/release/basic_agent_launch --config-file "documentation/demo_planner_executor_management/agent_basic_config.toml"  --log-level "warn" &

sleep 5
echo $'\n'


echo "##############################################################"
echo $'\n'

echo "G) Launch Planner Management Agent listening on port 8180, connected to one individual domain agent"
echo $'\n'
echo "Planner Agent"
echo $'\n'
./target/release/launch_planner_agent --config-file "documentation/demo_planner_executor_management/agent_planner_config.toml"  --log-level "warn"  &
sleep 5
echo $'\n'

echo "##############################################################"
echo $'\n'

echo "F) Launch Executor Management Agent listening on port 8180, connected to one individual domain agent"
echo $'\n'
echo "Executor Agent"
echo $'\n'
./target/release/launch_executor_agent --config-file "documentation/demo_planner_executor_management/agent_executor_config.toml"  --log-level "warn"  &
sleep 5
echo $'\n'

echo "##############################################################"
echo $'\n'

echo "F) Launch the agent_client with complex requests. The outcome of the agents will be evaluated by the LLM as a Judge."
echo " Agents will redefine and re execute workflow if evaluation rating is below 5 out of 10."

echo $'\n'
read -n 1 -s -r -p "Press any key to continue..."
echo $'\n'


echo "Ask the questions to the orchestrator : He should answer to the whole set of questions by send queries to appropriate agent"
echo $'\n'

# Check for the --dynamic-generation or --high-level-plan flag
#
#if [[ "$1" == "--dynamic-generation" ]]; then
#    ./target/release/simple_workflow_agent_client --port 8280 --log-level "warn" --generation-type "dynamic_generation" 
#elif [[ "$1" == "--high-level-plan" ]]; then
#    ./target/release/simple_workflow_agent_client --port 8280 --log-level "warn" --generation-type "high_level_plan"
#else
    # execution based on a pre defined workflow. good for automation
#    ./target/release/simple_workflow_agent_client --port 8280 --log-level "warn"
#fi


# Base command for the simple_workflow_agent_client
CLIENT_COMMAND="./target/release/simple_workflow_agent_client --port 8280 --log-level \"warn\""

# Variable to hold additional arguments based on the first parameter
GENERATION_TYPE_ARG=""

# Check for the --dynamic-generation or --high-level-plan flag
if [[ "$1" == "--dynamic-generation" ]]; then
    GENERATION_TYPE_ARG="--generation-type \"dynamic_generation\""
elif [[ "$1" == "--high-level-plan" ]]; then
    GENERATION_TYPE_ARG="--generation-type \"high_level_plan\""
fi

# Variable to hold the --graph-file argument based on the second parameter
GRAPH_FILE_ARG=""

# Check if the second parameter (config file path) is provided
# -n "$2" checks if the string stored in $2 is not empty
if [[ -n "$2" ]]; then
    GRAPH_FILE_ARG="--graph-file \"$2\""
fi

# Construct the final command
# We use 'eval' to correctly handle the quoted arguments within the variables
eval "$CLIENT_COMMAND $GENERATION_TYPE_ARG $GRAPH_FILE_ARG"



echo $'\n'

echo "When it is finished..." 
echo "* You can access registered agents:"
echo "curl http://127.0.0.1:4000/agents"
echo "* You can access evaluation of the outcome of the agents"
echo "curl http://127.0.0.1:7000/evaluations"
echo "Just Try them"
echo $'\n'
