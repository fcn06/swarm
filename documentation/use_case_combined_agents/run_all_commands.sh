#!/bin/bash

echo "**************************************************************************"
echo "THE BELOW EXAMPLES ARE CONFIGURED WITH GEMENI, for Domain Agents and Orchestraor, as well as MCP runtime"
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
echo "Export LLM_A2A_API_KEY beforehand beforehand as API Key related to LLM controlling Domain Agents. Our configuration files are targetting Gemini"
echo "Export LLM_MCP_API_KEY beforehand beforehand as API Key related to LLM Controlling MCP Runtime. Our configurations are targetting Gemini"
echo "Export  LLM-FULL-API-KEY beforehand as API Key related to LLM controlling orchestrator. Our configurations are targetting Gemini" 
echo "Export LLM_JUDGE_API_KEY  beforehand as the API Key related to LLM Controlling Evaluation Service ( LLM as a Judge).Our configurations are targetting GROQ"
echo "So to summarize : "
echo "export LLM_A2A_API_KEY=<YOUR GEMENI API KEY>"
echo "export LLM_MCP_API_KEY=<YOUR GEMENI API KEY>"
echo "export LLM_FULL_API_KEY=<YOUR GEMENI API KEY>"
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
./target/release/discovery_service &
sleep 3
echo $'\n'

echo "C) Launch Evaluation Service so that agents can have their output evaluated ( LLM as a Judge)"
./target/release/evaluation_service &
sleep 3
echo $'\n'

echo "C) Launch Memory Service so that agents can have log their conversation and plan"
./target/release/memory_service &
sleep 3
echo $'\n'

echo "D) Launch three MCP server, each with individual tool on three different ports"
./target/release/examples/main-server --port 8001 weather &
./target/release/examples/main-server --port 8002 customer &
./target/release/examples/main-server --port 8003 scrape &
sleep 4
echo $'\n'
echo $'\n'

echo "##############################################################"
echo $'\n'

echo "E) Launch three domain agents listening on three different ports (8081,8083,8085), each using mcp config"
echo $'\n'
echo "Weather Domain Agent"
echo $'\n'

./target/release/basic_agent_launch --config-file "documentation/use_case_combined_agents/weather_domain_agent.toml" &

sleep 5
echo $'\n'
echo $'\n'

echo $'\n'
echo "Customer Domain Agent"
echo $'\n'
./target/release/basic_agent_launch  --config-file "documentation/use_case_combined_agents/customer_domain_agent.toml" &
sleep 5
echo $'\n'
echo $'\n'

echo $'\n'
echo "Web Scraper Domain Agent"
echo $'\n'
./target/release/basic_agent_launch  --config-file "documentation/use_case_combined_agents/web_scraper_domain_agent.toml" &
sleep 8
echo $'\n'
echo $'\n'

echo "##############################################################"
echo $'\n'

echo "F) Launch Orchestration agent listening on port 9080, connected to three individual agents"
echo $'\n'
echo "Orchestrator Agent"
echo $'\n'
./target/release/orchestration_agent_launch --config-file "documentation/use_case_combined_agents/agent_orchestration_config.toml"  &
# If you want to have specific log level you can specify it on command line ( trace, debug, info, warn, error. Default is warn)
#./target/release/orchestration_agent_launch --config-file "documentation/use_case_combined_agents/agent_full_config.toml" --log-level "debug" &
sleep 5
echo $'\n'
echo $'\n'

echo "##############################################################"
echo $'\n'

echo "F) Launch the agent_client with complex requests, some of them are targetting a specific tool, and the last one is  a general knowledge question"

echo $'\n'
read -n 1 -s -r -p "Press any key to continue..."
echo $'\n'

#echo "Ask the questions to first agent . He should answer to the scraping question"
#./target/release/simple_a2a_client --port 8081 
#read -n 1 -s -r -p "Press any key to continue..."

#echo "Ask the questions to second agent. He should answer to the weather question"
#./target/release/simple_a2a_client --port 8083 
#read -n 1 -s -r -p "Press any key to continue..."

#echo "Ask the questions to third agent. He should answer to the customer question"
#./target/release/simple_a2a_client --port 8085 
#read -n 1 -s -r -p "Press any key to continue..."

echo "Ask the questions to the orchestrator : He should answer to the whole set of questions by send queries to appropriate agent"
echo $'\n'
./target/release/simple_a2a_client --port 9080 
echo $'\n'

echo "When it is finished..." 
echo $'\n'
echo "* You can access evaluations of the orchestrator by the LLM Judge :"
echo "curl http://127.0.0.1:7000/evaluations"
echo $'\n'
echo "* You can access what is recorded by memory service:"
echo "curl http://127.0.0.1:5000/conversation/{conversation_id}"
echo $'\n'
echo "* You can access registered agents:"
echo "curl http://127.0.0.1:4000/agents"
echo $'\n'

echo "Just Try them"
echo $'\n'