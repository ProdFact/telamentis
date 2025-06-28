# 🛡️ TelaMentis Security Hardening Guide

This guide provides recommendations for securing your TelaMentis deployment. Security is a multi-layered concern, encompassing the application, its data, underlying infrastructure, and integrations.

## 1. Threat Model Overview

Consider potential threats such as:
*   **Unauthorized Access**: Gaining access to data or administrative functions without permission.
*   **Data Breach**: Exfiltration of sensitive tenant data.
*   **Denial of Service (DoS)**: Overwhelming the system to make it unavailable.
*   **Data Corruption/Tampering**: Malicious modification or deletion of data.
*   **Injection Attacks**: Exploiting vulnerabilities in query handling or input processing.
*   **Tenant Data Bleed**: One tenant accessing another's data due to flaws in isolation.

## 2. API Security (Presentation Layer)

*   **Authentication**:
    *   Enforce strong authentication for all API endpoints.
    *   **Recommended**: OAuth 2.0 / OpenID Connect with JWT Bearer Tokens.
    *   For service-to-service communication, API Keys with proper entropy and rotation policies can be used.
    *   Implement robust password policies if using direct credential login (less ideal for service APIs).
*   **Authorization**:
    *   **Tenant Scoping**: The `TenantId` extracted from authentication context (e.g., JWT claim) MUST be used to scope all data operations. This is the primary authorization mechanism.
    *   **Role-Based Access Control (RBAC)**: (Future Enhancement) For administrative APIs or fine-grained access within a tenant, consider RBAC. E.g., `tenant_admin` vs. `tenant_user`.
*   **Input Validation**:
    *   Rigorously validate all incoming data (path parameters, query parameters, request bodies).
    *   Use schema validation (e.g., OpenAPI schema for FastAPI).
    *   Sanitize inputs to prevent injection attacks (e.g., if raw query parts are ever constructed from user input, though adapters should use parameterized queries).
*   **Rate Limiting & Quotas**:
    *   Implement per-tenant and per-IP rate limiting to prevent abuse and DoS.
    *   Set reasonable request size limits.
*   **HTTPS Enforcement**:
    *   All API traffic MUST be over HTTPS (TLS 1.2+).
    *   Use tools like Let's Encrypt for certificates.
    *   Configure HSTS (HTTP Strict Transport Security).
*   **API Gateway**: Consider using an API Gateway (e.g., AWS API Gateway, Kong, Nginx) in front of the Presentation Layer for handling auth, rate limiting, TLS termination, and WAF.

## 3. Data Security

