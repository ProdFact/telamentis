# 🤖 AI Agent Integration Patterns with TelaMentis

TelaMentis serves as a durable, real-time, and temporally-aware memory for AI agents. This document outlines common patterns for integrating various types of AI agents with TelaMentis to enhance their capabilities.

## 1. Core Agent Interactions with TelaMentis

Regardless of the agent type, common interactions include:

*   **Learning/Memory Formation**:
    *   Agents process new information (user input, documents, observations).
    *   This information is transformed into `Node`s and `TimeEdge`s, often via the **LLM Extraction Pipeline**.
    *   `GraphStore::upsert_node` and `GraphStore::upsert_edge` persist this knowledge.
*   **Memory Retrieval/Context Augmentation**:
    *   Before an agent responds or acts, it queries TelaMentis for relevant context.
    *   `GraphStore::query` is used, often with temporal constraints (e.g., "as-of" a certain time, or "currently valid facts").
    *   Retrieved graph data augments prompts for LLMs, improving response quality and grounding.
*   **Reflection/Summarization**:
    *   Periodically, agents might query their knowledge graph to synthesize summaries, identify patterns, or consolidate memories. These summaries can also be stored back into TelaMentis as new nodes/edges.

## 2. Agent Archetypes & Integration Strategies

### 2.1. Conversational AI Agents (Chatbots)

*   **Goal**: Maintain conversation history, user preferences, and learned facts for coherent, personalized interactions.
*   **Memory Formation**:
    *   Each user message and agent response can be stored as a `Message` node.
        *   `User` --(`SENT_MESSAGE {timestamp: T_msg}` `valid_from: T_msg`)--> `Message_User_1`
        *   `Agent` --(`SENT_MESSAGE {timestamp: T_msg}` `valid_from: T_msg`)--> `Message_Agent_1`
        *   `Message_Agent_1` --(`IN_REPLY_TO {timestamp: T_msg}` `valid_from: T_msg`)--> `Message_User_1`
    *   LLM Extraction runs on conversation snippets to extract:
        *   **User Preferences**: `User` --(`PREFERS {topic: "SciFi", strength: 0.8}` `valid_from: T_learn`)--> `Topic_SciFi`
        *   **Key Entities Mentioned**: `Message_User_1` --(`MENTIONS` `valid_from: T_msg`)--> `Entity_AcmeCorp`
        *   **Facts Stated**: `User` --(`STATED_FACT {fact: "lives in London"}` `valid_from: T_learn`)--> `FactNode_LivesInLondon` (or directly as edge: `User` --(`LIVES_IN` `valid_from: T_learn`)--> `City_London`)
*   **Memory Retrieval**:
    *   **Short-term Context**: Retrieve last N messages directly from a simple cache or short-term store.
    *   **Long-term Context**: Before generating a response, query TelaMentis:
        *   "Retrieve `PREFERS` edges for this `User`."
        *   "Retrieve recently mentioned `Entity` nodes by this `User`."
        *   "What `FactNode`s related to the current conversation topic were previously stated by this `User`?"
    *   Temporal queries: "What did this `User` prefer regarding `Topic_Travel` last summer?"
        *   `MATCH (u:User {id_alias: $user_id})-[r:PREFERS]->(t:Topic {name:"Travel"}) WHERE r.valid_from <= '2023-08-31' AND (r.valid_to IS NULL OR r.valid_to >= '2023-06-01') RETURN r.props`
*   **Benefit**: Agents remember past interactions, adapt to user preferences over time, and avoid repeating questions.

### 2.2. RAG (Retrieval Augmented Generation) Agents

*   **Goal**: Ground LLM responses in factual knowledge from a corpus, represented in TelaMentis.
*   **Memory Formation (Corpus Ingestion)**:
    *   Documents are processed, chunked, and entities/relationships are extracted.
        *   `Document` --(`HAS_CHUNK`)--> `Chunk`
        *   `Chunk` --(`MENTIONS_TERM {tf_idf: 0.2}` `valid_from: T_ingest`)--> `Term_KnowledgeGraph`
        *   `Entity_A` --(`APPEARS_WITHIN_SENTENCE_WITH` `valid_from: T_ingest`)--> `Entity_B` (collocation)
        *   (Optional) Store embeddings for `Chunk` nodes or `Entity` nodes for semantic search.
*   **Memory Retrieval**:
    1.  User query is received.
    2.  **Candidate Retrieval**: Query TelaMentis for `Chunk`s or `Document`s related to query terms or semantically similar concepts (if embeddings are used).
        *   This might involve graph traversal: "Find `Chunk`s connected to `Term`s from the user query, then expand to related `Term`s or `Entity`s."
    3.  **Context Augmentation**: Retrieved text from `Chunk`s is added to the LLM prompt along with the original user query.
*   **Temporal RAG**:
    *   If documents have publication dates or versions: `Document {publish_date: T_pub}`.
    *   Queries can be filtered: "Retrieve information about 'quantum computing' from `Document`s published before 2020."
*   **Benefit**: Reduces LLM hallucination, provides up-to-date information (if corpus is updated), and allows citation of sources.

