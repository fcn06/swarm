echo "**************************************************************************"
echo "This will kill all processes created for this demo"
echo "Orchestration agent, three domain agents, mcp servers,"
echo "memory service, evaluation service and discovery service"
echo "**************************************************************************"

read -n 1 -s -r -p "Press any key to start..."

echo "Kill Orchestration Agent"
echo $'\n'
pkill -f "orchestration_a"

echo "Kill The three domain agents"
echo $'\n'
pkill -f "basic_agent_lau"

echo "Kill The three MCP Servers"
echo $'\n'
pkill -f "main-server"

echo "Kill The Memory Service"
echo $'\n'
pkill -f "memory_service"

echo "Kill The Evaluation Service"
echo $'\n'
pkill -f "evaluation_serv"

echo "Kill The Discovery Service"
echo $'\n'
pkill -f "discovery_servi"