*   **Encryption in Transit (EIT)**:
    *   **Client &lt;-&gt; Presentation Layer**: HTTPS (as above).
    *   **Presentation Layer &lt;-&gt; TelaMentis Core**: If on different hosts, use TLS for gRPC/HTTP or secure UDS permissions.
    *   **Core &lt;-&gt; Storage Adapters (Database)**: Ensure database connections use TLS (e.g., Bolt+s:// for Neo4j).
    *   **Core &lt;-&gt; LLM Connectors**: HTTPS for all LLM API calls.
*   **Encryption at Rest (EAR)**:
    *   Encrypt database files (e.g., using LUKS for disk encryption, or native database encryption features like Neo4j Enterprise transparent disk encryption).
    *   Encrypt backups.
*   **Tenant Data Isolation**:
    *   This is paramount. The Storage Adapter's implementation of tenant filtering (`_tenant_id` property or dedicated databases) is critical.
    *   Regularly audit and test tenant isolation mechanisms.
*   **PII (Personally Identifiable Information) Handling**:
    *   Identify PII within your graph data.
    *   **Minimize**: Only store PII that is absolutely necessary.
    *   **Anonymization/Pseudonymization**: Consider techniques if full PII is not required for agent functionality.
    *   **Access Control**: Restrict access to PII even within a tenant if possible.
    *   **LLM Data**: Be cautious about sending PII to LLMs. Use redaction techniques or ensure LLM provider has strong data privacy agreements.
    *   The "Safeguard" plugin (from `lifecycle-and-plugins.md`) could play a role here.
*   **Data Retention & Deletion**:
    *   Implement policies for data retention and secure deletion, respecting GDPR's "Right to be Forgotten."
    *   `kgctl tenant delete` should securely remove tenant data. Bitemporal "logical deletes" (setting `valid_to`) might not be sufficient for full erasure requests; physical deletion capabilities are needed.

## 4. Plugin Security

*   **Code Review**: Thoroughly review any third-party or community-contributed plugins before deployment.
*   **Secure Configuration**:
    *   Plugin configurations (API keys, credentials) should be managed as secrets (see Secrets Management).
    *   Avoid hardcoding secrets in plugin code.
*   **Permissions (Future - Dynamic Loading)**: If dynamic plugin loading (e.g., from `.so` files) is implemented, a sandboxing or permission model would be essential to limit plugin capabilities. (Rust trait-based plugins compiled in are less risky here but still need scrutiny).
*   **Input to Plugins**: Inputs to request pipeline plugins should be treated as untrusted if originating externally.

## 5. Infrastructure Security

*   **Secure Host Configuration**:
    *   Harden underlying operating systems.
    *   Regularly apply security patches to OS and all software components.
*   **Docker Security**:
    *   Use minimal base images (e.g., Alpine Linux).
    *   Run containers as non-root users.
    *   Scan images for vulnerabilities.
    *   Configure resource limits for containers.
*   **Network Security**:
    *   Use firewalls to restrict network access to necessary ports.
    *   In Kubernetes, use NetworkPolicies for pod-to-pod communication control.
    *   Segment networks (e.g., separate database network from application network).
*   **Dependency Management**:
    *   Regularly scan Rust dependencies for vulnerabilities (`cargo audit`).
    *   Update dependencies promptly.

## 6. Secrets Management

*   **NEVER** hardcode secrets (API keys, database passwords, encryption keys) in code or configuration files committed to version control.
*   **Use**:
    *   Environment variables (loaded securely into the application).
    *   Dedicated secrets management tools (e.g., HashiCorp Vault, AWS Secrets Manager, Azure Key Vault, Kubernetes Secrets).
*   Ensure secrets have minimal necessary permissions and are rotated regularly.

## 7. LLM Security

*   **Prompt Injection**:
    *   This is a challenging problem. Mitigations include:
        *   Strong system prompts that define expected input/output and forbid instruction overriding.
        *   Input sanitization/validation before sending to LLM.
        *   Output parsing and validation (e.g., ensuring LLM output for `ExtractionEnvelope` matches the JSON schema).
        *   Treat LLM output as untrusted until validated.
*   **Data Privacy with LLMs**: As mentioned in PII, be extremely careful about data sent to external LLMs. Prefer providers with strong privacy commitments or use on-premise/private LLMs for sensitive data.
*   **API Key Security**: Protect LLM API keys rigorously using secrets management.
*   **Monitoring LLM Usage**: Track token counts and costs to detect anomalies or potential abuse.

## 8. Auditing and Logging for Security

*   Enable detailed audit logs for security-sensitive events:
    *   Authentication successes and failures.
    *   Tenant creation/deletion.
    *   Administrative actions via `kgctl`.
    *   Significant data modification/deletion operations.
    *   Changes to security configurations.
*   Ensure logs are stored securely and retained appropriately.
*   Regularly review logs for suspicious activity.

## 9. Incident Response

*   Have an incident response plan in place detailing steps to take in case of a security breach.
    *   Containment, eradication, recovery, lessons learned.
    *   Communication plan (internal and external).

## 10. Regular Security Assessments

*   Conduct periodic security assessments, including:
    *   Vulnerability scanning.
    *   Penetration testing (especially for API endpoints and tenant isolation).
    *   Code reviews focused on security.

Security is an ongoing process, not a one-time task. Continuously review and update your security posture as TelaMentis evolves and new threats emerge.