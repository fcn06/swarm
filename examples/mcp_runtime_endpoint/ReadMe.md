MCP Agent Endpoint to interact directly with MCP runtime

To compile it

```bash
cargo build --release --example mcp_agent_endpoint
```

To run it you need do the following things : 

Declare API key for LLM server that will manage your tools
```bash
export LLM_MCP_API_KEY="<YOUR-MCP-API-KEY>"
```

Then run the server :
```bash
./target/release/examples/mcp_agent_endpoint &
```

Here is an example call to this REST Endpoint :
```bash
curl -d '{"role":"user", "content":"What is the weather in Boston"}' -H "Content-Type: application/json" -X POST http://localhost:3000/msg
```
