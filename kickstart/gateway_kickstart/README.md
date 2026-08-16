# Gateway Kickstart (`gateway_kickstart/`)

Standalone, self-contained suite for building, launching, testing, and managing **Mode 2: The Swarm Model Gateway Server** (`swarm_server`).

---

## 🌐 Supported Endpoints

| Endpoint | Standard | Features |
| :--- | :--- | :--- |
| **`POST /v1/chat/completions`** | OpenAI Compatible | Backward-compatibility for IDEs, extensions, and standard LLM clients. |
| **`POST /v1/responses`** | Open Responses | Modern stateful multi-turn sessions with `previous_response_id` chaining and SSE streaming. |

---

## ⚙️ Target URL & Provider Configuration

All gateway settings, default models, and provider target URLs are configured in [`config_files/gateway_config.toml`](config_files/gateway_config.toml):

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
# For local LLMs or custom proxies (Ollama / vLLM / llama.cpp / LocalAI)
# api_url = "http://127.0.0.1:11434/v1/chat/completions"
```

---

## 📁 Directory Layout

```
swarm/kickstart/gateway_kickstart/
├── 01_launch_gateway.sh           # Builds and runs swarm_server on port 8080
├── 02_test_chat_completions.sh    # Sends test POST /v1/chat/completions request
├── 03_test_open_responses.sh      # Sends test multi-turn stateful POST /v1/responses request
├── 04_terminate_gateway.sh        # Stops the running gateway server
├── README.md                      # Documentation
└── config_files/                  # Localized configuration & payload templates
    ├── gateway_config.toml        # Server bind, default model & provider URLs
    ├── chat_completions_request.json
    ├── open_responses_request.json
    └── open_responses_followup_request.json
```

---

## 🔑 Environment Variables

Export your provider API key before launching (at least one):

```bash
export GROQ_API_KEY="gsk_..."
# or
export GEMINI_API_KEY="AIza..."
# or
export OPENAI_API_KEY="sk-..."
```

---

## 🚀 Quick Usage

```bash
cd swarm

# 1. Launch the Gateway Server (configured via gateway_config.toml):
./kickstart/gateway_kickstart/01_launch_gateway.sh

# 2. Test Chat Completions:
./kickstart/gateway_kickstart/02_test_chat_completions.sh "What is the weather like in Boston ?"

# 3. Test Open Responses (Stateful Multi-Turn):
./kickstart/gateway_kickstart/03_test_open_responses.sh

# 4. Terminate the Gateway Server:
./kickstart/gateway_kickstart/04_terminate_gateway.sh
```
