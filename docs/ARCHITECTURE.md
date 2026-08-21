# Swarm Architecture

This document describes the main runtime components and how the gateway and orchestration modes fit together.

## Architectural idea

Swarm treats model access and agent execution as two levels of the same AI runtime.

```text
                     Application
                         |
                         v
                +----------------+
                |     SWARM      |
                +----------------+
                  /            \
                 /              \
        Model Gateway        Agent Runtime
             |                   |
       Provider APIs      Planner / Executor
             |             Agents / MCP
             \                   /
              \                 /
               +---------------+
               | Shared Core   |
               |---------------|
               | Providers     |
               | State         |
               | Auth          |
               | Protocols     |
               | Tokio Runtime |
               +---------------+
```

## Mode 1 — Multi-Agent Orchestration

### Planner Agent — Port 8280

Analyzes incoming requests and produces an execution DAG.

Responsibilities:

- task decomposition;
- dependency identification;
- specialist selection;
- execution-plan generation.

Planning may be LLM-driven and therefore probabilistic.

### Executor Agent — Port 9580

Executes the generated DAG through explicit task dependencies.

Responsibilities:

- DAG traversal;
- dependency resolution;
- scheduling;
- result propagation;
- execution control.

This is better described as **deterministic workflow execution over dynamically generated plans**.

### Domain Agents — Port 8180

Specialized agents execute domain tasks and can invoke tools through MCP.

### Discovery Service — Port 4000

Maintains service and agent registration.

### Memory Service — Port 5000

Provides shared conversational and workflow state.

### Evaluation Service — Port 7000

Evaluates outputs and persists quality information.

### MCP Server — Port 8000

Exposes tool capabilities through MCP.

## Mode 2 — Model Gateway

The standalone gateway runs on port `8080`.

Supported API surfaces include:

```text
POST /v1/chat/completions
POST /v1/responses
```

The gateway can route to cloud providers and local inference backends.

## Shared foundation

The important architectural distinction is that both modes are built around common abstractions rather than two independent products.

Core shared concerns include:

- provider adapters;
- authentication;
- state management;
- message contracts;
- runtime primitives;
- configuration;
- protocol handling.

## Core crates

### `swarm_commons`

Shared traits, types, protocol contracts, provider abstractions, and state utilities.

### `swarm_services`

Infrastructure services such as discovery, memory, and evaluation.

### `swarm`

Core orchestration engines, specialist agents, and gateway runtime.

## Target architecture

The longer-term direction is a single execution graph that can trace:

```text
API Request
  ↓
Gateway
  ↓
Planner
  ↓
Workflow
  ├── Model Call
  ├── Agent
  │    └── MCP Tool
  └── Evaluation
```

This is the foundation for unified observability, policy enforcement, and runtime optimization.
