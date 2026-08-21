# Swarm Roadmap

The roadmap prioritizes features that create extra value specifically because Swarm combines model routing and agent execution.

## Current foundation

### Gateway

- [x] OpenAI-compatible `/v1/chat/completions`
- [x] Open Responses `/v1/responses`
- [x] SSE streaming
- [x] Stateful response chaining
- [x] Multi-provider routing
- [x] Cloud and local inference backends

### Agent runtime

- [x] Dynamic workflow planning
- [x] DAG-based execution
- [x] Planner / executor separation
- [x] Domain agents
- [x] Shared memory
- [x] Service discovery
- [x] Typed A2A contracts

### MCP

- [x] MCP integration
- [x] SSE transport
- [x] Streamable transport
- [x] Streaming tool execution

### Evaluation

- [x] LLM-as-a-Judge
- [x] Persistent evaluation records
- [x] Quality scoring
- [x] Critique and feedback

## Near-term priorities

### 1. Unified end-to-end tracing

Target trace:

```text
API Request
  ↓
Gateway
  ↓
Planner
  ↓
Executor
  ├── Agent
  │    └── MCP Tool
  └── Model Call
  ↓
Evaluator
```

Planned:

- [ ] Global trace IDs
- [ ] OpenTelemetry support
- [ ] Cross-service spans
- [ ] Token and cost attribution
- [ ] Tool-call visibility
- [ ] Evaluation data attached to traces
- [ ] Per-step latency breakdown

**Value:** one observability model across inference and agent execution.

---

### 2. Policy-based model routing

Examples:

```text
Sensitive data → local model
Coding task → coding-specialized model
Low-cost task → economy model
High-risk task → premium model + evaluator
```

Planned:

- [ ] Declarative routing policies
- [ ] Provider allow / deny rules
- [ ] Local-only policies
- [ ] Per-agent model policies
- [ ] Metadata-based routing
- [ ] Mandatory evaluation rules
- [ ] Privacy-aware routing

**Value:** Swarm becomes a runtime policy layer rather than only a proxy.

---

### 3. Cost / latency / quality-aware routing

Planned:

- [ ] Provider latency statistics
- [ ] Token-cost accounting
- [ ] Historical quality metrics
- [ ] Weighted routing
- [ ] Model fallback chains
- [ ] Adaptive provider selection
- [ ] Agent-specific model optimization

Potential objective:

```text
maximize quality
subject to:
  latency < 2s
  cost < $0.02
  provider = private
```

**Value:** evaluation history can directly improve routing decisions.

---

### 4. Durable workflow execution

Planned:

- [ ] Durable workflow state
- [ ] Checkpoints
- [ ] Configurable retries
- [ ] Idempotent task execution
- [ ] Crash recovery
- [ ] Workflow pause / resume
- [ ] Cancellation
- [ ] Dead-letter handling
- [ ] Replay from checkpoint

**Value:** moves Swarm toward production-grade agent execution.

---

### 5. Unified `swarm serve`

Target experience:

```bash
swarm serve
```

Optional components:

```bash
swarm serve --gateway --agents --memory --evaluation
```

Planned:

- [ ] Single-process development mode
- [ ] Modular service activation
- [ ] Embedded discovery
- [ ] Embedded memory
- [ ] Embedded evaluation
- [ ] Production multi-service mode
- [ ] Configuration validation

**Value:** one binary for onboarding, modular decomposition for production.

## Strategic roadmap

### 6. Workflow replay and time-travel debugging

- [ ] Execution snapshots
- [ ] Deterministic replay support
- [ ] Mocked model responses
- [ ] Mocked tool responses
- [ ] Compare workflow runs
- [ ] Branch from previous execution steps

**Value:** agent failures become reproducible and debuggable.

---

### 7. Evaluation-driven optimization

- [ ] Golden datasets
- [ ] Prompt regression testing
- [ ] Model-vs-model comparison
- [ ] Agent regression testing
- [ ] Online quality monitoring
- [ ] Automated regression detection
- [ ] Dataset export for SFT / preference optimization

**Value:** evaluation becomes an improvement loop, not only logging.

---

### 8. Governance and authorization

- [ ] RBAC
- [ ] Tenant isolation
- [ ] Agent permissions
- [ ] Tool-level authorization
- [ ] Provider allowlists
- [ ] Model allowlists
- [ ] Audit trails
- [ ] Data-retention policies
- [ ] Redaction policies

**Value:** common governance across model calls and agent actions.

---

### 9. Human-in-the-loop workflows

- [ ] Approval checkpoints
- [ ] Human review queues
- [ ] Pause / resume
- [ ] Escalation policies
- [ ] Editable intermediate outputs
- [ ] Approval metadata

**Value:** enables higher-risk and business-critical workflows.

---

### 10. Semantic capability registry

- [ ] Agent capability metadata
- [ ] MCP tool catalog
- [ ] Semantic capability search
- [ ] Dynamic specialist selection
- [ ] Capability health checks
- [ ] Tool version tracking

**Value:** agents and tools become discoverable runtime resources.

---

### 11. Provider resilience

- [ ] Automatic provider failover
- [ ] Circuit breakers
- [ ] Health-aware routing
- [ ] Retry budgets
- [ ] Rate-limit awareness
- [ ] Provider concurrency controls

---

### 12. Semantic caching

- [ ] Exact-response cache
- [ ] Semantic request cache
- [ ] Agent-step caching
- [ ] Privacy-aware cache policies
- [ ] Cache observability

---

### 13. Multi-tenancy

- [ ] Tenant API keys
- [ ] Isolated state
- [ ] Tenant routing policies
- [ ] Per-tenant quotas
- [ ] Per-tenant budgets
- [ ] Usage attribution

---

### 14. Cloud-native deployment

- [ ] Official Docker image
- [ ] Kubernetes manifests
- [ ] Helm chart
- [ ] Health and readiness endpoints
- [ ] External state backends
- [ ] Secret-manager integration

---

### 15. Reproducible benchmark suite

- [ ] Gateway overhead
- [ ] Streaming throughput
- [ ] Concurrent connections
- [ ] Memory footprint
- [ ] Cold-start time
- [ ] MCP overhead
- [ ] Planner / executor throughput

Published benchmark results should include hardware, methodology, concurrency, request payloads, and latency distributions.
