# 🧩 Middleware Architecture for Long-Term Memory in LLM Agents

> This document describes a modular, plugin-driven middleware system for building cognitive memory infrastructure in LLM agents, using graph-based storage and context-aware pipelines.

---

## 1. Overview

This architecture enables LLM agents to leverage graph-based knowledge storage (Neo4j or Memgraph), LLM-powered semantic enrichment, and a flexible plugin-based middleware system inspired by ASP.NET Core.

---

## 2. From State Machine to Contextual Middleware

- The flow is **not fixed**.
- Execution depends on message `type`, available `services`, and the evolving `context`.
- Each plugin performs a task, modifies the context, and decides whether to continue or halt the pipeline.

---

## 3. Message Structure

```json
{
  "type": "conversation" | "batch_conversation" | "mood" | "preference_save" | ...,
  "payload": { ... },
  "user_id": "abc123",
  "metadata": {
    "timestamp": "...",
    "status": "default" | "error" | "exit" | "on_exit"
  }
}
```

---

## 4. Context Structure

```python
Context = {
  user_id: str,
  request: {...},
  response: {...},
  persistible_data: Dict[type, Any],
  services: Dict[interface, implementation],
  metadata: {
    status: "default" | "error" | "exit" | "on_exit",
    trace: [...],
    errors: [...]
  }
}
```

---

## 5. Middleware Plugins

Plugins are modular, stateless functions or classes that:
- Receive and mutate the `Context`
- Request services (e.g., `iUserProvider`, `iApiKeyProvider`)
- Optionally read/write `persistible_data` (e.g., GraphData, ConversationSummary)
- Handle their own errors or propagate status changes

**Example plugin types:**
- `ValidateUserId`
- `EnrichWithMemory`
- `LLMExtractEntities`
- `DeduplicateGraphNodes`
- `MergeToGraph`
- `SimpleStorage`
- `DomainRuleProcessor`
- `ObservabilityWrapper`

---

## 6. Plugin Control & Orchestration

- A PluginRunner resolves which plugins to invoke based on message type and context
- Plugins declare what interfaces or context keys they depend on
- Middleware manager tracks:
    - execution order
    - errors thrown or handled
    - final context state

---

## 7. Flow Control

Plugins can:
- Exit pipeline on error or invalid state
- Continue on recoverable conditions
- Trigger alternate plugin paths (e.g., fallback, retry, error resolution chains)

This provides **graceful degradation** and **partial processing** when complete processing is not possible.

---

## 8. Service & Dependency Model

Context serves as a **Service Locator**. Plugins can request services such as:
- `iUserProvider`
- `iApiKeyProvider`
- `iContextLogger`
- `iConfigService`

This enables separation of concerns, runtime configurability, and testability.

---

## 9. Observability & Metrics

- Wrap all plugin calls with timing, tracing, and exception tracking
- Persist or stream trace logs to a monitoring dashboard
- Support per-plugin execution metadata (duration, success/failure, retries)

---

## 10. Memory Infrastructure

The memory layer remains graph-based, with:
- Entity and relation extraction from LLMs
- Node deduplication via embedding similarity
- Flexible insertion paths (Neo4j or Memgraph)
- Weekly summarization support for memory compression

---

## 11. Advanced Features (Planned)

- Plugin Dependency Graph: resolve ordering based on declared needs
- Plugin Config: per-type and per-user plugin chains
- Parallel Plugin Execution: fork-join models for batch processing
- Adaptive Pipelines: dynamically adjust execution path based on real-time context state
- Introspection: expose context and plugin state as GraphQL or REST for UI observability

---

## 12. Summary

This architecture builds a **modular, pluggable, and context-aware middleware system** for agent memory and task processing. It is not a rigid state machine, but a distributed, adaptive pipeline where plugins operate over a shared evolving context—allowing scalable, personalized, and reflective AI agents to emerge.

---

## See Also

- [Lifecycle & Plugins](./lifecycle-and-plugins.md)
- [Plugin Development Guide](./plugin_development.md)
- [LLM Extraction Framework](./llm_extraction.md) 