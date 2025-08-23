# Swarm Codebase Structure

This document provides a comprehensive overview of the `swarm` codebase structure, outlining the key components and their interactions. The project is organized as a Cargo workspace, which allows for modular development and efficient dependency management.

## Project Root

The root of the project contains the main `Cargo.toml` file, which defines the workspace members and shared dependencies. This centralized configuration ensures consistency across all crates and simplifies the build process.

### Key Directories

- **`agent_discovery_service`**: This crate is responsible for discovering and registering agents within the swarm. It provides a centralized service that allows agents to find and communicate with each other.
- **`agent_memory_service`**: This component provides a memory service for agents, allowing them to store and retrieve information.
- **`agent_protocol_backbone`**: This crate defines the core agent trait and the communication protocols that govern agent interactions. It serves as the foundation for all agents in the swarm.
- **`basic_agent`**: This is a sample implementation of a basic agent that demonstrates how to build and integrate new agents into the swarm.
- **`configuration`**: This directory contains the configuration files for the swarm, including settings for agents, services, and the overall environment.
- **`documentation`**: This directory contains the project documentation, including this document.
- **`examples`**: This directory contains example code that demonstrates how to use the various components of the swarm.
- **`llm_api`**: This crate provides an interface to the Language Model (LLM) API, allowing agents to leverage its capabilities.
- **`mcp_runtime`**: This component is responsible for managing the Master Control Program (MCP) runtime, which orchestrates the overall behavior of the swarm.
- **`orchestration_agent`**: This agent is responsible for orchestrating complex tasks that require the coordination of multiple agents.
- **`workflow_agent`**: This agent is responsible for executing predefined workflows, coordinating tasks and agents as per the workflow definition.
- **`workflow_management`**: This crate provides functionalities for defining, managing, and orchestrating complex multi-agent workflows.

## Core Components

The core components of the swarm are the `agent_protocol_backbone` and the `mcp_runtime`. These two crates provide the foundation for the entire system and are essential for its operation.

### `agent_protocol_backbone`

The `agent_protocol_backbone` is the heart of the swarm. It defines the `Agent` trait, which all agents must implement, and the communication protocols that enable agents to interact with each other. The key modules in this crate are:

- **`business_logic`**: This module contains the core business logic for the agents, including the `Agent` trait and the `handle_request` method.
- **`config`**: This module is responsible for loading and managing the agent configuration.
- **`planning`**: This module defines the data structures used for planning and executing tasks.
- **`server`**: This module contains the server implementation that listens for and responds to agent requests.

### `mcp_runtime`

The `mcp_runtime` is responsible for managing the overall behavior of the swarm. It initializes the MCP client, manages the agent lifecycle, and orchestrates the execution of tasks. The key modules in this crate are:

- **`mcp_agent_logic`**: This module contains the core logic for the MCP agent, including the `execute_loop` method.
- **`mcp_client`**: This module provides a client for interacting with the MCP server.
- **`mcp_tools`**: This module contains the tools that the MCP agent can use to perform tasks.

## Conclusion

The `swarm` codebase is well-structured and modular, which makes it easy to understand and maintain. The use of a Cargo workspace and the separation of concerns into distinct crates are key strengths of the architecture. This documentation provides a high-level overview of the project, and further details can be found in the source code and the individual crate documentation.