### 2.3. ReAct (Reasoning + Acting) Agents / Task-Oriented Agents

*   **Goal**: Decompose complex tasks, plan steps, execute actions (tools), and learn from outcomes.
*   **Memory Formation**:
    *   **Task Decomposition**: `Task_Main` --(`DECOMPOSED_INTO` `valid_from: T_plan`)--> `SubTask_1`
    *   **Tool Usage**: `Agent` --(`USED_TOOL {tool_name: "WebSearch", params: {...}}` `valid_from: T_act, valid_to: T_act_end`)--> `ToolExecution_1`
    *   **Observations**: `ToolExecution_1` --(`PRODUCED_OBSERVATION {content: "..."}` `valid_from: T_obs`)--> `Observation_1`
    *   **Learned Outcomes/Strategies**:
        *   If a plan succeeds: `Plan_A` --(`LED_TO_SUCCESS {metric: 0.9}` `valid_from: T_learn`)--> `Goal_X`
        *   If a tool fails: `Tool_WebSearch` --(`FAILED_WITH_PARAMS {params: ..., error: "..."}` `valid_from: T_fail`)--> `ErrorNode_Timeout`
*   **Memory Retrieval for Planning/Reflection**:
    *   "What `SubTask`s are pending for `Task_Main`?"
    *   "What `Observation`s were made during the last attempt to achieve `Goal_X`?"
    *   "Which `Tool`s have previously `FAILED_WITH_PARAMS` similar to the current situation?"
    *   "Retrieve successful `Plan`s for `Goal`s similar to the current one." (Requires similarity metric).
*   **Temporal Aspects**:
    *   Track task execution over time: "How long did `SubTask_1` take?" (derived from `valid_from`/`valid_to` of action edges).
    *   "What was the state of `Task_Main` as of yesterday?"
*   **Benefit**: Agents can learn from past successes/failures, improve planning, and adapt strategies.

### 2.4. Self-Improving / Reflective Agents (e.g., inspired by MemGPT)

*   **Goal**: Agents that can reflect on their own memory, synthesize higher-level knowledge, and manage memory tiers.
*   **Memory Formation**: Similar to conversational agents, but with more emphasis on internal states and thoughts.
    *   `Agent` --(`HAD_THOUGHT {content: "..."}` `valid_from: T_thought`)--> `ThoughtNode_1`
    *   `ThoughtNode_1` --(`TRIGGERED_BY`)--> `Message_User_X`
*   **Reflection Process (can be a periodic background task or triggered by events)**:
    1.  **Query**: Retrieve recent interactions, thoughts, extracted entities within a time window.
        *   `MATCH (t:ThoughtNode) WHERE t.valid_from >= $startTime AND t.valid_from < $endTime RETURN t`
    2.  **Summarize/Abstract**: Pass retrieved data to an LLM with a prompt to summarize, find patterns, or generate insights.
        *   Prompt: "Based on these thoughts and interactions, what are the key themes or unresolved questions?"
    3.  **Store Synthesis**: Store the LLM's output back into TelaMentis as new, higher-level nodes/edges.
        *   `ReflectionSummary_1 {theme: "User planning a trip", period_start: T1, period_end: T2}`
        *   `User` --(`SHOWED_INTEREST_IN_TOPIC {topic: "Paris Travel", strength: HIGH}` `valid_from: T_reflect_start, valid_to: T_reflect_end`)--> `Topic_Paris`
*   **Benefit**: Enables agents to build a deeper understanding over time, manage information overload, and potentially discover novel insights from their own experiences.

## 3. Using the LLM Extraction Framework

Most agent memory formation patterns will heavily rely on TelaMentis's LLM Extraction framework:
1.  Agent gathers raw text (conversation, document, observation).
2.  Prepares an `ExtractionContext` (messages, system prompt instructing JSON output).
3.  Calls an `LlmConnector::extract`.
4.  The `LlmConnector` returns an `ExtractionEnvelope` (nodes, relations).
5.  Core TelaMentis logic (or the agent itself) then performs deduplication (using `id_alias`) and upserts the data into the `GraphStore`.

## 4. Key Considerations for Agent Developers

*   **Tenant ID**: Ensure all agent interactions with TelaMentis are scoped by the correct `TenantId` (e.g., representing the end-user or the agent's operational context).
*   **Schema Design**: Refer to the [Schema Design Guide](./schema_design_guide.md) to model agent memories effectively.
*   **Prompt Engineering**: Crucial for both LLM extraction and for generating good agent responses based on retrieved graph context.
*   **Error Handling & Retries**: Implement robust error handling for API calls to TelaMentis and LLM services.
*   **Cost Management**: Be mindful of LLM costs associated with extraction and summarization. Use cost-effective models where possible. TelaMentis's LLM routing can help.
*   **Performance**: Optimize queries to TelaMentis. Cache frequently accessed, less volatile data if appropriate.

By integrating with TelaMentis, AI agents can transcend statelessness and develop rich, evolving memories, leading to more intelligent, adaptive, and personalized behaviors.