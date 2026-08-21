# Deployment and Configuration

Swarm can currently run as an agent orchestration stack, a standalone gateway, or both.

## Prerequisites

Install Rust:

```bash
rustup update
```

Rust `1.80+` is recommended.

Copy the environment template:

```bash
cp .env.example .env
```

Configure one or more providers:

```bash
export GROQ_API_KEY="gsk_your_groq_api_key_here"
export GEMINI_API_KEY="your_gemini_api_key_here"
export OPENAI_API_KEY="your_openai_api_key_here"
```

For local inference:

```bash
export SWARM_LLM_URL="http://localhost:11434/v1/chat/completions"
```

## Gateway mode

Start:

```bash
cd swarm
./kickstart/gateway_kickstart/01_launch_gateway.sh
```

Test chat completions:

```bash
./kickstart/gateway_kickstart/02_test_chat_completions.sh
```

Test Open Responses:

```bash
./kickstart/gateway_kickstart/03_test_open_responses.sh
```

Stop:

```bash
./kickstart/gateway_kickstart/04_terminate_gateway.sh
```

Gateway configuration lives in:

```text
kickstart/gateway_kickstart/config_files/
```

## Agent orchestration mode

Start:

```bash
cd swarm
./kickstart/multi_agent_orchestration_kickstart/01_launch_all.sh
```

Run the weather example:

```bash
./kickstart/multi_agent_orchestration_kickstart/02_test_weather_query.sh \
  "What is the current weather in Boston?"
```

Stop:

```bash
./kickstart/multi_agent_orchestration_kickstart/03_terminate_all.sh
```

Agent configuration lives in:

```text
kickstart/multi_agent_orchestration_kickstart/config_files/
```

## Default ports

| Service | Port |
|---|---:|
| Gateway | `8080` |
| MCP Server | `8000` |
| Domain Agent | `8180` |
| Planner | `8280` |
| Executor | `9580` |
| Discovery | `4000` |
| Memory | `5000` |
| Evaluation | `7000` |

The two modes can run concurrently because their ports are separated.

## Gateway configuration example

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
api_url = "http://localhost:11434/v1/chat/completions"
recommended_models = [
  "llama3.2:latest",
  "mistral:latest",
  "deepseek-r1:8b"
]
```

## Target deployment direction

A future simplified local experience is:

```bash
swarm serve
```

while preserving the ability to decompose Swarm into independently scalable services for production environments.
