# 🚀 Build, Connect, and Manage Intelligent Agents with Flexible Workflows in Rust 🦀

## **Why Swarm?**

**Building complex, multi-agent systems and dynamic workflows is hard.** You need to handle communication, manage sequences of tasks, and integrate external tools. **Swarm makes it easy.**

Swarm is a Rust-based framework that allows you to build, connect, and manage a network of specialized, intelligent agents through flexible workflows. Think of it as a central nervous system for your digital workforce, capable of executing predefined task sequences or generating dynamic plans on the fly. Whether you need a customer service bot that orchestrates multiple tasks and agents, or a data analyst that can delegate complex data processing workflows, Swarm provides the backbone for them to collaborate seamlessly and efficiently.

## **🌐 How It Works**

**Swarm empowers you to define and execute powerful workflows involving multiple specialized agents and tools.** At its core, Swarm now focuses on **Workflow-Driven Agents** that can either execute predefined, static workflows loaded from a file, or dynamically generate new workflows based on the user's request and available resources.

*   **Domain Agents (Specialists):** These are single-purpose, domain-specific agents, each with access to their own LLM and a dedicated set of tools (e.g., customer care, weather forecasting, data retrieval).
*   **Workflow Agent (The Conductor):** This advanced agent leverages the `workflow_management` capabilities to process user requests. It can:
    *   **Execute Static Workflows:** Load and run predefined workflows from JSON files, ensuring consistent execution of multi-step processes.
    *   **Generate Dynamic Workflows:** Analyze user requests and available agent skills and tools to create a tailored execution plan on the fly, delegating tasks and monitoring their completion.
*   **Workflow Management Runtime:** This core component provides the infrastructure for defining, parsing, and executing complex graphs of tasks and agent interactions, whether they are static or dynamically generated.

<p align="center" width="60%">
    <img width="60%" src="./documentation/illustrations/Simple_Representation_Architecture.png">
</p>
*(Note: Workflow Agent, capable of both orchestrating and executing structured workflows is at the core of the project)*

For a detailed description of Swarm's core components, refer to the "Core Components of Swarm" section below.

*(Note: An optional MCP Server is provided in the `examples` section for testing purposes.)*


## **🚀 Quickstart: Your First Swarm in 5 Minutes**

Follow these steps to get a multi-agent system running.

### Prerequisites

