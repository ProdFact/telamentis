# 🕰️ Advanced Temporal Reasoning with TelaMentis

TelaMentis's bitemporal foundation (`valid_time` and `transaction_time` on `TimeEdge`s) enables powerful temporal queries. Beyond basic "as-of" and "as-at" lookups, this document explores more advanced temporal reasoning patterns that can enhance AI agent capabilities.

## 1. Recap: Bitemporal Primitives

*   **`valid_from` (VF)**: When a fact became true in the modeled world.
*   **`valid_to` (VT)**: When a fact ceased to be true (or `None` for ongoing). The interval `[VF, VT)` represents the valid duration.
*   **`transaction_start_time` (TS)**: When this version of the fact was recorded in the database.
*   **`transaction_end_time` (TE)**: When this version was superseded/logically deleted (`None` for current version). The interval `[TS, TE)` represents database presence.

This guide primarily focuses on reasoning over `valid_time`, assuming queries operate on the current state of database knowledge (i.e., where `TE` is `None`).

## 2. Sequencing and Ordering

Determining the order of events or state changes.

*   **"Did event A happen before event B?"**
    *   Requires identifying `TimeEdge`s representing A and B.
    *   Compare `A.valid_from` and `B.valid_from`.
    *   Example: `UserLoggedInEdge.valid_from < ItemPurchasedEdge.valid_from`.
*   **"What was the sequence of status changes for Order X?"**
    *   Query all `TimeEdge`s related to Order X's status (e.g., `Order` --(`HAS_STATUS {status: "value"}`)--> `StatusConcept`), order by `valid_from`.
    ```cypher
    MATCH (o:Order {id: "X"})-[r:HAS_STATUS]->()
    RETURN r.props.status, r.valid_from, r.valid_to
    ORDER BY r.valid_from
    ```

## 3. Allen's Interval Algebra

Allen's Interval Algebra defines 13 basic relations between two time intervals (e.g., `[A_VF, A_VT)` and `[B_VF, B_VT)`). TelaMentis queries can be constructed to find these relationships.

Let interval A be `[A_VF, A_VT)` and interval B be `[B_VF, B_VT)`. (`A_VT` or `B_VT` can be infinite/`None`).

*   **A `equals` B**: `A_VF = B_VF AND A_VT = B_VT`
*   **A `precedes` B**: `A_VT < B_VF`
*   **A `meets` B**: `A_VT = B_VF`
*   **A `overlaps` B**: `A_VF < B_VF AND A_VT > B_VF AND A_VT < B_VT`
*   **A `during` B**: `A_VF > B_VF AND A_VT < B_VT`
*   **A `starts` B**: `A_VF = B_VF AND A_VT < B_VT`
*   **A `finishes` B**: `A_VF > B_VF AND A_VT = B_VT`
    (And their inverses: `preceded_by`, `met_by`, `overlapped_by`, `contains`, `started_by`, `finished_by`)

**Example Query (A `overlaps` B for employment periods):**
*"Find employees whose employment at 'Acme' overlapped with their employment at 'BetaCorp'."*
```cypher
MATCH (p:Person)-[r_acme:EMPLOYED_AT]->(:Company {name: "Acme"}),
      (p)-[r_beta:EMPLOYED_AT]->(:Company {name: "BetaCorp"})
WHERE r_acme.valid_from < r_beta.valid_from 
  AND r_acme.valid_to > r_beta.valid_from 
  AND r_acme.valid_to < r_beta.valid_to // Assuming valid_to is not null for closed periods
RETURN p.name, r_acme.valid_from AS acme_start, r_acme.valid_to AS acme_end,
       r_beta.valid_from AS beta_start, r_beta.valid_to AS beta_end
```

## 4. Durations
Calculating how long a state persisted.

*   **"How long was Project X in 'Active' status?"**
    *   Find the TimeEdge for `ProjectX --(HAS_STATUS {status: "Active"})--> StatusConcept`.
    *   Duration = `edge.valid_to - edge.valid_from`. (Requires date math functions, often provided by the database or calculated client-side).
    *   If `valid_to` is `None`, duration is relative to "now".
    
    ```cypher
    MATCH (:Project {id: "X"})-[r:HAS_STATUS {status: "Active"}]->()
    // Assuming valid_to is not null for this calculation, or we use 'now'
    RETURN duration.between(r.valid_from, r.valid_to) AS active_duration 
    // 'duration.between' is a Neo4j function example
    ```
    > Note: Handling NULL `valid_to` requires `COALESCE(r.valid_to, 'far_future_date')` or specific `IS NULL` checks.

