# Design Principles

Swarm is guided by a small set of principles intended to keep the project focused as capabilities grow.

## 1. Progressive complexity

Users should be able to start with a simple model endpoint and adopt tools, state, evaluation, and agents only when needed.

```text
Gateway → Tools → Agents → Evaluation → Policy
```

## 2. One runtime model

Model calls and agent workflows should not require unrelated infrastructure stacks.

Provider access, state, authentication, observability, and policy should converge on shared abstractions.

## 3. Explicit execution

Agent workflows should be inspectable as structured execution graphs.

The planner may be probabilistic, but workflow execution should remain explicit and traceable.

## 4. Open protocols

Prefer interoperable interfaces and standards.

Current examples include:

- OpenAI-compatible APIs;
- Open Responses;
- MCP;
- A2A.

## 5. Cloud / local symmetry

Local models should be first-class runtime targets.

Applications should be able to move between cloud and local inference without rewriting their application-facing integration.

## 6. Observability by design

Model calls, agent actions, tool calls, policy decisions, and evaluations should eventually belong to the same trace.

## 7. Evaluation as a runtime concern

Quality should be measurable and persisted.

Evaluation data should eventually influence:

- regression testing;
- model selection;
- routing;
- prompt changes;
- agent optimization.

## 8. Production-oriented reliability

Swarm should prioritize:

- resumability;
- retries;
- idempotency;
- failure isolation;
- authentication;
- policy enforcement;
- reproducibility.

## 9. Lightweight by default

Rust and Tokio should translate into practical operational benefits:

- predictable memory;
- low runtime overhead;
- efficient concurrency;
- small deployment footprint.

Performance claims should be backed by reproducible benchmarks.

## 10. Modular deployment

Swarm should be easy to run locally as one runtime while allowing services to be separated and scaled independently in production.