1.  **Install Rust**: If you don't have it, get it from [rust-lang.org](https://www.rust-lang.org/tools/install).
2.  **Get an LLM API Key**: Swarm agents need an LLM to think. We recommend starting with a free plan from [Groq](https://console.groq.com/keys) or [Google AI Studio (for Gemini)](https://aistudio.google.com/app/apikey). For a detailed comparison of the models we've tested, see our [Model Comparison Guide](./documentation/model_comparison/quick_comparison.md).

### Step 1: Clone and Build the Project

```bash
git clone https://your-repo-url/swarm.git
cd swarm
cargo build --release
```

### Step 2: Set Your API Keys

The quickstart scenario uses both Gemini and Groq API by default. Export your API key as an environment variable:

```bash
# Replace <YOUR-GEMINI-API-KEY> and <YOUR-GROQ-API_KEY> with your actual key
# The below are examples. Each script will tell you which one to use
export LLM_A2A_API_KEY=<YOUR-GROQ-API-KEY>
export LLM_MCP_API_KEY=<YOUR-GROQ-API-KEY>
export LLM_WORKFLOW_API_KEY=<YOUR-GROQ-API-KEY>
export LLM_JUDGE_API_KEY=<YOUR-GROQ-API-KEY>
```

### Step 3: Choose Your Adventure: Dive In a Demo !


--- 
**_The Workflow Demo_** 

If you're interested in structured, sequential tasks and the power of workflow management, this demo is for you. It focuses on executing a workflow that manages a specific, multi-step process from start to finish. The demo runs a mix of direct tool calls and actions delegated to agents.

There are two variations for running this demo:

*   **Static Workflow (Default):** This executes a predefined workflow.

    ```bash
    # This command must be run from the root of the swarm project
    sh ./documentation/demo_workflow_management/run_all_commands.sh
    ```

*   **Dynamic Workflow Generation:** This dynamically generates a plan based on user queries and available resources. Use the `--dynamic-generation` flag.

    ```bash
    # This command must be run from the root of the swarm project
    sh ./documentation/demo_workflow_management/run_all_commands.sh --dynamic-generation
    ```

This is an excellent choice if you're looking to build agents for automated, step-by-step processes or want to understand how workflows are executed.

**Cleaning Up After the Demo:**
After running the demo, you can stop all the launched services using the provided termination script:

```bash
# This command must be run from the root of the swarm project
sh ./documentation/demo_workflow_management/terminate_all_agents_process.sh
```

---


**Congratulations, you've just run your first swarm!**

You'll see logs from all agents as the agent processes three sample requests.

For a deeper look, you can access explanations of the expected logs and inspect the configuration files `documentation/demo_combined_agents directory`.

---


Feel free to inspect the configuration files for the demo to get a better understanding of how they work.


## **💡 Core Components of Swarm**

Swarm is built around several key intelligent agent and workflow components, including a component acting as a Judge to evaluate their results:

*   **Basic Domain Agent (The Specialist):** 🗣️ This is your workhorse. An agent designed to be an expert in a single domain, like "weather forecasting" or "database queries."
*   **Workflow Agent (The Conductor):** 🧠 This agent is at the heart of orchestrating complex operations. It can either execute predefined workflows loaded from JSON files or dynamically generate a sequence of tasks and agent interactions based on a user's request and available resources. It leverages the `workflow_management` runtime to ensure seamless execution.
*   **Workflow Management Runtime:** 🔗 A powerful, flexible engine for defining, parsing, and executing complex workflows. It enables the creation of directed acyclic graphs (DAGs) that can integrate multiple agents, external tools, and conditional logic. This runtime can operate independently or be embedded within a `Workflow Agent` for dynamic plan generation and execution.
*   **MCP Runtime (Model Context Protocol):** 🛠️ A powerful runtime that enables agents to interact with external services and data sources. Imagine your agents querying a weather API or accessing a database!
*   **LLM as a Judge:** 🛠️ An LLM that will evaluate the outcome of agent and workflow executions. We advise that you use a different LLM/Model than the one you use for agents and MCP. You can try it yourself through the demos we presented above. The evaluation data can be an interesting input for training/fine-tuning LLMs to improve the accuracy of their answers or the relevance of the tools they use.


## **⚙️ Configuration Details**

The `configuration` directory is your go-to for customizing agent and workflow behavior. Here's a quick overview:

| File Name | Purpose |
| :----------------------- | :------------------------------------------------------------------------------------- |
| `agent_basic_config.toml`| Configures simple domain agents, including optional embedded MCP agents. |
| `agent_workflow_config.toml`| Configures the Workflow Agent, defining its LLM, skills, and optional embedded MCP runtime, along with its ability to execute static or dynamic workflows. |
| `mcp_runtime_config.toml` | Configures the MCP runtime settings, used by agents that integrate external tools. |
| `agent_judge_config.toml`| Configures the judge agent that will be used for evaluation services. |

**LLM Models:** Each agent can connect to its own LLM. URLs are parameterized in the config files, while API keys need to be injected at runtime as environment variables.

*(To enable MCP features, an MCP server must be running and configured in toml files. You can find one in the `examples` for testing.)*

## **🚀 Launching Your Agents Manually or Programmatically**

Getting your Swarm agents and workflows up and running is straightforward. For maximum flexibility and programmatic control, you can launch individual agents or the standalone workflow orchestrator directly. Remember to set the required API key environment variables (e.g., `LLM_A2A_API_KEY`, `LLM_FULL_API_KEY`, `LLM_MCP_API_KEY`) *before* running these commands.

*   **Basic Domain Agent:** Your individual intelligent assistant specialized in a single domain

    Via Script :

    ```bash
    # Run compiled binary.
    # LLM_A2A_API_KEY: API Key for the A2A agent's LLM.
    # LLM_MCP_API_KEY: Optional API Key for the embedded MCP Runtime's LLM (can be the same as A2A).
    # Both API keys must be compatible with llm_url defined in the config file.
    # You can define log level (default is "warn").
    ./target/release/basic_agent_launch --config-file "configuration/agent_basic_config.toml" --log-level "warn"
    ```

    Or programmatically, in just a few lines of code :

    ```rust
    // load config file
    let basic_agent_config = AgentConfig::load_agent_config(&args.config_file);

    // Create the modern server, and pass the runtime elements
    let server = AgentServer::<BasicAgent>::new(basic_agent_config.expect("Incorrect Basic Agent config file")).await?;

    println!("🌐 Starting HTTP server only...");
    server.start_http().await?;
    ```


*   **Workflow Agent:** This agent can execute predefined or dynamically generated workflows, connecting to other agents and MCP tools to achieve its goals.

    Via Script :

    ```bash
    # Run compiled binary.
    # LLM_WORKFLOW_API_KEY: API Key for the Workflow Agent's LLM.
    # LLM_MCP_API_KEY: Optional API Key for the embedded MCP Runtime's LLM (can be the same LLM as Workflow Agent).
    # Both API keys must be compatible with llm_url defined in the config file.
    # You can define log level (default is "warn").
    ./target/release/launch_agent_workflow --config-file "configuration/agent_workflow_config.toml" --log-level "warn"
    ```
    Or programmatically, in just a few lines of code :

    ```rust
    // load config file
    let workflow_agent_config = AgentConfig::load_agent_config(&args.config_file);

    // Set up an agent
     let agent = WorkFlowAgent::new(workflow_agent_config.clone(), evaluation_service, memory_service, discovery_service.clone(), workflow_runners).await?;

    // Set up an agent server
    let server = AgentServer::<WorkFlowAgent>::new(workflow_agent_config, agent, discovery_service).await?;
   
    println!("🌐 Starting HTTP server only...");
    server.start_http().await?;
    ```

*   **Workflow Orchestrator (Standalone):** A dedicated service for managing and executing complex workflows independently of a specific agent.

    Via Script :

    ```bash
    # Run compiled binary.
    # You can define log level (default is "warn").
    ./target/release/workflow_management --log-level "warn"
    ```
 

## **🔬 Under the Hood: Swarm.rs Crate Breakdown**

The Swarm project is composed of several specialized sub-crates, each serving a distinct purpose:

*   `agent_discovery_service`: An HTTP service where agents can register themselves, enabling dynamic discovery of available agents and their capabilities for workflow planning.
*   `agent_memory_service`: A capability for all agents to share their interactions and ensure better coordination within workflows.
*   `agent_evaluation_service`: A capability for all agents to have their performance evaluated (LLM as a Judge Model), which can be integrated into workflow feedback loops.
*   `configuration`: Contains default locations of Swarm configuration files and prompts, making it easy to customize agent and workflow behavior.
*   `agent_core`: Contains foundational elements to launch an agent, interact with an agent, and enable request processing capabilities for all agent types.
*   `basic_agent`: Provides business logic to launch a basic, specialist agent.
*   `workflow_agent`: Provides the business logic to launch a Workflow Agent, enabling it to execute static or dynamically generated workflows.
*   `workflow_management`: The core runtime for defining, parsing, and executing graphs of agents and tasks. This crate allows for loading static workflows from files or being leveraged by a `Workflow Agent` to dynamically construct execution plans.
*   `mcp_runtime`: A runtime to connect to a ModelContextProtocol Server, designed to be embedded into an agent for interacting with external services and tools.
*   `llm_api`: Offers a convenient interface for interacting with various Large Language Models via an OpenAI-compatible API.
*   `documentation`: Contains a series of example configuration files and guides for various agent and workflow setups.
*   `examples`: Contains multiple illustrative examples to help you interact with Swarm, including demonstrations of static and dynamic workflow execution.

## **🗺️ Road Ahead & How You Can Contribute**

Swarm is currently a project for discovery and exploration! While not production-ready, it's a fantastic playground for understanding how these protocols and workflow management can be combined to build powerful agentic systems. We are actively enhancing its capabilities and robustness.

We're continuously working on improvements, including:

*   **Enhanced Workflow Features:** Continuously improving the flexibility and power of the Workflow Management Runtime, including more advanced conditional logic, error handling, and parallel execution.
*   **Advanced Dynamic Discovery:** Further enhancing agents' ability to dynamically discover and integrate relevant skills and tools for workflow generation.
*   **Context Enrichment:** Enriching requests with relevant context to make workflow plans more accurate and robust.
*   **Identity Management:** Implementing Self-Sovereign Identity concepts to enable agents to have their own verifiable identities within the swarm.
*   **Improved Observability:** Enhancing logging, tracing, and monitoring capabilities for better insight into workflow execution.
*   **Scalable & Asynchronous Communication:** Improving the underlying communication mechanisms for higher throughput and responsiveness.
*   **Improved UI:** Developing a more intuitive user interface for defining, visualizing, and managing workflows.
*   **Broader LLM Integration:** Expanding compatibility and testing with a wider range of LLMs and fine-tuned models for specialist agents.
*   **Code Refactoring & Unit Tests:** Ongoing efforts to improve code clarity, maintainability, and test coverage across the codebase.

## **🤝 How to Contribute**

We welcome contributions of all kinds! Whether you're a developer, a writer, or just have a great idea, we'd love to have you.

Here are a few ways you can help:

1.  **Report Bugs**: If you find a bug, please [open an issue](https://github.com/fcn06/swarm/issues) and provide as much detail as possible.
2.  **Suggest Features**: Have an idea for a new feature? We'd love to hear it! [Open an issue](https://github.com/fcn06/swarm/issues) to start the discussion.
3.  **Submit Pull Requests**: If you're ready to contribute code, please fork the repository and submit a pull request. We recommend starting with issues labeled `good first issue`.
4.  **Improve the Documentation**: If you see an area where the documentation could be better, please don't hesitate to make the change and submit a pull request.

Before contributing, please read our (coming soon) `CONTRIBUTING.md` file for more detailed guidelines.


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

*Note: Due to their active development, we pin to recent ( but fixed revision ) in `Cargo.toml` to maintain stability.*