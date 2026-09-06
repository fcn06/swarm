# 🚀 Swarm — One Runtime from LLM Call to Agent Workflow 🦀

[![Rust](https://img.shields.io/badge/Rust-1.80%2B-orange?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue)](LICENSE)
[![MCP](https://img.shields.io/badge/MCP-SSE%20%2B%20Streamable-purple)](https://modelcontextprotocol.io)
[![Binary Size](https://img.shields.io/badge/Gateway%20Binary-~12MB-success)](#deployment-patterns-one-codebase-two-deploy-patterns)
[![Resilience](https://img.shields.io/badge/Resilience-Circuit%20Breaker%20%2B%20Throttling-brightgreen)](#enterprise-resilience--production-hardening)

> **Swarm is a production-grade, Rust-native AI runtime that unifies model routing, agent orchestration, MCP tools, state, and evaluation — giving your team one cohesive stack from simple LLM proxying to complex multi-agent workflows.**

---

## Table of Contents

- [Why Swarm?](#why-swarm)
- [Deployment Patterns: One Codebase, Two Deploy Patterns](#deployment-patterns-one-codebase-two-deploy-patterns)
- [Enterprise Resilience & Production Hardening](#enterprise-resilience--production-hardening)
- [Quick Onboarding Scenario](#quick-onboarding-scenario)
  - [Step 1 — Put Swarm in Front of Your Models](#step-1--put-swarm-in-front-of-your-models)
  - [Step 2 — Verify Health & Readiness](#step-2--verify-health--readiness)
  - [Step 3 — Execute Chat & Stateful Responses](#step-3--execute-chat--stateful-responses)
  - [Step 4 — Add Agent Orchestration When You Need It](#step-4--add-agent-orchestration-when-you-need-it)
- [Key Capabilities](#key-capabilities)
- [Project Direction](#project-direction)
- [Documentation](#documentation)
- [Repository Structure](#repository-structure)
- [Contributing](#contributing)
- [License](#license)

---

## Why Swarm?

Building AI products typically forces engineering teams to manage two separate, disconnected stacks:

1. An **LLM Gateway** for multi-provider routing, timeouts, rate limiting, and failover.
2. An **Agent Framework** for DAG planning, tool execution, session memory, and multi-agent coordination.

Operating separate systems introduces protocol impedance, duplicated authentication, fragmented observability, and unpredictable latency.

**Swarm solves this by unifying both worlds around a single Tokio-based async runtime and typed protocol abstractions.**

```text
               Client Application / API Consumer
                               │
                               ▼
 ┌───────────────────────────────────────────────────────────┐
 │                       S W A R M                           │
 ├─────────────────────────────┬─────────────────────────────┤
 │   Standalone Gateway        │   Full Agent Runtime        │
 │   • /v1/chat/completions    │   • Planner & Executor DAGs │
 │   • /v1/responses (State)   │   • Domain Specialist Agents│
 │   • Provider Circuit Breaker│   • Native MCP Tool Host    │
 │   • Concurrency Throttling  │   • Shared Working Memory   │
 │   • Health Probes           │   • LLM-as-a-Judge Eval     │
 └──────────────┬──────────────┴──────────────┬──────────────┘
                │                             │
         ┌──────┴──────────────┐       ┌──────┴──────────────┐
         ▼                     ▼       ▼                     ▼
    Cloud Models          Local Models   External Tools   Enterprise State
    OpenAI, Gemini, Groq  Ollama, vLLM   MCP SSE/HTTP     Discovery & Memory
```

### Key Customer Value Proposition

**Start as a high-performance, resilient model gateway. Scale into full agent workflows without ever replacing your underlying AI infrastructure.**

```text
Direct LLM Call
  → Resilient Model Gateway (12MB binary, sub-10ms startup)
    → Stateful Turn Chaining (/v1/responses)
      → MCP Tool Integration (Streaming SSE)
        → Autonomous Multi-Agent DAGs
          → Automated LLM Evaluation & Guardrails
```

---

## Deployment Patterns: One Codebase, Two Deploy Patterns

Different workloads demand different operational footprints. Swarm provides two first-class deployment targets from the same codebase:

### Pattern A: Standalone Gateway (`swarm_gateway`)
* **Target Workloads**: Kubernetes sidecar proxy, edge deployment, microservice LLM egress, high-throughput model router.
* **Footprint**: **~12 MB release binary**, cold starts under **10ms**, minimal RSS memory.
* **Isolation**: Strips away agent orchestration, MCP tool engines, and planner DAGs. If downstream agent tools fail, your gateway stays 100% online.
* **Capabilities**: `/v1/chat/completions`, `/v1/responses`, multi-provider routing, concurrency control, timeouts, and circuit breaking.

### Pattern B: Unified Swarm Server (`swarm_server`)
* **Target Workloads**: All-in-one AI platform, enterprise agent workspaces, complete developer environments.
* **Footprint**: Single unified binary containing both the gateway routes and the full multi-agent orchestration engine.
* **Capabilities**: Everything in Pattern A plus dynamic DAG planning, executor workers, domain specialist agents, native MCP server hosting, discovery service, and evaluation.

---

## Enterprise Resilience & Production Hardening

When deploying LLMs to production, upstream provider latency spikes, outages, and connection hangs can compromise your application. Swarm includes built-in, zero-dependency resilience features:

| Resilience Feature | Customer Benefit | HTTP Status On Trip |
|---|---|:---:|
| **Concurrency Limiting** | Caps inflight requests to prevent thread starvation and out-of-memory crashes under peak traffic surges. | `429 Too Many Requests` |
| **Execution Deadlines** | Enforces per-request timeouts so hung provider connections or slow generation never lock client sockets. | `408 Request Timeout` |
| **Provider Circuit Breakers** | Automatically detects consecutive provider failures (e.g. OpenAI or Groq outage), failing fast and allowing automatic cooldown recovery. | `503 Service Unavailable` |
| **Cloud-Native Health Checks** | Built-in `/health` and `/v1/health` endpoints returning provider readiness for Kubernetes probes and load balancers. | `200 OK` / `503 Unhealthy` |

### Resilience Configuration Example

Add this simple section to your `gateway_config.toml` to guard your production endpoints:

```toml
[resilience]
# Concurrency limit for inflight requests (protects Tokio worker threads)
max_concurrent_requests = 512

# Global request timeout in seconds (guards against hung upstream connections)
request_timeout_seconds = 60

# Circuit breaker: trip provider offline if N consecutive errors occur
circuit_breaker_failure_threshold = 5

# Cooldown window: wait N seconds before testing provider recovery
circuit_breaker_reset_seconds = 30
```

---

## Quick Onboarding Scenario

### Step 1 — Put Swarm in Front of Your Models

Launch the standalone gateway in seconds:

```bash
cd swarm
./kickstart/gateway_kickstart/01_launch_gateway.sh
```

### Step 2 — Verify Health & Readiness

Confirm the gateway and configured providers are healthy:

```bash
curl http://localhost:8080/health
```

```json
{
  "status": "healthy",
  "version": "0.1.0",
  "providers_configured": 4
}
```

### Step 3 — Execute Chat & Stateful Responses

**Standard OpenAI-Compatible Chat (`/v1/chat/completions`)**

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

**Stateful Open Responses (`/v1/responses` — Multi-Turn Session Memory)**

```bash
# Turn 1: Initial query (returns session ID "resp_123...")
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

Swarm routes seamlessly across cloud and local providers:

| Provider | Type | Typical Use Case |
|---|---|---|
| **Groq** | Cloud | Ultra-low latency Llama 3.3 / Qwen generation |
| **Google Gemini** | Cloud | Multimodal and large context-window reasoning |
| **OpenAI** | Cloud | Industry-standard GPT-4o models |
| **Ollama / vLLM / llama.cpp** | Local / Self-Hosted | Private, on-premise, zero-data-egress inference |

### Step 4 — Add Agent Orchestration When You Need It

When your use case evolves from single LLM calls to multi-step reasoning, launch the full orchestration stack:

```bash
./kickstart/multi_agent_orchestration_kickstart/01_launch_all.sh
```

Test a coordinated agent task:

```bash
./kickstart/multi_agent_orchestration_kickstart/02_test_weather_query.sh \
  "What is the current weather in Boston?"
```

The request automatically coordinates across:
```text
User → Planner → Execution DAG → Executor → Domain Agent → MCP Tool → Evaluation → Response
```

---

## Key Capabilities

### 🌐 Model Gateway
- OpenAI-compatible `/v1/chat/completions` and stateful `/v1/responses`.
- Multi-provider intelligent routing (OpenAI, Gemini, Groq, local Ollama/vLLM).
- SSE streaming with token-level output.
- In-memory lock-free session store (`DashMap`) for conversation history.
- Available as a standalone, zero-orchestration **12MB** binary (`swarm_gateway`).

### 🛡️ Enterprise Resilience
- **Concurrency Throttling**: Semaphore-backed inflight request bounding (`429 Too Many Requests`).
- **Request Timeouts**: Configurable per-request deadlines (`408 Request Timeout`).
- **Circuit Breaker**: Automatic provider outage isolation with half-open recovery probes (`503 Service Unavailable`).
- **Health Probes**: Liveness and readiness `/health` endpoints for orchestration platforms.

### 🤖 Agent Runtime
- Dynamic workflow planning with DAG execution graphs.
- Strict Planner / Executor separation for safety and auditability.
- Domain specialist agents connected via typed A2A contracts.
- Centralized Agent Discovery and Shared Working Memory services.

### 🔌 Native MCP (Model Context Protocol)
- Full MCP client and server protocol support.
- Standard SSE and Streamable HTTP transports.
- Dynamic tool discovery and streaming execution.

### 🔍 Built-in Evaluation
- Automated LLM-as-a-Judge quality assessment.
- Historical audit scoring, critique generation, and regression tracking.
- Pre-deployment quality gates for agent behaviors and prompts.

### ⚡ Rust & Tokio Performance
- Sub-10ms cold start times.
- Extremely low memory footprint with zero garbage-collection pauses.
- Predictable P99 latencies under heavy enterprise concurrency.

---

## Project Direction

Swarm is evolving from an independent Gateway + Framework into an **Enterprise AI Control Plane**:

```text
[ Gateway + Agent Framework ]  ──►  [ Unified AI Runtime & Control Plane ]
```

Key roadmap initiatives:
- **Unified End-to-End Tracing**: Distributed OpenTelemetry traces spanning prompt, gateway routing, DAG nodes, and MCP tool execution.
- **Cost & Latency Routing**: Dynamic model selection based on real-time price, latency SLAs, and provider health.
- **Durable Workflow Execution**: Checkpoint and resume long-running agent workflows across process restarts.
- **Human-in-the-Loop Policies**: Interactive approval steps embedded directly into execution graphs.
- **Unified `swarm serve` CLI**: Single command local development and deployment.

See [docs/ROADMAP.md](docs/ROADMAP.md) for full roadmap details.

---

## Documentation

- [Architecture & Design](docs/ARCHITECTURE.md)
- [Deployment & Operations](docs/DEPLOYMENT.md)
- [Evaluation & Continuous Improvement](docs/EVALUATION.md)
- [Design Principles](docs/DESIGN_PRINCIPLES.md)
- [Roadmap](docs/ROADMAP.md)

---

## Repository Structure

```text
swarm/
├── kickstart/
│   ├── gateway_kickstart/                     # Standalone gateway scripts & configs
│   │   ├── 01_launch_gateway.sh
│   │   ├── 02_test_chat_completions.sh
│   │   ├── 03_test_open_responses.sh
│   │   ├── 04_terminate_gateway.sh
│   │   └── config_files/gateway_config.toml  # Resilience & provider settings
│   └── multi_agent_orchestration_kickstart/   # Full agent cluster kickstarts
├── agent_factory/                             # Unified swarm_server binary & builder
├── basic_agent/                               # Base agent scaffolding & traits
├── planner_agent/                             # DAG planning engine
├── executor_agent/                            # Dynamic task execution worker
├── workflow_management/                       # State and workflow transitions
├── resource_invoker/                          # Tool invocation & MCP bridge
└── docs/                                      # Comprehensive architectural guides
```

---

## Contributing

Contributions, issues, architecture discussions, and real-world feedback are warmly welcomed!

- Report bugs or submit feature requests via [GitHub Issues](https://github.com/fcn06/swarm/issues).
- Discuss design ideas and deployment patterns in [Discussions](https://github.com/fcn06/swarm/discussions).
- If Swarm accelerates your AI stack, consider giving the repository a ⭐!

---

## License

Distributed under the [Apache License 2.0](LICENSE).

---

> **Swarm — One runtime from LLM call to agent workflow.**
