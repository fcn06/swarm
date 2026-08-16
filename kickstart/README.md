# Swarm Multi-Agent & MCP Kickstart (`kickstart/`)

Self-contained kickstart setup with localized configurations in `kickstart/config_files/`.

---

## 📁 Directory Layout

```
swarm/kickstart/
├── 01_launch_all.sh           # Builds and launches Discovery, Memory, MCP server, and Agents
├── 02_test_weather_query.sh   # Sends the Boston weather query and prints the clean response
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
| **Basic Domain Agent** | `8080` | `kickstart/config_files/agent_basic_config.toml` |
| **Planner Orchestrator** | `8280` | `kickstart/config_files/agent_planner_config.toml` |
| **Executor Agent** | `9580` | `kickstart/config_files/agent_executor_config.toml` |

---

## 🚀 Quick Usage

```bash
cd swarm

# 1. Export your Groq API key:
export GROQ_API_KEY="gsk_..."

# 2. Launch the full agent stack:
./kickstart/01_launch_all.sh

# 3. In another terminal, run the query:
./kickstart/02_test_weather_query.sh

# 4. Stop everything when done:
./kickstart/03_terminate_all.sh
```
