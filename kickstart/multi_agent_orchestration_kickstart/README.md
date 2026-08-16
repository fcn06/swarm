# Multi-Agent Orchestration Kickstart (`multi_agent_orchestration_kickstart/`)

Self-contained kickstart setup with localized configurations for **Mode 1: Multi-Agent Orchestration & MCP Tools System**.

---

## 📁 Directory Layout

```
swarm/kickstart/multi_agent_orchestration_kickstart/
├── 01_launch_all.sh           # Builds and launches Discovery, Memory, MCP server, and Agents
├── 02_test_weather_query.sh   # Sends queries to the planner agent and prints responses
├── 03_terminate_all.sh        # Stops all running agent and service processes
├── README.md                  # Quickstart documentation
└── config_files/              # Localized configuration files
    ├── agent_basic_config.toml
    ├── agent_planner_config.toml
    ├── agent_executor_config.toml
    ├── mcp_runtime_config.toml
    ├── factory_config.toml
    ├── mix_agent_tools_workflow.json
    └── mix_agent_tools_workflow_with_email_step.json
```

---

## 🏛️ Services Launched

| Component | Port | Config File |
| :--- | :--- | :--- |
| **Discovery Service** | `4000` | In-memory registry |
| **Memory Service** | `5000` | In-memory task store |
| **MCP Tools Server** | `8000` | `main-server` with `weather`, `customer`, `scrape`, `search` |
| **Basic Domain Agent** | `8080` | `config_files/agent_basic_config.toml` |
| **Planner Orchestrator** | `8280` | `config_files/agent_planner_config.toml` |
| **Executor Agent** | `9580` | `config_files/agent_executor_config.toml` |

---

## 🚀 Quick Usage

```bash
cd swarm

# 1. Export your Groq API key:
export GROQ_API_KEY="gsk_..."

# 2. Launch the full agent cluster:
./kickstart/multi_agent_orchestration_kickstart/01_launch_all.sh

# 3. Run a query:
./kickstart/multi_agent_orchestration_kickstart/02_test_weather_query.sh "What is the current weather in Boston ?"

# 4. Stop everything when done:
./kickstart/multi_agent_orchestration_kickstart/03_terminate_all.sh
```