## 5. Temporal Aggregations
Counting or summarizing facts over time periods.

*   **"How many users signed up each month in 2023?"**
    *   Requires User nodes with a creation TimeEdge or a `CREATED_AT` property that can be mapped to `valid_from`.
    *   Group by month extracted from `valid_from`.
    
    ```cypher
    MATCH (u:User)-[r:WAS_CREATED]->() // Assuming a conceptual creation edge
    WHERE r.valid_from >= date("2023-01-01") AND r.valid_from < date("2024-01-01")
    RETURN r.valid_from.year AS year, r.valid_from.month AS month, count(u) AS signups
    ORDER BY year, month
    ```

## 6. Detecting State Changes (Snapshots & Deltas)
Identifying when a specific property within an edge's props changed, or when an edge of a certain kind appeared/disappeared.

*   **"When did User X's role change?"**
    *   Query all `HAS_ROLE` edges for User X, order by `valid_from`.
    *   Iterate through the sequence, comparing `props.role` of adjacent edges.
*   **"Show all versions of relationship R between Node A and Node B."**
    *   Requires querying by `transaction_time` if you want to see how the database's record of that relationship evolved.
    *   If only `valid_time` is considered, you see the evolution of the fact in the real world.
    
    ```cypher
    MATCH (a:Node {id: "A"})-[r:SOME_KIND]->(b:Node {id: "B"})
    RETURN r.props, r.valid_from, r.valid_to //, r.transaction_start, r.transaction_end
    ORDER BY r.valid_from // or r.transaction_start for DB history
    ```

## 7. Temporal Trends and Patterns
Analyzing how data evolves over longer periods.

*   **"Show the evolution of average sentiment score for Topic Z over the last year, aggregated weekly."**
    *   Requires Sentiment edges with scores and `valid_from` timestamps.
    *   Group by week, calculate average score.
*   **Identifying Co-occurring Temporal Patterns:**
    *   **"Do sales of Product A typically increase after marketing campaigns for Product A become active?"**
        *   This involves finding `CampaignActive` intervals and `SalesIncrease` intervals and checking for temporal relationships like `starts_after_start_of` or `overlaps_with_end_of`.

## 8. Point Events vs. Interval Events/States
*   **Point Events:** Occurrences at a specific moment (e.g., "User Clicked Button").
    *   Model with `valid_from = valid_to = event_timestamp`.
    *   Or, create an Event node: `User --(PERFORMED_EVENT)--> ClickEvent {timestamp: T}`.
*   **Interval Events/States:** Durations (e.g., "User Was Logged In").
    *   Model with `valid_from = start_time`, `valid_to = end_time` (or `None`).

## 9. Challenges and Considerations
*   **Query Complexity:** Advanced temporal queries can become complex. Encapsulate common patterns in helper functions or a DSL within TelaMentis or client libraries.
*   **Performance:**
    *   Ensure `valid_from` and `valid_to` (and transaction times) are well-indexed. Range indexes are essential.
    *   Databases may have specialized temporal extensions or indexing strategies.
*   **Timezone Handling:** Standardize on UTC for all timestamps within TelaMentis. Convert to local timezones only at the presentation/application layer.
*   **Clock Skew:** Be aware of potential clock skew if timestamps originate from distributed sources. Use NTP.
*   **Defining "Now":** `current_timestamp()` can vary if a query runs for a long time. For consistency within a transaction, "now" should be fixed at the start.

## 10. Implications for AI Agent Memory
Advanced temporal reasoning allows agents to:

*   **Build Timelines:** Construct chronological narratives of events and states.
*   **Understand Causality (Tentatively):** Infer potential causal links by observing sequences and temporal overlaps (though true causality requires more than just temporal correlation).
*   **Predict Future States (Rudimentary):** By analyzing past trends and patterns.
*   **Perform Counterfactual Reasoning (Conceptually):** "What if event X had happened earlier?" (Requires more than just queries; involves simulation based on graph state at different times).
*   **Manage Long-Term Memory Coherence:** Resolve conflicting information based on recency or specific validity periods.

By mastering these advanced temporal reasoning patterns, developers can build significantly more sophisticated and context-aware AI agents using TelaMentis.