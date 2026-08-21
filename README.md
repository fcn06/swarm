# 🚀 Swarm — One Runtime from LLM Call to Agent Workflow 🦀

[![Rust](https://img.shields.io/badge/Rust-1.88-orange?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue)](LICENSE)
[![MCP](https://img.shields.io/badge/MCP-SSE%20%2B%20Streamable-purple)](https://modelcontextprotocol.io)

 
> **Swarm is a Rust-native AI runtime that unifies model routing, agent orchestration, MCP tools, state, and evaluation — so teams don't have to operate separate gateway and agent stacks.**
 
---
 
## Table of Contents
 
- [Why Swarm?](#why-swarm)
- [Quick Onboarding Scenario](#quick-onboarding-scenario)
- [Key Capabilities](#key-capabilities)
- [Project Direction](#project-direction)
- [Documentation](#documentation)
- [Repository Structure](#repository-structure)
- [Contributing](#contributing)
- [License](#license)
---
 
## Why Swarm?
 
AI applications often start with direct LLM calls, then gradually add provider routing, tools, memory, agents, and evaluation.
 
That growth commonly produces two separate infrastructure layers:
 
- an **LLM gateway** for model access and routing
- an **agent framework** for planning, tools, workflows, and state
Swarm combines both around one Tokio-based runtime and shared protocol abstractions.
 
```text
Application
    │
    ▼
┌───────────────────────┐
│         SWARM         │
├───────────────────────┤
│ Model Gateway          │
│ Agent Orchestration    │
│ MCP Tools              │
│ Shared State           │
│ Evaluation             │
└───────────┬────────────┘
            │
     ┌──────┴───────┐
     │              │
 Cloud Models    Local Models
 OpenAI          Ollama
 Gemini          vLLM
 Groq            llama.cpp
```
 
### Key USP
 
**Start as a model gateway. Grow into agent workflows without replacing your AI infrastructure.**
 
Swarm is designed for progressive adoption:
 
```text
LLM API
  → Multi-provider Gateway
    → Stateful Responses
      → MCP Tools
        → Agent Workflows
          → Evaluation & Policy
```
 
---
 
## Quick Onboarding Scenario
 
Imagine an application that currently calls OpenAI directly.
 
### Step 1 — Put Swarm in Front of the Model
 
Start the gateway:
 
```bash
cd swarm
./kickstart/gateway_kickstart/01_launch_gateway.sh
```
 
Your application can then use either the OpenAI-compatible endpoint or the Open Responses endpoint:
 
**Chat Completions (`/v1/chat/completions`)**
 
```bash
curl -X POST http://localhost:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "openai/gpt-oss-20b",
    "messages": [
      {"role": "user", "content": "Explain Swarm in one sentence."}
    ]
  }'
```
 
**Open Responses (`/v1/responses` — Stateful Multi-Turn)**

```bash
# Turn 1: Initial query (returns an "id", e.g. "resp_123...")
curl -X POST http://localhost:8080/v1/responses \
  -H "Content-Type: application/json" \
  -d '{
    "model": "openai/gpt-oss-20b",
    "input": "My favorite city is Boston."
  }'

# Turn 2: Follow-up query linked with previous_response_id
curl -X POST http://localhost:8080/v1/responses \
  -H "Content-Type: application/json" \
  -d '{
    "model": "openai/gpt-oss-20b",
    "previous_response_id": "resp_123...",
    "input": "What is its most famous monument?"
  }'
```
 
Behind the same gateway, Swarm can route to:
 
| Provider | Type |
|---|---|
| OpenAI | Cloud |
| Google Gemini | Cloud |
| Groq | Cloud |
| Ollama | Local |
| vLLM | Local |
| llama.cpp | Local |
| Other OpenAI-compatible endpoints | Cloud / Local |
 
### Step 2 — Add Agent Execution When You Need It
 
Launch the orchestration stack:
 
```bash
./kickstart/multi_agent_orchestration_kickstart/01_launch_all.sh
```
 
Run the example workflow:
 
```bash
./kickstart/multi_agent_orchestration_kickstart/02_test_weather_query.sh \
  "What is the current weather in Boston?"
```
 
The request can now flow through:
 
```text
User → Planner → Execution DAG → Executor → Domain Agent → MCP Tool → Evaluation → Response
```
 
The important part: the gateway and agent runtime share the same architectural foundation.
 
---

## Key capabilities

### 🌐 Model Gateway

- OpenAI-compatible `/v1/chat/completions`
- Open Responses `/v1/responses`
- SSE streaming
- Stateful response chaining
- Cloud and local model backends
- Multi-provider routing

### 🤖 Agent Runtime

- Dynamic workflow planning
- DAG-based execution
- Planner / executor separation
- Domain specialist agents
- Shared memory
- Agent discovery
- Typed A2A contracts

### 🔌 MCP

- Native MCP integration
- SSE transport
- Streamable HTTP
- Streaming tool execution

### 🔍 Evaluation

- Built-in LLM-as-a-Judge
- Persistent evaluation history
- Quality scoring and critique
- Foundation for regression testing and model comparison

### ⚡ Rust / Tokio

- Async high-concurrency runtime
- Predictable memory usage
- Strongly typed protocol contracts
- Lightweight self-hosting
- Suitable for cloud and local AI infrastructure

## Project direction

Swarm aims to evolve from:

```text
Gateway + Agent Framework
```

into:

```text
AI Runtime / AI Control Plane
```

where model routing, agent execution, tools, policy, observability, and evaluation are part of the same execution graph.

High-value roadmap areas include:

- unified end-to-end tracing;
- policy-based model routing;
- cost / latency / quality-aware routing;
- durable workflow execution;
- workflow replay;
- human-in-the-loop approval;
- enterprise governance;
- simplified `swarm serve` deployment.

See [docs/ROADMAP.md](docs/ROADMAP.md) for details.

## Documentation

- [Architecture](docs/ARCHITECTURE.md)
- [Roadmap](docs/ROADMAP.md)
- [Evaluation & Continuous Improvement](docs/EVALUATION.md)
- [Deployment & Configuration](docs/DEPLOYMENT.md)
- [Design Principles](docs/DESIGN_PRINCIPLES.md)

## Repository structure

```text
swarm/
├── kickstart/
│   ├── multi_agent_orchestration_kickstart/
│   └── gateway_kickstart/
├── basic_agent/
├── planner_agent/
├── executor_agent/
├── agent_factory/
└── examples/
```

## Contributing

Contributions, issues, architecture discussions, and real-world feedback are welcome.

If Swarm is useful to you, consider starring the repository.

## License

Apache License 2.0.

---

> **Swarm — one runtime from LLM call to agent workflow.**
