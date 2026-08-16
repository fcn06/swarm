A2A Agent Endpoint to interact directly with an A2A Agent

To compile it

```bash
cargo build --release --example a2a_agent_endpoint
```

To run it you need do the following things : 

Then run the server :
```bash
./target/release/examples/a2a_agent_endpoint &
```

Here is an example call to this REST Endpoint :
```bash
curl -d '{"role":"user", "content":"What is the weather in Boston"}' -H "Content-Type: application/json" -X POST http://localhost:3000/msg
```
