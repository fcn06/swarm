# **Swarm.rs: Orchestrating Intelligent Agents in Rust 🦀**

## **Welcome to Swarm.rs\!**

Swarm.rs is a Rust project that empowers you to build and orchestrate intelligent agents. Think of it as a control center for your digital assistants, allowing them to communicate, collaborate, and tackle complex tasks together.

We combine two powerful communication protocols:

* MCP (Microservice Communication Protocol): For agents to interact with external services and data sources. Imagine your agents querying a weather API or accessing a database\!  
* A2A (Agent-to-Agent Protocol): For agents to talk directly to each other, sharing information and coordinating efforts.

The real magic happens with our Planner. It's the brain of the operation, observing your agents, understanding their capabilities, and orchestrating them to achieve ambitious goals.

Whether you're a Rust enthusiast, an AI developer, or just curious about multi-agent systems, we invite you to explore Swarm.rs\!

## **Why Swarm.rs?**

* Rust-powered performance and safety: Build robust and efficient agent systems.  
* Flexible architecture: Easily integrate with external services and create collaborative agent networks.  
* Intuitive orchestration: The Planner simplifies complex multi-agent workflows.  
* Open for collaboration: We're actively developing and welcome your ideas and contributions\!

## **How It Works: A Glimpse into the Architecture**

The diagram below illustrates how our agents, powered by MCP and A2A, interact under the guidance of the Planner.

![Swarm Architecture](documentation/a2a_mcp_architecture.drawio.png)

* MCP Agents: Your agents that connect to the outside world (e.g., fetching real-time data, interacting with APIs).  
* A2A Agents: Agents that specialize in specific tasks and communicate with each other to achieve sub-goals.  
* Planner: The orchestrator that understands the overall goal, breaks it down, and directs the A2A agents to execute the plan.

## **Getting Started: Join the Swarm\!**

Ready to dive in? Here's how to get your first Swarm.rs components up and running.

### **Prerequisites**

To enable your agents to think and communicate, you'll need access to an OpenAI-compatible chat completion API.

* Our recommendation for speed: [Groq](https://api.groq.com/openai/v1/chat/completions) (we've had great success with the qwen/qwen3-32b model).  
* Also tested with: [Gemini](https://generativelanguage.googleapis.com/v1beta/openai/chat/completions).

You'll also benefit from having an mcp\_server running. 

Here is a separate complementary project may be used to start your testing :

* Test MCP Server : [https://github.com/fcn06/mcp_server](https://github.com/fcn06/mcp_server) 



### **Installation & Building**

First, compile the workspace to ensure all dependencies are in sync:
```bash
cargo build --release
```

### **Running Components**

Now, let's fire up some agents\!

* A2A Agent Server (your individual intelligent assistants):  
  You can run multiple A2A agents, each with its own configuration.  

```bash
  # Option 1: Run directly with cargo ( LLM_API_KEY needs to be set in .env)  
  cargo run --bin simple_agent_server -- --config-file "configuration/agent_a2a_config.toml"

  # Option 2: Run compiled binary (recommended. As per configuration, it uses Groq and qwen3)  
  LLM_API_KEY=<YOUR-API-KEY> ./target/release/simple_agent_server --config-file "configuration/agent_a2a_config.toml"
```
NB: It seems that google openai compatible api, does not accept tool_call_id that is needed for MCP protocol, so I suggest to stay with groq for a2a/mcp agent


* Planner (the orchestrator):  
  Ask the Planner to achieve a goal by providing a user query.  
```bash
  # Option 1: Run directly with cargo  ( LLM_API_KEY needs to be set in .env) 
  cargo run --bin planner_agent -- --user-query "What is the weather in Boston?"

  # Option 2: Run compiled binary (recommended. As per configuration it uses gemini)
  LLM_API_KEY=<YOUR-API-KEY> ./target/release/planner_agent --user-query "What is the weather in Boston?"
```
  *Remember to replace "What is the weather in Boston?" with your own query\!*

### **Configuration**

The configuration directory is your go-to for customizing agent behavior:

*   `agent_a2a_config.toml`: Configuration for A2A agents.
*   `agent_mcp_config.toml`: Configuration for MCP agents.
*   `agent_planner_config.toml`: Configuration for the planner agent.


LLM Models:  
We've found `qwen/qwen3-32b` to be highly effective, especially for MCP interactions.  

Currently, A2A and MCP agents share the same LLM, but the Planner can use a different one (configurable via LLM\_API\_URL in `agent_planner_config.toml`).

## **Under the Hood: Swarm.rs Crate Breakdown**

The swarm project is composed of several specialized sub-crates:

* `a2a_agent_backbone`: The core framework for A2A agents, including a server for standalone agents and a client for testing. These agents can also incorporate MCP agents for external interactions.  
* `a2a_planner_backbone`: The brain for the A2A Planner. It connects to declared A2A agents, understands their skills, creates a plan, and executes it to achieve your goals.  
* `configuration`: Manages all your Swarm.rs configuration files.  
* `llm_api`: Provides a convenient interface for interacting with various Large Language Models via an OpenAI-compatible API.  
* `mcp_agent_backbone`: The core framework for MCP agents. Note: An external mcp\_server (like the illustrative project or Apify) is needed for these agents to function.  
* `mcp_agent_endpoint`: A testing utility for MCP agents to receive and process requests.

## **Road Ahead & How You Can Contribute**

Swarm.rs is a project born out of discovery and exploration\! While not production-ready, it's a fantastic playground for understanding how these protocols can be combined to build powerful agentic systems.

We're continuously working on improvements, including:

* Recursivity: Enabling "planner of planners" for even more complex orchestration.  Work In Progress
* Refactoring: Continuously improving code clarity and maintainability.  
* Unit Tests: Enhancing robustness and reliability.

We're eager for your comments, suggestions, and contributions\! Whether it's a bug report, a feature idea, or a pull request, your input helps shape the future of Swarm.rs.

## **Special Thanks**

We heavily rely on and appreciate the fantastic work of these actively developed crates:

* MCP Protocol: [https://github.com/modelcontextprotocol/rust-sdk](https://github.com/modelcontextprotocol/rust-sdk)  
* A2A Protocol: [https://github.com/EmilLindfors/a2a-rs](https://github.com/EmilLindfors/a2a-rs)

*Note: Due to their active development, we might occasionally pin to specific commits in Cargo.toml to maintain stability.*