# 🤖 LLM Extraction Framework in TelaMentis

TelaMentis is designed to seamlessly integrate with Large Language Models (LLMs) to transform unstructured or semi-structured text data—often from AI agent conversations or documents—into structured knowledge graph elements (`Nodes` and `TimeEdge`s). This process is called LLM Extraction.

## 1. Purpose and Vision

The goal of LLM extraction in TelaMentis is to:

*   **Automate Knowledge Graph Population**: Enable AI agents to "learn" from their interactions and external data sources by automatically updating their knowledge graph memory.
*   **Bridge Unstructured and Structured Worlds**: Convert natural language text into queryable graph structures.
*   **Enable Richer Agent Capabilities**: Allow agents to build and maintain complex mental models of their environment, users, and tasks.

## 2. The `LlmConnector` Trait

The core abstraction for LLM integration is the `LlmConnector` trait defined in `TelaMentis-core`. Any LLM provider can be integrated by implementing this trait.

```rust
// Simplified from TelaMentis-core/src/llm.rs
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use crate::types::TenantId; // Assuming TenantId is available

// Represents a user or assistant message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmMessage {
    pub role: String, // "user", "assistant", "system"
    pub content: String,
}

// Context passed to the LLM for extraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionContext {
    pub messages: Vec<LlmMessage>,
    pub system_prompt: Option<String>, // Overrides default or provides specific instructions
    pub desired_schema: Option<String>, // Optional: string representation of desired JSON schema
                                      // Potentially other context: current_time, user_profile, etc.
}

// Candidate for a node to be created/updated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionNode {
    pub id_alias: String, // User-defined ID, used for deduplication
    pub label: String,
    pub props: serde_json::Value,
    pub confidence: Option<f32>, // Optional: LLM's confidence in this extraction
}

// Candidate for a relation to be created/updated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionRelation {
    pub from_id_alias: String, // Refers to id_alias of a node
    pub to_id_alias: String,   // Refers to id_alias of a node
    pub type_label: String,         // Type of the relationship (e.g., "WORKS_FOR")
    pub props: serde_json::Value,
    // For temporal relations, valid_from/valid_to might be in props or dedicated fields
    pub valid_from: Option<chrono::DateTime<chrono::Utc>>,
    pub valid_to: Option<chrono::DateTime<chrono::Utc>>,
    pub confidence: Option<f32>,
}

// Metadata about the extraction process
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExtractionMetadata {
    pub provider: String,
    pub model_name: String,
    pub latency_ms: Option<u64>,
    pub input_tokens: Option<u32>,
    pub output_tokens: Option<u32>,
    pub cost_usd: Option<f64>, // If available
    pub warnings: Vec<String>,
}

// The structured output expected from the LLM (or parsed from its output)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionEnvelope {
    pub nodes: Vec<ExtractionNode>,
    pub relations: Vec<ExtractionRelation>,
    pub meta: Option<ExtractionMetadata>, // Populated by the connector or core
}

impl ExtractionEnvelope {
    // Helper to provide the schema LLMs should adhere to
    pub fn json_schema_example() -> &'static str {
        r#"{
  "nodes": [
    {"id_alias": "string (unique within this extraction)", "label": "string (e.g., Person, Organization)", "props": {"key": "value", ...}, "confidence": "float (0.0-1.0, optional)"}
  ],
  "relations": [
    {"from_id_alias": "string (refers to node id_alias)", "to_id_alias": "string (refers to node id_alias)", "type_label": "string (e.g., WORKS_FOR)", "props": {"key": "value", ...}, "valid_from": "datetime (ISO8601, optional)", "valid_to": "datetime (ISO8601, optional, null for open)", "confidence": "float (0.0-1.0, optional)" }
  ]
}"#
    }
}


#[derive(Debug, thiserror::Error)]
pub enum LlmError {
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("API error from LLM provider: {0}")]
    ApiError(String),
    #[error("Timeout during LLM call")]
    Timeout,
    #[error("Failed to parse LLM response: {0}")]
    ResponseParseError(String),
    #[error("LLM response failed schema validation: {0}")]
    SchemaValidationError(String),
    #[error("Extraction budget exceeded")]
    BudgetExceeded,
    #[error("Internal connector error: {0}")]
    InternalError(String),
}


#[async_trait]
pub trait LlmConnector: Send + Sync {
    /// Receives context (e.g., last N messages, system prompt),
    /// returns a typed extraction envelope.
    async fn extract(&self, tenant: &TenantId, context: ExtractionContext) -> Result<ExtractionEnvelope, LlmError>;

    /// Optional: run an arbitrary completion (fallback for dev tools, testing).
    // async fn complete(&self, tenant: &TenantId, req: CompletionReq) -> Result<String, LlmError>;
}
```

