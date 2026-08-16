# 🚀 Swarm: Intelligent Multi-Agent Framework & Model Gateway in Rust 🦀

> **Swarm** is a high-performance Rust framework designed to build, coordinate, and orchestrate intelligent AI agents. It operates in **two distinct modes**:
>
> 1. **🤖 Multi-Agent Orchestration & Tool Execution Mode**: Orchestrates collaborative agent networks (Planner, Executor, Domain Agents) with real-time MCP tools, service discovery, shared memory, and optional LLM-as-a-Judge evaluation.
> 2. **🌐 Model Gateway Server Mode**: Serves as a unified high-performance gateway supporting standard OpenAI-compatible `/v1/chat/completions` (for IDEs and extensions) and stateful multi-turn `/v1/responses` (Open Responses standard).

---

## 🧭 Swarm at a Glance: Two Operating Modes

```
+--------------------------------------------------------------------------------------------------+
|                                        SWARM MODES                                               |
+--------------------------------------------------------------------------------------------------+
|                                                                                                  |
|   MODE 1: MULTI-AGENT & MCP ORCHESTRATION               MODE 2: MODEL GATEWAY SERVER             |
|   (kickstart/)                                          (gateway_kickstart/)                     |
|                                                                                                  |
|   • Planner Agent (Dynamic plan generation)             • POST /v1/chat/completions (OpenAI)     |
|   • Executor Agent (Workflow DAG execution)             • POST /v1/responses (Open Responses)   |
|   • Domain Specialists with MCP Tool integration        • Stateful multi-turn turn chaining      |
|   • Discovery (4000) & Memory (5000) services           • Outbound model adaptors (Groq, Gemini) |
|   • Optional LLM-as-a-Judge self-correction             • High-throughput lock-free cache        |
|                                                                                                  |
+--------------------------------------------------------------------------------------------------+
```

---

## 📦 Prerequisites

1. **Install Rust**: [rust-lang.org](https://www.rust-lang.org/tools/install).
2. **Set your Groq API Key**:
   ```bash
   export GROQ_API_KEY="gsk_your_groq_api_key_here"
   ```

---

## 🤖 Mode 1: Multi-Agent & MCP Orchestration

In this mode, Swarm coordinates specialized agents that dynamically collaborate to solve user requests using live tools:

- **Planner Agent (Port 8280)**: Analyzes the user request and dynamically constructs an execution workflow.
- **Executor Agent (Port 9580)**: Executes each step in the workflow graph with dependency resolution.
- **Domain Agent (Port 8080)**: Specialist agent equipped with the **MCP Runtime** to execute tools (Weather, Customer, Wikipedia search, Scraping).
- **MCP Server (Port 8000)**: Model Context Protocol SSE server providing live tools.
- **Discovery (Port 4000) & Memory (Port 5000)**: Centralized agent registry and conversational context store.

### Quickstart (Mode 1)

```bash
cd swarm

# 1. Launch all agents, MCP server, and infrastructure services:
./kickstart/01_launch_all.sh

# 2. Run a sample query (e.g., Live Weather via MCP):
./kickstart/02_test_weather_query.sh "What is the current weather in Boston ?"

# 3. Stop all background processes when done:
./kickstart/03_terminate_all.sh
```

All configurations for Mode 1 are located in [`kickstart/config_files/`](kickstart/config_files).

---

## 🌐 Mode 2: Standalone Model Gateway Server

In this mode, `swarm_server` acts as a unified model gateway for client applications, IDEs, and developer tools:

- **OpenAI-Compatible Chat Completions (`POST /v1/chat/completions`)**: Backward-compatible with standard developer tools, IDE extensions, and OpenAI SDKs.
- **Open Responses Standard (`POST /v1/responses`)**: Modern stateful API with multi-turn conversation chaining (`previous_response_id`) and SSE streaming.

### Quickstart (Mode 2)

```bash
cd swarm

# 1. Launch the standalone Gateway Server (port 8080):
./gateway_kickstart/01_launch_gateway.sh

# 2. Test Chat Completions:
./gateway_kickstart/02_test_chat_completions.sh "Explain Swarm architecture in 2 sentences"

# 3. Test Stateful Multi-Turn Open Responses:
./gateway_kickstart/03_test_open_responses.sh

# 4. Stop the Gateway Server:
./gateway_kickstart/04_terminate_gateway.sh
```

---

## 📁 Directory Structure

```
swarm/
├── kickstart/                 # Mode 1: Multi-Agent & MCP Launch Suite
│   ├── 01_launch_all.sh
│   ├── 02_test_weather_query.sh
│   ├── 03_terminate_all.sh
│   ├── README.md
│   └── config_files/          # Localized agent & MCP configurations
│
├── gateway_kickstart/         # Mode 2: Standalone Gateway Launch Suite
│   ├── 01_launch_gateway.sh
│   ├── 02_test_chat_completions.sh
│   ├── 03_test_open_responses.sh
│   ├── 04_terminate_gateway.sh
│   └── README.md
│
├── basic_agent/               # Specialist domain agent embedding MCP runtime
├── planner_agent/             # Workflow planner and orchestrator
├── executor_agent/            # Workflow graph executor
├── agent_factory/             # Dynamic agent instantiation runtime
└── examples/                  # Example MCP servers and standalone runners
```

---

## 💡 Core Architecture

* **`swarm_commons`**: Shared core traits, A2A interaction protocols, LLM adapters, and state stores.
* **`swarm_services`**: Microservices providing Agent Discovery, Shared Memory, and LLM-as-a-Judge Evaluation.
* **`swarm`**: Core orchestration engines, specialist agents, and the unified gateway server.

---

## 🗺️ Roadmap & Contributing

- [x] Multi-Agent dynamic workflow planning and execution.
- [x] Model Context Protocol (MCP) tool integration.
- [x] Open Responses (`/v1/responses`) stateful turn chaining.
- [x] OpenAI-compatible (`/v1/chat/completions`) endpoint.
- [ ] Multi-provider load balancing and fallback routing (Gemini, Groq, OpenAI, Ollama).
- [ ] Distributed Self-Sovereign Identity (SSI) agent authentication.

We welcome contributions! Feel free to open issues or pull requests.

---

## ⭐ Show Your Support

If you find Swarm useful, please consider starring our repository! Your support helps us grow.

---

## 🙏 Special Thanks

We rely on the fantastic work of these actively developed crates:

* **MCP Protocol:** [https://github.com/modelcontextprotocol/rust-sdk](https://github.com/modelcontextprotocol/rust-sdk)
* **A2A Protocol:** [https://github.com/EmilLindfors/a2a-rs](https://github.com/EmilLindfors/a2a-rs)
