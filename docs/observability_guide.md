# 📊 TelaMentis Observability Guide

Effective observability is critical for understanding the behavior, performance, and health of a TelaMentis deployment. This guide outlines best practices for logging, metrics, and tracing within TelaMentis and its ecosystem of plugins.

## 1. Pillars of Observability

TelaMentis aims to support the three pillars of observability:

1.  **Logging**: Recording discrete events with contextual information. Useful for debugging errors and understanding specific request flows.
2.  **Metrics**: Aggregated numerical data representing the health and performance of the system over time. Useful for dashboards, alerting, and trend analysis.
3.  **Tracing**: Tracking a single request as it flows through various components of the distributed system. Useful for identifying bottlenecks and understanding component interactions.

## 2. Logging

### 2.1. Structured Logging
*   **Format**: All logs should be emitted in a structured format, preferably JSON. This allows for easier parsing, filtering, and analysis by log management systems (e.g., ELK Stack, Splunk, Grafana Loki).
*   **Rust Crate**: Use crates like `tracing` with `tracing-subscriber` and formatters like `tracing-bunyan-formatter` or `tracing-logfmt` configured for JSON output.

### 2.2. Key Log Fields
Every log entry should ideally include:
*   `timestamp`: ISO8601 UTC timestamp of the event.
*   `level`: Log level (e.g., `ERROR`, `WARN`, `INFO`, `DEBUG`, `TRACE`).
*   `message`: The human-readable log message.
*   `target` or `module`: The Rust module/target path (e.g., `TelaMentis_core::graph_service`).
*   `tenant_id`: (If applicable) The `TenantId` associated with the operation.
*   `request_id` or `correlation_id`: A unique ID to trace a single request across multiple log entries and components.
*   `span_id`, `trace_id`: (If using distributed tracing) OpenTelemetry compatible IDs.
*   Component-specific fields (e.g., `operation_name`, `duration_ms`, `error_details`).

### 2.3. Log Levels
*   **ERROR**: Critical errors that prevent normal operation (e.g., database connection failure, unrecoverable panic). Should always be investigated.
*   **WARN**: Potential issues or unexpected situations that don't necessarily stop operation but might indicate future problems (e.g., deprecated API usage, retryable error).
*   **INFO**: General operational information (e.g., service startup, significant configuration loaded, request completion for key operations).
*   **DEBUG**: Detailed information useful for debugging specific issues (e.g., intermediate steps in a complex operation, detailed request/response payloads - be careful with PII).
*   **TRACE**: Extremely verbose logging, typically only enabled for deep troubleshooting (e.g., entry/exit of many functions).

### 2.4. Correlation
*   A `request_id` should be generated at the entry point (Presentation Layer) and propagated through all layers (Core, Adapters). This is vital for tracing the lifecycle of a single request.

## 3. Metrics

### 3.1. Metric Types
*   **Counters**: Values that only increase (e.g., number of requests, errors, nodes created).
*   **Gauges**: Values that can go up or down (e.g., active connections, queue length, memory usage).
*   **Histograms/Summaries**: Track the distribution of values (e.g., request latencies, payload sizes). Useful for calculating percentiles (p50, p90, p99).

### 3.2. Key Metrics to Monitor

**TelaMentis Core & Presentation Layer:**
*   `requests_total`: Counter, tagged by `endpoint`, `method`, `status_code`, `tenant_id`.
*   `request_latency_seconds`: Histogram, tagged by `endpoint`, `method`, `tenant_id`.
*   `active_requests`: Gauge.
*   `graph_operations_total`: Counter, tagged by `operation_type` (e.g., `upsert_node`, `query`), `status` (success/failure), `tenant_id`.
*   `graph_operation_latency_seconds`: Histogram, tagged by `operation_type`, `tenant_id`.
*   `plugin_execution_latency_seconds`: Histogram, tagged by `plugin_name`, `stage` (pre/op/post), `tenant_id` (for request pipeline plugins).

**Storage Adapters (`GraphStore`):**
*   `db_query_latency_seconds`: Histogram, tagged by `query_type` (e.g., `MERGE_NODE`, `MATCH_EDGE_TEMPORAL`), `tenant_id`.
*   `db_errors_total`: Counter, tagged by `error_type`, `tenant_id`.
*   `db_connection_pool_active_connections`: Gauge.
*   `db_connection_pool_idle_connections`: Gauge.

