# Swarm Kickstart Suites (`kickstart/`)

Self-contained kickstart environments for running Swarm in its two primary operating modes:

```
swarm/kickstart/
├── multi_agent_orchestration_kickstart/   # Mode 1: Multi-Agent & MCP Orchestration
│   ├── 01_launch_all.sh
│   ├── 02_test_weather_query.sh
│   ├── 03_terminate_all.sh
│   ├── README.md
│   └── config_files/
│       ├── agent_basic_config.toml
│       ├── agent_planner_config.toml
│       ├── agent_executor_config.toml
│       ├── mcp_runtime_config.toml
│       ├── factory_config.toml
│       ├── mix_agent_tools_workflow.json
│       └── mix_agent_tools_workflow_with_email_step.json
│
└── gateway_kickstart/                     # Mode 2: Standalone Model Gateway Server
    ├── 01_launch_gateway.sh
    ├── 02_test_chat_completions.sh
    ├── 03_test_open_responses.sh
    ├── 04_terminate_gateway.sh
    ├── README.md
    └── config_files/
        ├── gateway_config.toml
        ├── chat_completions_request.json
        ├── open_responses_request.json
        └── open_responses_followup_request.json
```

---

## 🤖 Mode 1: Multi-Agent Orchestration (`multi_agent_orchestration_kickstart/`)

Launches the collaborative multi-agent cluster (Planner, Executor, Domain Agent with MCP Tools runtime, Discovery, and Memory services).

```bash
cd swarm

# 1. Start full multi-agent cluster:
./kickstart/multi_agent_orchestration_kickstart/01_launch_all.sh

# 2. Execute sample query with live MCP tool execution:
./kickstart/multi_agent_orchestration_kickstart/02_test_weather_query.sh "What is the current weather in Boston ?"

# 3. Stop all agent processes:
./kickstart/multi_agent_orchestration_kickstart/03_terminate_all.sh
```

---

## 🌐 Mode 2: Standalone Model Gateway Server (`gateway_kickstart/`)

Launches the unified gateway server (`swarm_server`) supporting OpenAI-compatible `/v1/chat/completions` and stateful multi-turn Open Responses `/v1/responses`.

```bash
cd swarm

# 1. Start the Gateway Server (configured via config_files/gateway_config.toml):
./kickstart/gateway_kickstart/01_launch_gateway.sh

# 2. Test OpenAI-compatible Chat Completions:
./kickstart/gateway_kickstart/02_test_chat_completions.sh "What is the weather like in Boston ?"

# 3. Test Stateful Multi-Turn Open Responses:
./kickstart/gateway_kickstart/03_test_open_responses.sh

# 4. Stop the Gateway Server:
./kickstart/gateway_kickstart/04_terminate_gateway.sh
```
