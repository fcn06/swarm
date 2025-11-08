# 🚀 Swarm: A Framework for Building and Orchestrating Intelligent Agents in Rust 🦀

> **Swarm** is a Rust framework for creating and managing a network of specialized, intelligent agents that collaborate through flexible workflows. It acts as a central nervous system for your digital workforce, allowing you to orchestrate complex tasks, either by executing predefined plans or by generating them dynamically.

<p align="center" width="60%">
    <img width="60%" src="./documentation/illustrations/Swarm_Agent_Factory_Illustration.png">
</p>

## **Why Swarm?**

Building multi-agent systems is complex. You need to manage communication, sequence tasks, and integrate external tools. Swarm simplifies this by providing the backbone for agents to collaborate seamlessly, allowing you to focus on building intelligent solutions, not on the plumbing.

---

## **How It Works**


Swarm's architecture is designed around a collaborative "conductor" and "specialist" model, facilitating efficient execution of complex tasks:

1.  **User Request & Planning (Planner Agent - The Conductor, Part 1):**
    *   A user initiates a request, which is received by the **Planner Agent** (or an **Agent Factory** can directly launch an agent to handle the request).
    *   The Planner Agent, acting as the primary orchestrator, determines the optimal course of action. It can:
        *   Load a predefined plan ("static workflow") from a JSON file.
        *   Dynamically generate a new plan ("dynamic workflow") by evaluating the capabilities of available **Domain Agents, Tools, or Tasks** (or by instructing an **Agent Factory** to instantiate the necessary agents).
    *   Once a plan is established, it is passed to the **Executor Agent**.

2.  **Execution & Evaluation (Executor Agent - The Doer; Planner Agent - The Conductor, Part 2):**
    *   The **Executor Agent** receives the plan from the Planner Agent and meticulously executes each step. This involves coordinating with **Domain Agents** and utilizing various tools through the **MCP Runtime**. The Executor Agent reports the outcome of the execution back to the Planner Agent.
    *   For **dynamic plans**, the **Planner Agent** then takes this outcome and, using an integrated **LLM-as-a-Judge** system via the **Evaluation Service**, critically assesses the execution's success. If the evaluation score is unsatisfactory, the Planner Agent can leverage this feedback to refine the plan, potentially regenerating and re-executing an improved workflow. This creates a powerful feedback loop for continuous improvement in dynamic scenarios.

3.  **Specialized Task Handling (Domain Agents - The Specialists):**
    *   **Domain Agents** are the core "workers," each specializing in a particular domain (e.g., customer care, weather forecasting, data analysis).
    *   Each Domain Agent is powered by its own Large Language Model (LLM) and equipped with a dedicated set of tools to accomplish its specific tasks, responding to directives from the Executor Agent.

This modular and iterative structure enables the creation of sophisticated multi-agent systems where a central orchestrator efficiently delegates, executes, and refines tasks through a team of specialized, intelligent agents.

---

## **🚀 Quickstart: Launch Your First Swarm in 5 Minutes**

### Prerequisites

