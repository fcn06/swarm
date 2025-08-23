echo "**************************************************************************"
echo "This will kill all processes created for this demo"
echo "Orchestration agent, three domain agents, mcp servers,"
echo "memory service, evaluation service and discovery service"
echo "**************************************************************************"

read -n 1 -s -r -p "Press any key to start..."

echo "Kill Workflow Agent"
echo $'\n'
pkill -f "launch_agent_wo"

echo "Kill The  domain agent"
echo $'\n'
pkill -f "basic_agent_lau"

echo "Kill The  MCP Server"
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