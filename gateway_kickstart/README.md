# Swarm Gateway Kickstart (`gateway_kickstart/`)

Standalone scripts for building, launching, testing, and managing the **Swarm Model Gateway Server** (`swarm_server`).

---

## 🌐 Supported Endpoints

| Endpoint | Standard | Features |
| :--- | :--- | :--- |
| **`POST /v1/chat/completions`** | OpenAI Compatible | Backward-compatibility for IDEs, extensions, and standard LLM clients. |
| **`POST /v1/responses`** | Open Responses | Modern stateful multi-turn sessions with `previous_response_id` chaining and SSE streaming. |

---

## 📁 Directory Layout

```
swarm/gateway_kickstart/
├── 01_launch_gateway.sh           # Builds and runs swarm_server on port 8080
├── 02_test_chat_completions.sh    # Sends test POST /v1/chat/completions request
├── 03_test_open_responses.sh      # Sends test multi-turn stateful POST /v1/responses request
├── 04_terminate_gateway.sh        # Stops the running gateway server
└── README.md                      # Documentation
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

# 1. Launch the Gateway Server (default port 8080):
./gateway_kickstart/01_launch_gateway.sh

# 2. Test Chat Completions:
./gateway_kickstart/02_test_chat_completions.sh "What is the weather like in Boston ?"

# 3. Test Open Responses (Stateful Multi-Turn):
./gateway_kickstart/03_test_open_responses.sh

# 4. Terminate the Gateway Server:
./gateway_kickstart/04_terminate_gateway.sh
```
