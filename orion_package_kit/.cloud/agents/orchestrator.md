# Agent Specification — Orchestrator

The **Orchestrator** is the central brain of the mcpkg cloud ecosystem. It coordinates installation flows and ensures all security checks are performed by specialized sub-agents.

## Primary Responsibilities

- **Pipeline Management**: Directs the `mcpkg-pipeline` state machine.
- **Workflow Coordination**: Spawns sub-agents for specialized tasks like security auditing or package rebuilding.
- **Failure Analysis**: Handles rollbacks and error reporting across the entire pipeline.

## Capabilities

- Read and parse `manifest.toml`.
- Execute and monitor terminal commands for package installation.
- Verify state transitions across the pipeline.
