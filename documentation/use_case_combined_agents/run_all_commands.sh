#!/bin/bash

echo "**************************************************************************"
echo "ALL THE BELOW EXAMPLES ARE CONFIGURED WITH GEMENI, for both agents and MCP"
echo "You should get an API key"
echo "This is the api that we will be targetting"
echo "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions"
echo "**************************************************************************"

echo "**************************************************************************"
echo "THESE COMMANDS HAVE TO BE SENT FROM THE ROOT OF THE REPOSITORY"
echo "Export LLM_A2A_API_KEY and LLM_MCP_API_KEY beforehand"
echo "Export  LLM-FULL-API-KEY beforehand"
echo "**************************************************************************"

read -n 1 -s -r -p "Press any key to start..."

echo $'\n'
echo "A) Compile the project"
cargo build --release 
echo $'\n'

echo "B) Compile the mcp server example"
cargo build --release --example main-server
echo $'\n'

echo "C) Launch three MCP server, each with individual tool on three different ports"
echo $'\n'
./target/release/examples/main-server --port 8001 weather &
./target/release/examples/main-server --port 8002 customer &
./target/release/examples/main-server --port 8003 scrape &
echo $'\n'


echo "D) Launch three agents listening on three different ports (8081,8083,8085), each using mcp config"
./target/release/simple_agent_server  --config-file "documentation/use_case_combined_agents/weather_a2a_agent.toml" &
./target/release/simple_agent_server  --config-file "documentation/use_case_combined_agents/customer_domain_a2a_agent.toml" &
./target/release/simple_agent_server  --config-file "documentation/use_case_combined_agents/web_scraper_a2a_agent.toml" &
echo $'\n' 
read -n 1 -s -r -p "Press any key to continue after three agents are launched on port 8081,8083,8085..."
echo $'\n'

echo "E) Launch full agent listening on port 9080, connected to three individual agents"
./target/release/full_agent_server --config-file "documentation/use_case_combined_agents/agent_full_config.toml" &
echo $'\n'
read -n 1 -s -r -p "Press any key to continue after coordinator agent is launched..."
echo $'\n'


echo "F) Launch the a2a_client with 4 requests, three of them are targetting a specific tool, and the fourth a general knowledge question"
#echo "Ask the questions to first agent . He should answer to the scraping question"
#./target/release/simple_a2a_client --port 8081 
#read -n 1 -s -r -p "Press any key to continue..."

#echo "Ask the questions to second agent. He should answer to the weather question"
#./target/release/simple_a2a_client --port 8083 
#read -n 1 -s -r -p "Press any key to continue..."

#echo "Ask the questions to third agent. He should answer to the customer question"
#./target/release/simple_a2a_client --port 8085 
#read -n 1 -s -r -p "Press any key to continue..."

echo "Ask the questions to full agent, the orchestrator one : He should answer to the whole set of questions"
echo $'\n'
./target/release/simple_a2a_client --port 9080 
echo $'\n'
