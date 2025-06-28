# 🧠 Recall Plugin Design

> The Recall Plugin performs a two-phase enrichment strategy grounded in contextual prompting and graph-based personalization for LLM agents.

---

## 1. Overview

The Recall Plugin helps LLM agents retrieve and enrich user prompts with relevant, personalized knowledge from a graph database. It operates in two distinct phases: intent detection and context enrichment.

---

## 2. Phase 1: Intent Detection & Scope Determination

- **Input:**
    - User's natural language prompt
    - A summary list of node types and relation types in the user's GraphDB (not full data)
- **Action:**
    - Forwards the prompt and the list of node/relation names to the Handle (LLM-based orchestrator)
    - The Handle infers the user's intent and responds with a directive:
        - Which node types and relationships from the GraphDB are likely to help answer this prompt
        - A suggested query scope, not a raw answer
    - If the graph is empty or lacks relevant nodes, the Handle acknowledges the absence and adjusts response expectations accordingly

**Example:**

| Prompt | Graph Summary | Handle Response |
|--------|--------------|----------------|
| "I broke up with my lover. What should I do?" | ["school", "career", "preferences"] | "There is no relational data about the user's romantic partner, but general personal preferences exist. Generate advice based on personality and life history." |

---

## 3. Phase 2: Enriched Prompt Completion

- **Input:**
    - Original prompt
    - Queried data from the relevant subset of the user's GraphDB (as determined in Phase 1)
- **Action:**
    - The Recall Plugin enriches the prompt with this scoped context before passing it to the LLM for completion

---

## 4. Summary

The Recall Plugin enables:
- Personalized, context-aware LLM responses
- Efficient use of graph data by scoping queries to only relevant nodes/relations
- Graceful handling of missing or sparse user data

---

## See Also

- [Middleware Architecture](./middleware.md)
- [Lifecycle & Plugins](./lifecycle-and-plugins.md)
- [Plugin Development Guide](./plugin_development.md)