**Concrete Implementations (Plugins):**
*   `connectors/openai`: Uses OpenAI's Chat Completions API (e.g., GPT-4o, GPT-3.5-turbo). Often leverages function calling or JSON mode for structured output.
*   `connectors/anthropic`: Uses Anthropic's Claude models.
*   `connectors/gemini`: (Planned) For Google's Gemini models.

## 3. The Extraction Pipeline

The process of extracting knowledge using LLMs typically follows these steps:

1.  **Context Building (Core / Application Logic)**:
    *   Gathers relevant information to send to the LLM. This might include:
        *   The last *K* messages from an agent-user conversation.
        *   A document or text snippet to be processed.
        *   The current date/time (for resolving temporal references).
        *   User profile information or other contextual data.

2.  **Prompt Engineering & Formatting (Connector / Core)**:
    *   A **System Prompt** is constructed. This is critical and instructs the LLM on its role, the desired output format (JSON matching `ExtractionEnvelope`), and any constraints.
        ```text
        You are an expert knowledge graph extraction engine. Analyze the provided text/conversation.
        Identify relevant entities (as nodes) and relationships (as relations) between them.
        Return your findings strictly as a JSON object matching the following schema:
        {
          "nodes": [ {"id_alias":"string", "label":"string", "props":object, "confidence": float} ],
          "relations": [ {"from_id_alias":"string", "to_id_alias":"string", "type_label":"string", "props":object, "valid_from": "datetime", "valid_to": "datetime", "confidence": float} ]
        }
        - `id_alias` should be a descriptive, unique identifier for nodes within this extraction (e.g., "user_john_doe", "acme_corp_hq").
        - If a date or time for `valid_from` or `valid_to` is mentioned, use ISO8601 format. If a relation is ongoing, `valid_to` can be omitted or null.
        - Only extract explicitly mentioned information. Do not infer or hallucinate.
        - If unsure about a piece of information, omit it or assign a low confidence score.
        ```
    *   The user messages/text are formatted according to the LLM provider's API (e.g., list of messages with roles).

3.  **LLM API Call (`LlmConnector::extract`)**:
    *   The `LlmConnector` sends the formatted prompt and context to the configured LLM.
    *   Handles authentication, network requests, timeouts, and retries.

4.  **Response Parsing & Validation (Connector / Core)**:
    *   The LLM's response (ideally JSON text) is received.
    *   The connector attempts to parse this text into the `ExtractionEnvelope` struct.
    *   **Schema Validation**: The parsed structure is validated against the expected schema. If it fails (e.g., missing fields, incorrect types), it's an error. This is a key step in mitigating malformed output.

5.  **Deduplication & Merging (Core / `GraphService`)**:
    *   Before writing to the graph, the extracted `ExtractionNode`s and `ExtractionRelation`s are processed:
        *   **Nodes**: The `id_alias` from `ExtractionNode` is used to query the `GraphStore` (for the current tenant) to see if a node with that alias already exists.
            *   If exists: Properties might be merged (e.g., new props added, existing ones updated based on a strategy like "last write wins" or more complex logic).
            *   If not exists: A new node is prepared for creation.
        *   **Relations**: Similar logic, using `from_id_alias` and `to_id_alias` to resolve to existing or newly identified node UUIDs. The uniqueness of a relation might depend on its type, properties, and validity period.
    *   **World Assumption**: This stage can apply an "Open World Assumption" (new information is added) or a "Closed World Assumption" (information not explicitly stated is false, leading to potential retractions – more complex). TelaMentis typically defaults to an Open World approach for LLM extractions.

