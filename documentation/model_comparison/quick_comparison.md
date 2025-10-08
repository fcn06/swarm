# LLM Model Comparison for Swarm

This document provides a summary of our internal evaluation of various Large Language Models (LLMs) for use with the Swarm framework. The goal of this comparison is to help you select the best model for your specific use case.

## Supported LLM Providers

We have tested the following LLM providers, which offer generous free plans:

*   **Groq:** [https://api.groq.com/openai/v1/chat/completions](https://api.groq.com/openai/v1/chat/completions)
*   **Gemini:** [https://generativelanguage.googleapis.com/v1beta/openai/chat/completions](https://generativelanguage.googleapis.com/v1beta/openai/chat/completions)

## Model Evaluation

Below is a summary of our findings. The models were evaluated based on their performance in three key areas: General Purpose, MCP (Model Context Protocol), and Orchestration.

| Provider | Model | General Purpose | MCP | Orchestration / Workflow Management | Use Case & Comments |
| :--- | :--- | :---: | :---: | :---: | :--- |
| Groq | qwen/qwen3-32b | ✓ | ✓ | ✓ | **Recommended for Basic Agents.** Works well and is very fast, but you may encounter rate limiting with the free plan. |
| Groq | openai/gpt-oss-20b | ✓ | ✓ | ✓ | **Excellent for complex use cases. Recommended for Worklflow management and LLM as a Judge, but also for MCP runtime** Works well and is very fast |
| Groq | deepseek-r1-distill-llama-70b | ✓ | ✓ | ✓ | **Another good alternative.** Works well and is very fast, but you may encounter rate limiting with the free plan. |
| Groq | meta-llama/llama-4-scout-17b-16e-instruct | ✓ | ✓ | ✓ | **A solid choice.** Works well and is very fast, but you may encounter rate limiting with the free plan. |
| Google | gemini-2.0-flash | ✓ | ✓ | ✓ | **Good, but can be verbose.** Works well, but the responses can be more verbose than other models. |
| Groq | llama-3.1-8b-instant | ✓ | ✗ | ✗ | **Only for simple tasks.** This model is not recommended for complex orchestration or MCP tasks. |

## Future Plans & How You Can Help

Our model testing is an ongoing effort, and we welcome community contributions to make Swarm compatible with a wider range of LLMs.

### Testing with OpenAI

While we have focused on providers with generous free tiers, we are very interested in understanding how Swarm performs with OpenAI's models (e.g., GPT-4, GPT-4o). If you have access and are willing to share your findings, please **[open an issue](https://github.com/your-repo-url/swarm/issues)** to share your results. Your feedback is invaluable!

### Integrating `llama.cpp` for Edge AI

We are excited about the prospect of running smaller, highly specialized models on edge devices. To that end, we plan to integrate with `llama.cpp` ([https://github.com/ggml-org/llama.cpp](https://github.com/ggml-org/llama.cpp)). This will unlock new possibilities for privacy-focused, efficient, and fine-tuned agents.

This is a significant undertaking, and we would love community support. If you have experience with `llama.cpp` and are interested in contributing, please let us know!

## Conclusion

For most use cases, we recommend using one of the Groq models, such as **qwen/qwen3-32b**. These models provide a good balance of performance and speed. If you find that the responses are too verbose, you may want to try one of the other Groq models.

For simple tasks that do not require complex orchestration or MCP, the **llama-3.1-8b-instant** model may be a good choice. However, we do not recommend using this model for more complex tasks.
