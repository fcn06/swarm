# Swarm Codebase Structure

This document provides a comprehensive overview of the `swarm` codebase structure, outlining the key components and their interactions. The project is organized as a Cargo workspace, which allows for modular development and efficient dependency management.

## Project Root

The root of the project contains the main `Cargo.toml` file, which defines the workspace members and shared dependencies. This centralized configuration ensures consistency across all crates and simplifies the build process.

### Key Directories

- **`agent_core`**: This crate defines the core agent functionalities, communication protocols, and execution logic that all agents within the swarm build upon.
- **`agent_discovery_service`**: This crate is responsible for discovering and registering agents within the swarm. It provides a centralized service that allows agents to find and communicate with each other.
- **`agent_evaluation_service`**: This component provides services for evaluating agent performance and behavior.
- **`agent_memory_service`**: This component provides a memory service for agents, allowing them to store and retrieve information.
- **`basic_agent`**: This is a sample implementation of a basic agent that demonstrates how to build and integrate new agents into the swarm.
- **`configuration`**: This directory contains the configuration files for the swarm, including settings for agents, services, and the overall environment, as well as prompts for various agents.
- **`documentation`**: This directory contains the project documentation, including this document, code structure, demo workflows, illustrations, and model comparisons.
- **`examples`**: This directory contains example code that demonstrates how to use the various components of the swarm, such as A2A agent endpoints and MCP runtime endpoints.
- **`llm_api`**: This crate provides an interface to the Language Model (LLM) API, allowing agents to leverage its capabilities for chat and tool usage.
- **`mcp_runtime`**: This component is responsible for managing the Master Control Program (MCP) runtime, which orchestrates the overall behavior of the swarm. It includes agent logic, client interactions, and tool definitions.
- **`workflow_agent`**: This agent is responsible for executing predefined workflows, coordinating tasks and agents as per the workflow definition.
- **`workflow_management`**: This crate provides functionalities for defining, managing, and orchestrating complex multi-agent workflows, including graph-based orchestration, task management, and agent/tool communication.

## Core Components

The core components of the swarm are the `agent_core` and the `mcp_runtime`. These two crates provide the foundation for the entire system and are essential for its operation.

### `agent_core`

The `agent_core` is the heart of the swarm. It defines the core `Agent` trait, agent interaction protocols, and the fundamental business logic for agents. The key modules in this crate are:

- **`agent_interaction_protocol`**: Defines the communication protocols for agents.
- **`business_logic`**: Contains the core business logic, including the `Agent` trait and services.
- **`execution`**: Handles the execution results of agent tasks.
- **`graph`**: Defines graph structures for agent workflows.
- **`llm_interaction`**: Manages interactions with LLMs.
- **`logging`**: Provides logging services for agents.
- **`server`**: Contains the server implementation for handling agent requests.

### `mcp_runtime`

The `mcp_runtime` is responsible for managing the overall behavior of the swarm. It initializes the MCP client, manages the agent lifecycle, and orchestrates the execution of tasks. The key modules in this crate are:

- **`mcp_agent_logic`**: Contains the core logic for the MCP agent, including processing responses.
- **`mcp_client`**: Provides a client for interacting with the MCP server.
- **`mcp_tools`**: Contains the tools that the MCP agent can use to perform tasks.
- **`runtime`**: Manages the MCP runtime environment.

## Conclusion

The `swarm` codebase is well-structured and modular, which makes it easy to understand and maintain. The use of a Cargo workspace and the separation of concerns into distinct crates are key strengths of the architecture. This documentation provides a high-level overview of the project, and further details can be found in the source code and the individual crate documentation.