6.  **Batch Upsert to `GraphStore` (Core / `GraphService`)**:
    *   All validated and deduplicated nodes and relations (now as `Node` and `TimeEdge` structs) are sent to `GraphStore::upsert_node` and `GraphStore::upsert_edge` in a batch.
    *   The `GraphStore` adapter handles the actual database writes, including bitemporal versioning.

7.  **Audit Trail & Metadata Storage (Core / Connector)**:
    *   The `ExtractionMetadata` (provider, model, latency, token counts, cost) is valuable for monitoring, debugging, and cost tracking.
    *   This metadata can be:
        *   Stored as properties on the created/updated nodes/edges (e.g., `_llm_source_model: "gpt-4o"`).
        *   Written to a separate audit log or metrics system.

## 4. Safety, Hallucination Mitigation, and Cost Control

Working with LLMs requires attention to several practical concerns:

*   **Hallucination Mitigation**:
    *   **Strict Prompting**: Instructing the LLM to only extract explicit information and not infer.
    *   **Schema Enforcement**: Rejecting any output that doesn't conform to the `ExtractionEnvelope` JSON schema. This is a strong filter.
    *   **Confidence Scoring**: If the LLM provides confidence scores (or if they can be derived), relations/nodes below a certain threshold can be flagged for human review or treated with caution.
    *   **Grounding (Future)**: Providing the LLM with access to existing graph data to ground its extractions.
*   **Token Limits & Output Control**:
    *   Set `max_tokens` in LLM API calls to prevent excessively long (and costly) responses.
    *   Prompt the LLM to be concise.
*   **Cost-Aware Routing & Budgeting**:
    *   TelaMentis can implement a routing layer to select LLM providers/models based on:
        *   **Pre-defined Budgets**: If a daily/monthly budget for a provider (e.g., OpenAI) is exhausted, route to a fallback (e.g., Anthropic or a cheaper model).
        *   **Latency Requirements**: Prefer local/faster models for time-sensitive extractions.
        *   **Cost-Effectiveness**: Use cheaper models for routine tasks and more powerful/expensive models for complex extractions.
    *   Configuration for routing rules can live in `config.yaml`:
      ```yaml
      llm:
        default_provider: openai # Default if no rules match or for general use
        routing_rules:
          - priority: 1
            provider: openai
            model: gpt-4o-mini
            # conditions: (e.g., task_type: "simple_extraction")
            budget_per_day_usd: 20.00 # Optional: daily budget for this rule
          - priority: 2
            provider: anthropic
            model: claude-3-haiku
            fallback_for: ["openai/gpt-4o-mini"] # If openai/gpt-4o-mini budget is hit
        providers:
          openai:
            api_key: ${OPENAI_API_KEY} # Loaded from env
            # Default model if not specified in routing
            default_model: gpt-3.5-turbo
          anthropic:
            api_key: ${ANTHROPIC_API_KEY}
            default_model: claude-3-sonnet
      ```

## 5. Handling Temporal Information

LLMs can often extract temporal information ("event X happened on Y date", "Z was valid until T").
*   The prompt should instruct the LLM to include `valid_from` and `valid_to` in ISO8601 format within the `relations` part of the JSON output if such information is present in the text.
*   The `ExtractionRelation` struct has optional `valid_from` and `valid_to` fields.
*   The core logic then maps these to `TimeEdge`'s bitemporal properties.

## 6. Iteration and Improvement

LLM-based extraction is an iterative process:
*   **Prompt Tuning**: Continuously refine system prompts for better accuracy and adherence to the schema.
*   **Model Selection**: Experiment with different LLM models to find the best balance of cost, performance, and quality.
*   **Feedback Loops**: Incorporate mechanisms for users to correct or validate LLM extractions, which can then be used to fine-tune prompts or even specialized models (future).

By providing a robust framework for LLM extraction, TelaMentis enables AI agents to build and maintain rich, dynamic knowledge graphs from the diverse information they encounter. 