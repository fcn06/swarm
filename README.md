# 🚀 Swarm: Intelligent Multi-Agent Framework & Model Gateway in Rust 🦀

[![Rust](https://img.shields.io/badge/Rust-1.88-orange?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue)](LICENSE)
[![MCP](https://img.shields.io/badge/MCP-SSE%20%2B%20Streamable-purple)](https://modelcontextprotocol.io)

> While experimenting with multi-agent systems, AI stacks often end up with two separate pieces of infrastructure: an orchestration layer for agents and tools, and a gateway layer for routing LLM requests.
>
> **Swarm** is an open-source AI orchestration framework and model gateway written in Rust that unifies both patterns around a single high-performance Tokio runtime—sharing provider abstractions, state management, and protocol contracts.

---

## 🧭 The Dual-Mode Architecture

Many AI architectures separate these concerns into a dedicated proxy for lightweight routing and a separate orchestrator for complex reasoning. Swarm unifies both patterns: it can run as a full agent orchestration stack or as a standalone LLM gateway without requiring two unrelated frameworks.

<p align="center">
  <img src="illustrations/swarm_dual_ai_framework.jpeg" alt="Swarm Dual AI Framework" width="60%" />
  <br />
  <em>Figure: Overview of Swarm's dual operating modes — Multi-Agent Orchestration &amp; Model Gateway Server.</em>
</p>

```plaintext
+--------------------------------------------------------------------------------------------------+
|                                        SWARM MODES                                               |
+--------------------------------------------------------------------------------------------------+
|                                                                                                  |
|   MODE 1: MULTI-AGENT & MCP ORCHESTRATION               MODE 2: MODEL GATEWAY SERVER             |
|   (kickstart/multi_agent_orchestration_kickstart/)      (kickstart/gateway_kickstart/)           |
|                                                                                                  |
|   • Planner Agent (Dynamic plan generation)             • POST /v1/chat/completions (OpenAI)     |
|   • Executor Agent (Workflow DAG execution)             • POST /v1/responses (Open Responses)   |
|   • Domain Specialists with MCP Tool integration        • Stateful multi-turn chaining           |
|   • Discovery (4000) & Memory (5000) services           • Multi-provider (Groq, Gemini, OpenAI,  |
|   • Evaluation & Judge Service (Port 7000)                Ollama / vLLM / local endpoints)       |
|   • Resilient OAuth2 / JWT authentication               • High-throughput lock-free cache        |
|                                                                                                  |
+--------------------------------------------------------------------------------------------------+
```

---

## 🤖 Mode 1: Deterministic Multi-Agent Orchestration with MCP

Coordinating multiple agents becomes much easier when service boundaries and message contracts are explicit. Mode 1 splits responsibilities across decoupled, specialized services:

* **Planner Agent (Port 8280):** Analyzes incoming user requests and dynamically constructs execution DAGs.
* **Executor Agent (Port 9580):** Resolves task dependencies and controls step execution.
* **Domain Specialists (Port 8180):** Execute live tools via a native **Model Context Protocol (MCP)** runtime (supporting SSE and streaming tool calls).
* **Discovery (Port 4000) & Memory (Port 5000):** Maintain service registries and conversational state.
* **Evaluation Service (Port 7000):** Built-in LLM-as-a-Judge validation loop for output verification and self-correction.

Inter-agent communication relies on type-safe agent-to-agent (A2A) message contracts, catching contract and integration errors during development and compilation.

```text
User Request
     ↓
  Planner
     ↓
Execution DAG
     ↓
  Executor
     ↓
Weather Agent
     ↓
MCP Weather Tool
     ↓
 Evaluation
     ↓
Final Response
```

### Quickstart (Mode 1)

```bash
cd swarm

# 1. Launch all agents, MCP server, and infrastructure services:
./kickstart/multi_agent_orchestration_kickstart/01_launch_all.sh

# 2. Run a live MCP tool test query (e.g., Live Weather via MCP):
./kickstart/multi_agent_orchestration_kickstart/02_test_weather_query.sh "What is the current weather in Boston ?"

# 3. Stop all background processes when done:
./kickstart/multi_agent_orchestration_kickstart/03_terminate_all.sh
```

All configurations for Mode 1 are located in [`kickstart/multi_agent_orchestration_kickstart/config_files/`](kickstart/multi_agent_orchestration_kickstart/config_files).

> [!NOTE]
> **Port Allocation**: In Mode 1, the Basic Domain Agent listens on `Port 8180` (Planner on `8280`, Executor on `9580`, MCP Server on `8000`). In Mode 2, the Gateway Server listens on `Port 8080`. This clear separation allows both modes to run concurrently on the same machine without port conflicts.

---

## 🌐 Mode 2: OpenAI-Compatible Model Gateway Server

Mode 2 exposes an OpenAI-compatible gateway for client applications, developer tools, and automated pipelines.

- **OpenAI Compatibility (`POST /v1/chat/completions`):** Works with standard OpenAI SDKs, Cursor, and developer extensions.
- **Stateful Responses (`POST /v1/responses`):** Supports multi-turn conversation chaining using explicit `previous_response_id` references and SSE streaming.
- **Unified Multi-Provider Routing:** Route requests across **Groq**, **Google Gemini**, **OpenAI**, or local backends such as **Ollama**, **vLLM**, and **llama.cpp** through TOML configuration.

### ⚙️ Gateway Configuration Preview

Backend routing and default models are configured in [`kickstart/gateway_kickstart/config_files/gateway_config.toml`](kickstart/gateway_kickstart/config_files/gateway_config.toml):

```toml
[server]
bind_address = "0.0.0.0:8080"
log_level = "info"

[models]
default_model = "openai/gpt-oss-20b"

[providers.groq]
api_url = "https://api.groq.com/openai/v1/chat/completions"

[providers.google]
api_url = "https://generativelanguage.googleapis.com/v1beta/models"

[providers.openai]
api_url = "https://api.openai.com/v1/chat/completions"

[providers.custom]
# Local inference (Ollama / vLLM / llama.cpp / LocalAI)
api_url = "http://localhost:11434/v1/chat/completions"
recommended_models = ["llama3.2:latest", "mistral:latest", "deepseek-r1:8b"]
```

### Quickstart (Mode 2)

```bash
cd swarm

# 1. Launch the standalone Gateway Server (port 8080):
# (Automatically detects .env, active Ollama, or offers an interactive provider menu)
./kickstart/gateway_kickstart/01_launch_gateway.sh

# 2. Test OpenAI-Compatible Chat Completions:
curl -X POST http://localhost:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "openai/gpt-oss-20b",
    "messages": [
      {"role": "system", "content": "You are a helpful assistant."},
      {"role": "user", "content": "Explain Swarm architecture in 2 sentences."}
    ]
  }'

# 3. Test Stateful Open Responses (Multi-Turn chaining):
curl -X POST http://localhost:8080/v1/responses \
  -H "Content-Type: application/json" \
  -d '{
    "model": "openai/gpt-oss-20b",
    "input": "My favorite city is Boston."
  }'

# 4. Stop the Gateway Server:
./kickstart/gateway_kickstart/04_terminate_gateway.sh
```

All configurations for Mode 2 are located in [`kickstart/gateway_kickstart/config_files/`](kickstart/gateway_kickstart/config_files).

---

## ⚡ Why Rust?

Rust gives Swarm unique advantages for orchestration and gateway workloads:

* **⚡ Low-Overhead Request Handling:** Built on Tokio and Hyper for asynchronous, high-concurrency gateway workloads with sub-millisecond proxy latency.
* **🔒 Concurrent Session Management:** `DashMap` and `Arc`-based stores allow shared state across concurrent requests without a global application lock.
* **🪶 Small Runtime Footprint:** Lean memory consumption (<25MB baseline), no garbage collector, and predictable memory ownership make it ideal for lightweight deployments.
* **🛡️ Strongly Typed Protocols:** Internal MCP and A2A message contracts reduce schema mismatches and eliminate runtime integration panics.

---

## 📦 Prerequisites

1. **Install Rust**: [rust-lang.org](https://www.rust-lang.org/tools/install) (1.80+ recommended).
2. **Configure Environment Variables**:
   Copy `.env.example` to `.env` and fill in your preferred provider key:
   ```bash
   cp .env.example .env
   ```
   Or export directly in your shell:
   ```bash
   # Groq (default cloud provider for fast inference):
   export GROQ_API_KEY="gsk_your_groq_api_key_here"

   # Google Gemini (optional):
   export GEMINI_API_KEY="your_gemini_api_key_here"

   # OpenAI (optional):
   export OPENAI_API_KEY="your_openai_api_key_here"

   # Local Ollama (optional, e.g. http://localhost:11434/v1/chat/completions):
   # export SWARM_LLM_URL="http://localhost:11434/v1/chat/completions"
   ```

---

## 📁 Directory Structure

```
swarm/
├── kickstart/
│   ├── multi_agent_orchestration_kickstart/   # Mode 1: Multi-Agent & MCP Launch Suite
│   │   ├── 01_launch_all.sh
│   │   ├── 02_test_weather_query.sh
│   │   ├── 03_terminate_all.sh
│   │   ├── README.md
│   │   └── config_files/                      # Agent & MCP configurations
│   │
│   └── gateway_kickstart/                     # Mode 2: Standalone Gateway Launch Suite
│       ├── 01_launch_gateway.sh
│       ├── 02_test_chat_completions.sh
│       ├── 03_test_open_responses.sh
│       ├── 04_terminate_gateway.sh
│       ├── README.md
│       └── config_files/                      # Gateway server configuration & payload templates
│
├── basic_agent/                               # Specialist domain agent embedding MCP runtime
├── planner_agent/                             # Workflow planner and orchestrator
├── executor_agent/                            # Workflow graph executor
├── agent_factory/                             # Dynamic agent instantiation runtime
└── examples/                                  # Example MCP servers and standalone runners
```

---

## 💡 Core Architecture

* **`swarm_commons`**: Shared core traits, A2A interaction protocols, multi-provider LLM adapters, and lock-free state stores.
* **`swarm_services`**: Microservices providing Agent Discovery (4000), Shared Memory (5000), and LLM-as-a-Judge Evaluation (7000).
* **`swarm`**: Core orchestration engines, specialist agents, and the unified gateway server.

---

## 🗺️ Roadmap & Contributing

- [x] Multi-Agent dynamic workflow planning and execution.
- [x] Model Context Protocol (MCP) tool integration.
- [x] Open Responses (`/v1/responses`) stateful turn chaining.
- [x] OpenAI-compatible (`/v1/chat/completions`) endpoint.
- [x] Multi-provider routing (Google Gemini, Groq, OpenAI, Ollama / vLLM).
- [x] Resilient OAuth2 / JWT agent authentication and auto-discovery retries.

We welcome contributions! Feel free to open issues or pull requests.

---

## ⭐ Show Your Support

If you find Swarm useful, please consider starring our repository! Your support helps us grow.

---

## 🙏 Special Thanks

We rely on the fantastic work of these actively developed crates:

* **MCP Protocol:** [https://github.com/modelcontextprotocol/rust-sdk](https://github.com/modelcontextprotocol/rust-sdk)
* **A2A Protocol:** [https://github.com/EmilLindfors/a2a-rs](https://github.com/EmilLindfors/a2a-rs)
