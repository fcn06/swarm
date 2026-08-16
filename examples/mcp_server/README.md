# MCP Server

This is a very simple crate intended for illustration and testing purposes with the `swarm` project.

It provides a basic implementation of an MCP (Multi-Agent Communication Protocol) server.

The server exposes two tools:

*   `get_weather`: A tool to simulate fetching weather information.
*   `get_customer_details`: A tool to simulate retrieving customer details.
*   `scrape_url` : A tool to scrape an url and return a llm friendly content, using jina.ai reader api.

These tools are designed to be simple examples for agents within the `swarm` project to interact with, demonstrating the communication and tool-use capabilities within the agentic system.


### **Run this simple server**

First compile the project

```bash
cargo build --release --example main-server
```


Run the test MCP server that exposes these three tools

```bash
./target/release/examples/main-server all &
```

You can also launch a specific server instance for a specific port and a specific tool

For weather tool
```bash
./target/release/examples/main-server --port 8001 weather &
```

for customer tool
```bash
./target/release/examples/main-server --port 8002 customer &
```

for scraping url tool using jina.ai reader
```bash
./target/release/examples/main-server --port 8003 scrape &
```
