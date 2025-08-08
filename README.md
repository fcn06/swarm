# 🚀 Build, Connect, and Orchestrate Intelligent Agents in Rust 🦀

## **Why Swarm?**

Imagine a team of digital assistants, each focusing on one domain of expertise: one for shopping, one for customer service, another for data analysis. Swarm is the framework that lets them work together seamlessly to support customer requests...You can configure intelligent agents, connect, and orchestrate  and allow them to collaborate. 

We combine two powerful communication protocols (MCP and A2A) and an orchestration capability to create a control center for your digital assistants. Swarm enables **recursive design**. You can create, according to business value, a sophisticated network of agents.

Whether you're a Rust enthusiast, an AI developer, or just curious about multi-agent systems, Swarm provides a flexible and powerful framework for your agentic applications!

## **🌐 How It Works**

The diagram below illustrates how our agents, powered by MCP and A2A, interact under the guidance of the Full Agent. This architecture allows agents to communicate with each other to achieve sub-goals, connect to the outside world, and execute complex plans.

<p align="center" width="60%">
    <img width="60%" src="./documentation/illustrations/Simple_Representation_Architecture.png">
</p>

For a detailed description of Swarm's core components, refer to the "Core Components of Swarm" section below.

*(Note: An optional MCP Server is provided in the `examples` section for testing purposes.)*


## **🚀 Getting Started & Prerequisites**

Ready to dive in? Here's how to get your first Swarm components up and running.

**1. Install Rust:** If you don't have Rust installed, follow the instructions on the [official Rust website](https://www.rust-lang.org/tools/install).

**2. Obtain LLM API Keys:** To enable your agents to think and communicate, you'll need access to an OpenAI-compatible chat completion API. 

We have tested Groq and gemini, for which you can access pretty good free plan:

