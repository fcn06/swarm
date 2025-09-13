# 🚀 Swarm: A Framework for Building and Orchestrating Intelligent Agents in Rust 🦀

> **Swarm** is a Rust framework for creating and managing a network of specialized, intelligent agents that collaborate through flexible workflows. It acts as a central nervous system for your digital workforce, allowing you to orchestrate complex tasks, either by executing predefined plans or by generating them dynamically.

<p align="center" width="60%">
    <img width="60%" src="./documentation/illustrations/Simple_Representation_Architecture.png">
</p>

## **Why Swarm?**

Building multi-agent systems is complex. You need to manage communication, sequence tasks, and integrate external tools. Swarm simplifies this by providing the backbone for agents to collaborate seamlessly, allowing you to focus on building intelligent solutions, not on the plumbing.

## **How It Works**

Swarm's architecture is centered around a "conductor" and "specialist" model:

*   **Workflow Agent (The Conductor):** This is the orchestrator. It takes a user's request and determines the best course of action. It can either follow a predefined plan (a "static workflow") from a JSON file or, more powerfully, create a new plan on the fly (a "dynamic workflow") by assessing the skills of the available agents.
*   **Domain Agents (The Specialists):** These are your workhorse agents, each an expert in a specific domain (e.g., customer care, weather forecasting, data analysis). They have their own LLM and a dedicated set of tools to get their job done.

This structure allows you to build sophisticated systems where a central agent delegates tasks to a team of specialized agents.

## **🚀 Quickstart: Launch Your First Swarm in 5 Minutes**

### Prerequisites

1.  **Install Rust**: If you don't have it, get it from [rust-lang.org](https://www.rust-lang.org/tools/install).
2.  **Get an LLM API Key**: Swarm agents need an LLM to think. We recommend starting with a free plan from [Groq](https://console.groq.com/keys) or [Google AI Studio (for Gemini)](https://aistudio.google.com/app/apikey).

### Step 1: Clone and Build the Project

```bash
git clone https://your-repo-url/swarm.git
cd swarm
cargo build --release
```

### Step 2: Set Your API Keys

The quickstart scenario uses the Groq API by default. Export your API key as an environment variable:

```bash
# Replace <YOUR-GROQ-API-KEY> with your actual key
export LLM_A2A_API_KEY=<YOUR-GROQ-API-KEY>
export LLM_MCP_API_KEY=<YOUR-GROQ-API-KEY>
export LLM_WORKFLOW_API_KEY=<YOUR-GROQ-API-KEY>
export LLM_JUDGE_API_KEY=<YOUR-GROQ-API-KEY>
```

### Step 3: Run the Demo

Our main demo showcases the power of workflow management, running a mix of direct tool calls and actions delegated to agents. You have three ways to run it:

*   **Dynamic Workflow Generation:** Dynamically generates a plan based on your query and the available agents. This showcases Swarm's ability to adapt.

    ```bash
    # This command must be run from the root of the swarm project
    sh ./documentation/demo_workflow_management/run_all_commands.sh --dynamic-generation
    ```

*   **Static Workflow (Default):** Executes a predefined workflow from a file. This is great for predictable, repeatable processes.

    ```bash
    # This command must be run from the root of the swarm project
    sh ./documentation/demo_workflow_management/run_all_commands.sh
    ```

*   **High-Level Plan Generation:** Generates only a high-level plan, without executing it. This is useful for evaluating the planning capabilities of your LLM.

    ```bash
    # This command must be in the root of the swarm project
    sh ./documentation/demo_workflow_management/run_all_commands.sh --high-level-plan
    ```

**To stop all services after the demo, run:**

```bash
# This command must be in the root of the swarm project
sh ./documentation/demo_workflow_management/terminate_all_agents_process.sh
```

---

**Congratulations, you've just run your first swarm!**

---

## **💡 Core Components of Swarm**

Swarm is built from a set of modular components that work together:

*   **🗣️ Basic Domain Agent:** The **Specialist**. An agent designed to be an expert in a single domain, like "weather forecasting" or "database queries."
*   **🧠 Workflow Agent:** The **Conductor**. This agent orchestrates complex operations by executing predefined workflows or dynamically generating new ones. Each fulfillment of request is evaluated by LLM as a judge component, and potentially modify execution to improve fulfillment of user request.
*   **🔗 Workflow Management Runtime:** The **Engine**. This is the powerful, flexible engine that executes the workflows. It can run as part of a Workflow Agent or independently.
*   **🛠️ MCP Runtime (Model Context Protocol):** The **Bridge** to the outside world. This runtime enables agents to interact with external services and data sources, like a weather API or a database.
*   **⚖️ LLM as a Judge:** The **Evaluator**. An LLM that assesses the outcome of agent and workflow executions, providing a feedback loop for improvement.

## **🔬 Under the Hood: Crate Breakdown**

The Swarm project is composed of several specialized crates:

*   `agent_core`: Foundational elements for all agent types.
*   `basic_agent`: Business logic for the "specialist" domain agents.
*   `workflow_agent`: Business logic for the "conductor" workflow agent.
*   `workflow_management`: The core runtime for defining and executing workflows.
*   `mcp_runtime`: The runtime for connecting to external tools and services.
*   `llm_api`: A convenient interface for interacting with various Large Language Models.
*   `agent_discovery_service`: An HTTP service where agents can register themselves, allowing them to be discovered by the Workflow Agent.
*   `agent_memory_service`: A service allowing agents to share conversation history and context.
*   `agent_evaluation_service`: A service that uses an "LLM as a Judge" to evaluate agent performance.
*   `configuration`: Default configuration files and prompts.
*   `documentation`: Example configuration files and guides.
*   `examples`: Illustrative examples to help you interact with Swarm. In particular, there is a test MCP server with capability to scrape a url and search against wikipedia.

## **🗺️ Roadmap & Contributing**

Swarm is an active project, and we are constantly working on improvements. Our roadmap includes:

*   **Enhanced Workflow Features:** More advanced conditional logic, error handling, and parallel execution.
*   **Advanced Dynamic Discovery:** Better ways for agents to find and use the tools they need.
*   **Improved Observability:** Better logging, tracing, and monitoring.
*   **Broader LLM Integration:** Compatibility with a wider range of LLMs.

### **🤝 How to Contribute**

We welcome contributions! Whether you're a developer, a writer, or have a great idea, we'd love to have you. The best way to start is by opening an issue to discuss your ideas.

## **⭐ Show Your Support**

If you find Swarm useful, please consider starring our repository! Your support helps us grow.

## **🙏 Special Thanks**

We rely on the fantastic work of these actively developed crates:

*   **MCP Protocol:** [https://github.com/modelcontextprotocol/rust-sdk](https://github.com/modelcontextprotocol/rust-sdk)
*   **A2A Protocol:** [https://github.com/EmilLindfors/a2a-rs](https://github.com/EmilLindfors/a2a-rs)
