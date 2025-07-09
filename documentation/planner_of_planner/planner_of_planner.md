Here are the commands to test planner server, that will be the enabler for recursivity


Launch MCP Server (https://github.com/fcn06/mcp_server)
```bash
./target/release/main-server
```

Launch a2a server agent ( as per configuration, this uses groq and qwen3 )
```bash
LLM_API_KEY=<YOUR-API-KEY> ./target/release/simple_agent_server --config-file "configuration/agent_a2a_config.toml"
```

Launch planner server ( as per configuration, this uses Gemini)
```bash
LLM_API_KEY=<YOUR-API-KEY> ./target/release/planner_server
```

Launch a2a client that will send a query to planner_server
```bash
./target/release/simple_a2a_client 
```