*   **Groq:**  [https://api.groq.com/openai/v1/chat/completions](https://api.groq.com/openai/v1/chat/completions). 

*   **Gemini:** [https://generativelanguage.googleapis.com/v1beta/openai/chat/completions](https://generativelanguage.googleapis.com/v1beta/openai/chat/completions)

Below is summarized results of our internal evaluation :

| Provider | Model | General Purpose | MCP | Orchestration | Comments |
| :--- | :--- | :---: | :---: | :---: | :---: |
|Groq | qwen/qwen3-32b | ✓ | ✓ | ✓ | <span style="font-size: smaller;">Works pretty well, and very fast but you quickly reach rate limiting constrainst with free plan</span> |
|Groq | openai/gpt-oss-20b | ✓ | ✓ | ✓ | <span style="font-size: smaller;">Works pretty well, and very fast but you quickly reach rate limiting constrainst with free plan</span> |
|Groq | deepseek-r1-distill-llama-70b | ✓ | ✓ | ✓ | <span style="font-size: smaller;">Works pretty well, and very fast but you quickly reach rate limiting constrainst with free plan</span> |
|Groq | meta-llama/llama-4-scout-17b-16e-instruct | ✓ | ✓ | ✓ | <span style="font-size: smaller;">Works pretty well, and very fast but you quickly reach rate limiting constrainst with free plan</span> |
|Google | gemini-2.0-flash | ✓ | ✓ | ✓ | <span style="font-size: smaller;">Works pretty well, but it is often verbose</span> |
|Groq | llama-3.1-8b-instant | ✓ | ✗ | ✗ | <span style="font-size: smaller;">Works only for simple tasks</span> |

**3. Build Swarm:** Compile the workspace to ensure all dependencies are in sync:
```bash
cargo build --release
```

## **🚀 Quickstart through a simplified example scenario : Multi-Agent Orchestration Use Case:**

We created an easy to launch scenario to illustrate capabilities of the framework. It can be launched via script , from the root of swarm crate.

It is located in `documentation/use_case_combined_agents`. You will be able to find all the config files to support the scenario.

You should see logs from three MCP servers, three A2A agents, and the coordinating Full Agent. The script will then send three requests, and you will see the full agent orchestrate the response.

```bash
# The simple scenario has three domain driven agents and one that orchestrates 
# Domain driven agents can have their own LLM. In defined scenario we use gemini
export LLM_A2A_API_KEY=<YOUR-GEMINI-API-KEY>
# MCP Runtime can have their own LLM to manage interaction with MCP servers. In defined scenario we also use gemini
export LLM_MCP_API_KEY=<YOUR-GEMINI-API-KEY>
# Orchestrator agent can also have its own llm  LLM to manage interaction with other agents. In defined scenario we also use gemini
export LLM_FULL_API_KEY=<YOUR-GEMINI-API-KEY>
# This is the command line TO BE LAUNCHED FROM SWARM root, to launch the scenario
./documentation/use_case_combined_agents/run_all_commands.sh 
```
We hope that this quickstart will enable you to build your own scenario very easily

You can then deep dive in `documentation/use_case_combined_agents` to inspect configuration files, and launch script to understand how agents are connected together


## **💡 Core Components of Swarm**

Swarm is built around three key intelligent agent components:

*   **A2A Agent (Agent-to-Agent Protocol):** 🗣️ A simple, autonomous agent capable of direct communication with other A2A agents. It can also integrate an MCP Runtime for external interactions and connect to its own LLM.
*   **MCP Runtime (Model Context Protocol):** 🛠️ A powerful runtime that enables agents to interact with external services and data sources. Imagine your agents querying a weather API or accessing a database!
*   **Full Agent:** 🧠 An advanced A2A agent that acts as an orchestrator. It connects to various other A2A agents and an MCP server, understands their available skills and tools, creates a plan based on a user request, and executes it. Because a Full Agent is also an A2A agent, this allows for the creation of complex, hierarchical agent swarms where agents can be a part of multiple orchestration layers.


## **⚙️ Configuration Details**

The `configuration` directory is your go-to for customizing agent behavior. Here's a quick overview:

| File Name                | Purpose                                                                                |
| :----------------------- | :------------------------------------------------------------------------------------- |
| `agent_a2a_config.toml`  | Configures A2A simple agents, including optional embedded MCP agents                   |
| `agent_mcp_config.toml`  | Configures the MCP runtime settings.                                                   |
| `agent_full_config.toml` | Configures the Full Agent, including capabilities of using tools and skills                                                                                                              |

**LLM Models:** We've found `qwen/qwen3-32b` to be highly effective, especially for MCP interactions. Each agent can connect to its own LLM. URLs are parameterized in the config files, while API keys need to be injected at runtime as environment variables.

*(To enable MCP features, an MCP server must be running. You can find one in the `examples` for testing.)*

## **🚀 Launching Your Agents Manually or Programmatically**

Getting your Swarm agents up and running is straightforward. For maximum flexibility and programmatic control, you can launch individual agents directly. Remember to set the required API key environment variables (e.g., `LLM_A2A_API_KEY`, `LLM_FULL_API_KEY`, `LLM_MCP_API_KEY`) *before* running these commands.

*   **Simple Standalone A2A Agent Server:** Your individual intelligent assistant. You can run multiple A2A agents, each with its own configuration. (The example A2A agent embeds an MCP agent.)

    ```bash
    # Run compiled binary.
    # LLM_A2A_API_KEY: API Key for the A2A agent's LLM.
    # LLM_MCP_API_KEY: Optional API Key for the embedded MCP Runtime's LLM (can be the same as A2A).
    # Both API keys must be compatible with llm_url defined in the config file.
    # You can define log level (default is "warn").
    ./target/release/simple_agent_server --config-file "configuration/agent_a2a_config.toml" --log-level "warn"
    ```

    To emphasize how simple it is to use Swarm to launch an agent programmatically, here is a code snippet:
    ```rust
    use configuration::AgentA2aConfig;
    use a2a_agent_backbone::a2a_agent_logic::server::SimpleAgentServer;

    #[tokio::main]
    async fn main() -> Result<(), Box<dyn std::error::Error>> {
        // Load A2A config file
        let agent_a2a_config = AgentA2aConfig::load_agent_config("configuration/agent_a2a_config.toml")
            .expect("Incorrect A2A config file");
      
        // Create the agent server
        let server = SimpleAgentServer::new(agent_a2a_config).await?;

        println!("🌐 Starting HTTP server only...");
        server.start_http().await?;

        Ok(())
    }
    ```

*   **Full Agent Server (Orchestrator):** Ask the Full Agent to achieve a goal by providing a user query. This agent can connect to other A2A agents and MCP tools.

    ```bash
    # Run compiled binary.
    # LLM_FULL_API_KEY: API Key for the Full Agent's LLM.
    # LLM_MCP_API_KEY: Optional API Key for the embedded MCP Runtime's LLM (can be the same LLM as Full Agent).
    # Both API keys must be compatible with llm_url defined in the config file.
    # You can define log level (default is "warn").
    ./target/release/full_agent_server --config-file "configuration/agent_full_config.toml" --log-level "warn"
    ```

    To emphasize how simple it is to use Swarm to launch a full agent programmatically, here is a code snippet:
    ```rust
    use configuration::AgentFullConfig;
    use a2a_full_backbone::a2a_full_server::full_server::FullAgentServer;

    #[tokio::main]
    async fn main() -> Result<(), Box<dyn std::error::Error>> {
        // Load Full agent config file
        let agent_full_config = AgentFullConfig::load_agent_config("configuration/agent_full_config.toml")
            .expect("Incorrect Full Agent config file");
        
        // Create the full agent server
        let server = FullAgentServer::new(agent_full_config).await?;
    
        println!("🌐 Starting HTTP server only...");
        server.start_http().await?;
        
        Ok(())
    }
    ```

## **🔬 Under the Hood: Swarm.rs Crate Breakdown**

The Swarm project is composed of several specialized sub-crates, each serving a distinct purpose:

*   `agent_discovery_service`: An optional HTTP service where agents can register themselves. It exposes an endpoint to list all available agents, facilitating dynamic discovery.
*   `agent_memory_service`: A Work In Progress capability for all agents to share their interactions and ensure better coordination.
*   `configuration`: Contains Default Location of Swarm configuration files, making it easy to customize agent behavior.
*   `a2a_agent_backbone`: Provides the foundational code for a simple domain agent. It can incorporate an MCP runtime for external interactions and connect to its own LLM.
*   `a2a_full_backbone`: The core of the orchestrator Agent. It connects to declared domain agents and an MCP server, understands their skills and tools, creates, and executes plans to achieve your goals. It also connects to its own LLM.
*   `llm_api`: Offers a convenient interface for interacting with various Large Language Models via an OpenAI-compatible API.
*   `mcp_agent_backbone`: A runtime designed to be integrated into an A2A agent, granting it the capability to connect to an external set of tools via an MCP server. It can be connected to its own LLM.
*   `documentation`: Contains a series of example of toml config file for all sort of agents ( weather forecast, customer domain, web scraper, joke telling,...).
*   `examples`: Contains multiple illustrative examples to help you interact with Swarm:
    *   `mcp_agent_endpoint`: A testing utility for an MCP runtime to receive and process requests through an MCP server.
    *   `a2a_agent_endpoint`: A testing utility to interact directly with an A2A agent via a REST API. Also includes a simple UI for testing your configuration.
    *   `mcp_server`: A basic MCP server exposing three tools, primarily for testing purposes. You can start separately one instance on one port with one of these three tools ( weather, customer, url scraping), or the three tools together.

## **🗺️ Road Ahead & How You Can Contribute**

Swarm is currently a project for discovery and exploration! While not production-ready, it's a fantastic playground for understanding how these protocols can be combined to build powerful agentic systems. We are actively enhancing its capabilities and robustness.

We're continuously working on improvements, including:

*   **Memory & State Management** Capability to store all request from all agents, and to facilitate communication through the agent network.
*   **Dynamic Discovery:** Enable agents to understand dynamically what relevant skills are available to define an execution plan.
*   **Context Enrichment:** Enrich the request with relevant context, to make a plan more accurate.
*   **Evaluation:** Evaluation of the outcome of the swarm of agents, to enable distributed iterative decision processing ( test and learn).
*   **Identity Management:** Use Self Sovereign Identity concept to enable agent to have its own proven identity.
*   **Code Refactoring:** Continuously improving code clarity and maintainability.
*   **Unit Tests:** Enhancing robustness and reliability across the codebase.
*   **Scalable & Asynchronous Communication** Leveraging Tokio to enable agents to process requests from multiple clients in parallel.
*   **Improved UI** Improve the UI to interact with any kind of agent.
*   **Test with Llama.cpp Server** Test with llama.cpp server so that we can connect all kind of fine tuned models for single domain agents.


We're eager for your comments, suggestions, and contributions! 
Whether it's a bug report, a feature idea, or a pull request, your input helps shape the future of Swarm. 

You an also contribute by suggesting configuration files that you have been using to support your use case, as well as models you have been using for your experimentations

Join our community and help us build the next generation of intelligent agent orchestration!

## **⭐ Show Your Support**

If you find Swarm useful or interesting, please consider starring our repository! Your support helps us continue to develop and improve this project.

## **🙏 Special Thanks**

We heavily rely on and appreciate the fantastic work of these actively developed crates:

*   **MCP Protocol:** [https://github.com/modelcontextprotocol/rust-sdk](https://github.com/modelcontextprotocol/rust-sdk)
*   **A2A Protocol:** [https://github.com/EmilLindfors/a2a-rs](https://github.com/EmilLindfors/a2a-rs)

*Note: Due to their active development, we might occasionally pin to specific commits in `Cargo.toml` to maintain stability.*