1.  **Install Rust**: If you don't have it already, download and install it from [rust-lang.org](https://www.rust-lang.org/tools/install).
2.  **Get an LLM API Key**: Swarm agents require an LLM to function. We recommend obtaining a free API key from [Groq](https://console.groq.com/keys) or [Google AI Studio (for Gemini)](https://aistudio.google.com/app/apikey). It can also connect to a local llama.cpp openai compatible server instance

### Step 1: Clone and Build the Project

```bash
git clone https://github.com/fcn06/swarm.git
cd swarm
cargo build --release
```

### Step 2: Set Your LLM API Keys

The quickstart demo utilizes LLMs for various agent roles. For simplicity, you can use the *same* API key for all roles, especially when starting with Groq.

```bash
# Replace <YOUR-LLM-API-KEY> with your actual API key. 
# For the pre configured demo below, we use groq provider. ( https://groq.com/)
export LLM_A2A_API_KEY=<YOUR-LLM-API-KEY>       # For general Agent-to-Agent communication
export LLM_MCP_API_KEY=<YOUR-LLM-API-KEY>       # For Model Context Protocol interactions
export LLM_JUDGE_API_KEY=<YOUR-LLM-API-KEY>     # For the LLM-as-a-Judge evaluation service
export LLM_PLANNER_API_KEY=<YOUR-LLM-API-KEY>     # For the Planner Agent
```

### Step 3: Run the Demo

Our primary demo showcases Swarm's workflow management capabilities, integrating direct tool calls and delegated actions to specialized agents. You can run it in three modes:

*   **Dynamic Workflow Generation:** This mode dynamically generates an execution plan based on your query and the skills of available agents. It highlights Swarm's adaptive planning capabilities.
    ```bash
    # This command must be run from the root of the swarm project
    sh ./documentation/demo_planner_executor_management/run_all_commands.sh --dynamic-generation
    ```

*   **Static Workflow (Default):** This executes a predefined workflow loaded from a JSON file. Ideal for predictable and repeatable processes.
    ```bash
    # This command must be run from the root of the swarm project
    sh ./documentation/demo_planner_executor_management/run_all_commands.sh
    ```
As an illustration, here is what we can define in a JSON workflow :

<p align="center" width="20%">
    <img width="20%" src="./documentation/graph_visualizer/Graphical_Workflow_Representation.png">
</p>

Which is a simplified graphical representation of :

<p align="center" width="20%">
    <img width="20%" src="./documentation/graph_visualizer/Snapshot_json_workflow.png">
</p>

**Important: Stopping the Demo Services**
After running any of the above demos, it's crucial to stop all active agent processes to prevent resource consumption and conflicts.

```bash
# This command must be run from the root of the swarm project
sh ./documentation/demo_planner_executor_management/terminate_all_agents_process.sh
```

---

**Congratulations, you've just run your first swarm!**

---

## **🏭 Agent Factory: Programmatic Agent Launch**

Swarm provides an `AgentFactory` to dynamically launch and manage agents programmatically. This allows for creating flexible and scalable multi-agent systems where agents can be instantiated on-the-fly based on runtime conditions and specific needs.

The `AgentFactory` leverages the **MCP Runtime** to manage the lifecycle and communication of the agents it spawns. When an agent is launched via the factory, its configuration, including how it interacts with the Model Context Protocol (MCP), is defined. This enables newly created agents to immediately integrate into the broader Swarm ecosystem and communicate with other services and tools.

You can configure an agent's properties, such as its type, domain, name, description, and the LLM it uses, as well as its `mcp_runtime` settings, and then launch it with a single command.

Here’s a code snippet illustrating how to launch a "Basic_Agent" with `mcp_runtime` configuration:

```rust
      let agent_api_key = env::var("LLM_A2A_API_KEY").expect("LLM_A2A_API_KEY must be set");
     
    let factory_mcp_runtime_config = FactoryMcpRuntimeConfig::builder()
        .with_factory_mcp_llm_provider_url(LlmProviderUrl::Groq)
        .with_factory_mcp_llm_provider_api_key(agent_api_key.clone())
        .with_factory_mcp_llm_model_id("openai/gpt-oss-20b".to_string())
        .with_factory_mcp_server_url("http://localhost:8000/sse".to_string())
        .with_factory_mcp_server_api_key("".to_string())
        .build().map_err(|e| anyhow::anyhow!("Failed to build FactoryMcpRuntimeConfig: {}", e))?;

    let factory_agent_config = FactoryAgentConfig::builder()
        .with_factory_agent_url("http://127.0.0.1:8080".to_string())
        .with_factory_agent_type(AgentType::Specialist)
        .with_factory_agent_domains(AgentDomain::General)
        .with_factory_agent_name("Basic_Agent".to_string())
        .with_factory_agent_id("Basic_Agent".to_string())
        .with_factory_agent_description("An Agent that answer Basic Questions".to_string())
        .with_factory_agent_llm_provider_url(LlmProviderUrl::Groq)
        .with_factory_agent_llm_provider_api_key(agent_api_key)
        .with_factory_agent_llm_model_id("openai/gpt-oss-20b".to_string())
        .build().map_err(|e| anyhow::anyhow!("Failed to build FactoryAgentConfig: {}", e))?;

    agent_factory.launch_agent_with_mcp(&factory_agent_config,&factory_mcp_runtime_config,AgentType::Specialist).await?;

```

This capability is essential for creating adaptive systems that can scale their workforce based on the tasks at hand, ensuring proper integration and communication through the **MCP Runtime**.

---

## **💡 Other Examples**

You can interact with your agent system through a UI , and will find examples in [./documentation/Sample_Scenarios/scenario_ui.txt](./documentation/Sample_Scenarios/scenario_ui.txt)

<p align="center" width="60%">
    <img width="60%" src="./documentation/Sample_Scenarios/UI_Chat_Agent_System.png">
</p>

You can find other real world scenarios in [./documentation/Sample_Scenarios/scenario_1.txt](./documentation/Sample_Scenarios/scenario_1.txt)

You can find concrete example of workflow in [./documentation/Sample_Scenarios/mix_agent_tools_workflow.json](./documentation/Sample_Scenarios/mix_agent_tools_workflow.json)

---

## **💡 Core Components of Swarm**

Swarm is composed of several modular and interconnected components that work together to enable intelligent agent orchestration:

*   **🗣️ Domain Agents (The Specialists):** These are specialized agents, each acting as an expert in a particular domain (e.g., weather forecasting, database queries, customer care). They execute specific tasks as directed by the Executor Agent.
*   **✍️ Planner Agent (The Architect):** This specialized agent is the first part of the "Conductor." It focuses on generating detailed, step-by-step execution plans or workflows based on a high-level goal, which are then passed to the Executor Agent.
*   **🏃 Executor Agent (The Doer):** Completing the "Conductor" role, this agent takes an execution plan from the Planner, carries out the individual tasks by interacting with tools and other agents, and integrates with the LLM-as-a-Judge system for continuous evaluation and potential workflow refinement.
*   **🏭 Agent Factory (The Spawner):** This component allows for the dynamic, programmatic creation and management of agent instances at runtime, facilitating scalable and adaptive multi-agent systems.
*   **🔗 Workflow Management Runtime (The Engine):** This flexible core is responsible for defining, validating, and executing multi-agent workflows and plans. It is the underlying mechanism leveraged by the Executor Agent to manage the execution of planned tasks.
*   **🛠️ MCP Runtime (Model Context Protocol) (The Bridge):** This component facilitates seamless agent interaction with external services, tools, and diverse data sources, effectively extending the agents' capabilities to the outside world.
*   **⚖️ LLM as a Judge (The Evaluator):** An autonomous Large Language Model-based service that critically assesses the performance and outcomes of both individual agent actions and complete workflow executions, providing essential feedback for iterative improvement.
*   **🔌 Agent Service Adapters (The Communicators):** These client-side implementations provide the necessary interfaces for agents to interact with core Swarm services like discovery, memory, and evaluation, ensuring robust inter-agent and inter-service communication.
*   **🗳️ Agent Discovery Service:** An HTTP service enabling agents to register themselves and discover other available agents.
*   **🧠 Agent Memory Service:** A service designed to manage and share conversational history and contextual information among agents.
*   **📊 Agent Evaluation Service:** Implements the "LLM as a Judge" functionality to evaluate agent and workflow execution outcomes.
*   **⚙️ Configuration:** Stores default configuration files, prompts, and agent definitions.
*   **📚 Documentation:** Contains example configuration files, guides, and architectural illustrations.
*   **💡 Examples:** Provides illustrative code examples, including a test MCP server with web scraping and Wikipedia search capabilities.

---

## **🗺️ Roadmap & Contributing**

Swarm is an active project, and we are constantly working on improvements. Our roadmap includes:

*   **Finalize and Fine Tune Agent Factory** Inject properly evaluation, memory, discovery services.
*   **Make discovery service using embeddings, and use proper database** So that we can inject in agents the more relevant tools, tasks and domain agents in the prompt of the planner. This will be done in a separate project
*   **Enhanced Workflow Features** More advanced conditional logic.
*   **Improved Observability:** Better logging, tracing, and monitoring.
*   **Broader LLM Integration:** Compatibility with a wider range of LLMs.

### **🤝 How to Contribute**

We welcome contributions! Whether you're a developer, a writer, or have a great idea, we'd love to have you. The best way to start is by opening an issue to discuss your ideas.

---

## **⭐ Show Your Support**

If you find Swarm useful, please consider starring our repository! Your support helps us grow.
We plan to work on feature rich discovery, context/memory and evaluation services in a separate project

---

## **🙏 Special Thanks**

We rely on the fantastic work of these actively developed crates:

*   **MCP Protocol:** [https://github.com/modelcontextprotocol/rust-sdk](https://github.com/modelcontextprotocol/rust-sdk)
*   **A2A Protocol:** [https://github.com/EmilLindfors/a2a-rs](https://github.com/EmilLindfors/a2a-rs)
