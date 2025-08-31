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
echo "Export LLM_WORKFLOW_API_KEY  beforehand as the API Key related to LLM controlling the workflow agent interaction.Our configurations are targetting GROQ"
echo "So to summarize : "
echo "export LLM_A2A_API_KEY=<YOUR GROQ API KEY>"
echo "export LLM_MCP_API_KEY=<YOUR GROQ API KEY>"
echo "export LLM_JUDGE_API_KEY=<YOUR GROQ API KEY>"
echo "export LLM_WORKFLOW_API_KEY=<YOUR GROQ API KEY>"
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
./target/release/evaluation_service --config-file "documentation/demo_workflow_management/agent_judge_config.toml"  --log-level "warn" &
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

./target/release/basic_agent_launch --config-file "documentation/demo_workflow_management/agent_basic_config.toml"  --log-level "warn" &

sleep 5
echo $'\n'


echo "##############################################################"
echo $'\n'

echo "F) Launch Workflow Management Agent listening on port 8180, connected to one individual domain agent"
echo $'\n'
echo "Workflow Agent"
echo $'\n'
./target/release/launch_agent_workflow --config-file "documentation/demo_workflow_management/agent_workflow_config.toml"  --log-level "warn"  &
sleep 5
echo $'\n'

echo "##############################################################"
echo $'\n'

echo "F) Launch the agent_client with complex requests, some of them are targetting a specific tool, and the last one is  a general knowledge question"

echo $'\n'
read -n 1 -s -r -p "Press any key to continue..."
echo $'\n'


echo "Ask the questions to the orchestrator : He should answer to the whole set of questions by send queries to appropriate agent"
echo $'\n'

# Check for the --dynamic-generation flag
if [[ "$1" == "--dynamic-generation" ]]; then
    ./target/release/simple_workflow_agent_client --port 8180 --log-level "warn" --dynamic-generation
else
    # execution based on a pre defined workflow. good for automation
    ./target/release/simple_workflow_agent_client --port 8180 --log-level "warn"
fi

echo $'\n'

echo "When it is finished..." 
echo "* You can access registered agents:"
echo "curl http://127.0.0.1:4000/agents"
echo "Just Try it"
echo $'\n'