**LLM Connector Adapters (`LlmConnector`):**
*   `llm_requests_total`: Counter, tagged by `provider`, `model_name`, `status_code`, `tenant_id`.
*   `llm_request_latency_seconds`: Histogram, tagged by `provider`, `model_name`, `tenant_id`.
*   `llm_input_tokens_total`: Counter, tagged by `provider`, `model_name`, `tenant_id`.
*   `llm_output_tokens_total`: Counter, tagged by `provider`, `model_name`, `tenant_id`.
*   `llm_estimated_cost_usd_total`: Counter, tagged by `provider`, `model_name`, `tenant_id`. (If calculable).

**Source Adapters (`SourceAdapter`):**
*   `source_messages_received_total`: Counter, tagged by `source_type`, `tenant_id`.
*   `source_mutations_produced_total`: Counter, tagged by `source_type`, `mutation_type`, `tenant_id`.
*   `source_processing_errors_total`: Counter, tagged by `source_type`, `error_type`, `tenant_id`.
*   `source_processing_queue_length`: Gauge, tagged by `source_type`, `tenant_id`.

### 3.3. Tooling
*   **Rust Crates**: `metrics` crate facade with an exporter like `metrics-exporter-prometheus`.
*   **Collection & Storage**: Prometheus is a common choice.
*   **Visualization**: Grafana for dashboards.

## 4. Tracing

### 4.1. Distributed Tracing
*   **Purpose**: To follow a single request's journey across service boundaries (e.g., AI Agent -> FastAPI Presentation -> Rust Core -> Neo4j Adapter -> Neo4j DB).
*   **Mechanism**:
    *   Assign a unique `trace_id` to each incoming request.
    *   Each component/operation creates a `span` (with its own `span_id` and parent `span_id`).
    *   Spans are tagged with relevant attributes (e.g., HTTP method, DB query, LLM model).
    *   Context propagation (trace headers like W3C Trace Context) is essential.

### 4.2. Tooling
*   **Rust Crates**: `tracing` with OpenTelemetry integration (`tracing-opentelemetry`).
*   **Backends**: Jaeger, Zipkin, Grafana Tempo, AWS X-Ray.
*   **Visualization**: UI provided by the tracing backend to view trace waterfalls.

### 4.3. What to Trace
*   Incoming requests at the Presentation Layer.
*   Calls from Presentation Layer to TelaMentis Core.
*   Core business logic operations (e.g., `GraphService` methods).
*   Calls from Core to `GraphStore` adapters.
*   Calls from Core to `LlmConnector` adapters.
*   Calls from Core to `SourceAdapter` (if applicable in a request path).
*   Database queries within `GraphStore` adapters.
*   API calls within `LlmConnector` adapters.
*   Execution of each plugin in the Request Processing Pipeline.

## 5. Alerting

Set up alerts based on metrics to proactively identify issues:
*   High error rates (e.g., >5% HTTP 5xx errors).
*   High request latency (e.g., p99 latency > 1s for critical endpoints).
*   Database connection pool saturation.
*   LLM API error spikes or budget overruns.
*   Critical errors in logs.
*   Resource exhaustion (CPU, memory, disk).

## 6. Plugin Authors' Responsibilities

Plugin authors should:
*   Utilize the `tracing` crate for logging within their plugin.
*   Emit relevant metrics (e.g., execution time, errors, custom plugin-specific metrics) using the `metrics` facade.
*   Ensure their plugin correctly propagates tracing context if it makes further outbound calls.
*   Document any specific observability considerations for their plugin.

## 7. Dashboarding

Create dashboards (e.g., in Grafana) to visualize key metrics:
*   **Overview Dashboard**: Overall system health, request rates, error rates, key latencies.
*   **Per-Tenant Dashboard**: Filtered view of key metrics for a specific tenant.
*   **Component Dashboards**: Detailed metrics for Core, specific adapters (Storage, LLM), Presentation Layer.
*   **Error Analysis Dashboard**: Breakdown of error types, affected endpoints/tenants.

By implementing comprehensive observability, you can ensure your TelaMentis deployment is robust, performant, and easier to troubleshoot, ultimately leading to a more reliable experience for AI agents and users.