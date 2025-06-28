import React from 'react';
import clsx from 'clsx';
import { TypeAnimation } from 'react-type-animation';
import Link from '@docusaurus/Link';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import useBaseUrl from '@docusaurus/useBaseUrl';
import Layout from '@theme/Layout';
import HomepageFeatures from '../components/HomepageFeatures';
import Mermaid from '@theme/Mermaid';

import styles from './index.module.css';

const FeatureSection = () => {
  return (
    <section className={styles.valueProposition}>
      <div className="container">
        <div className={styles.valueHeading}>
          <h2>What problem are we solving?</h2>
          <p>
            AI agents struggle with statelessness - they forget important context between interactions. 
            Tela Mentis provides durable, searchable memory that evolves over time, enabling:
          </p>
        </div>
        <div className="row">
          <div className="col col--4">
            <div className={styles.featureCard}>
              <h3>Persistent Memory</h3>
              <p>
                Store conversations, facts, and relationships in a structured knowledge graph
                that persists between sessions and across multiple users.
              </p>
            </div>
          </div>
          <div className="col col--4">
            <div className={styles.featureCard}>
              <h3>Temporal Awareness</h3>
              <p>
                Track both when facts were true and when they were learned, 
                enabling time-aware reasoning and context retrieval.
              </p>
            </div>
          </div>
          <div className="col col--4">
            <div className={styles.featureCard}>
              <h3>Multi-Agent Collaboration</h3>
              <p>
                Enable multiple AI agents to share a common knowledge base while
                maintaining tenant isolation for security and privacy.
              </p>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
};

const DemoSection = () => {
  return (
    <section className={styles.demoSection}>
      <div className="container">
        <div className="row">
          <div className="col col--6">
            <h2>Knowledge Extraction with LLMs</h2>
            <p>
              Tela Mentis seamlessly integrates with LLMs to extract structured knowledge from unstructured text.
              This enables AI agents to build and maintain their knowledge graphs automatically.
            </p>
            <div className={styles.bulletPoints}>
              <div className={styles.bulletPoint}>
                <span className={styles.bulletIcon}>✓</span>
                <span>Unified API across OpenAI, Anthropic, and Google models</span>
              </div>
              <div className={styles.bulletPoint}>
                <span className={styles.bulletIcon}>✓</span>
                <span>Extract entities and relationships with temporal context</span>
              </div>
              <div className={styles.bulletPoint}>
                <span className={styles.bulletIcon}>✓</span>
                <span>Automatic schema alignment and deduplication</span>
              </div>
            </div>
            <div className={styles.codeContainer}>
              <pre className={styles.code}>
                <code>
                  {`// Extract knowledge from conversation
const context = {
  messages: [{ 
    role: "user", 
    content: "Alice started working at Acme Corp in January 2023" 
  }]
};

// Knowledge graph is automatically updated
const result = await telaMentis.extract(tenant, context);`}
                </code>
              </pre>
            </div>
          </div>
          <div className="col col--6">
            <div className={styles.demoGraphic}>
              <img 
                src={useBaseUrl('/img/knowledge-extraction.svg')} 
                alt="Knowledge Extraction Demo" 
                className={styles.demoImage}
              />
            </div>
          </div>
        </div>
      </div>
    </section>
  );
};

const ArchitectureSection = () => {
  return (
    <section className={styles.architectureSection}>
      <div className="container">
        <h2>Pluggable Architecture</h2>
        <p className={styles.sectionDescription}>
          Tela Mentis is built with a pluggable architecture that allows you to customize any component
          while maintaining a consistent core API. This makes it adaptable to any AI application stack.
        </p>
        <div className={styles.architectureDiagram}>
          <img 
            src={useBaseUrl('/img/architecture-diagram.svg')} 
            alt="Tela Mentis Architecture Diagram" 
            className={styles.architectureImage}
          />
        </div>
        <div className={styles.mermaidDiagram}>
          <Mermaid
            value={`graph TD
    A[AI Agents] --> B[Presentation Layer]
    B --> C[TelaMentis Core]
    C --> D[Storage Adapters]
    C --> E[LLM Connectors]
    
    subgraph "Storage Options"
        D --> D1[Neo4j]
        D --> D2[In-Memory]
        D --> D3[Your Custom Adapter]
    end
    
    subgraph "LLM Providers"
        E --> E1[OpenAI]
        E --> E2[Anthropic]
        E --> E3[Google Gemini]
    end`}
          />
        </div>
      </div>
    </section>
  );
};

function HomepageHeader() {
  const {siteConfig} = useDocusaurusContext();
  return (
    <header className={clsx('hero', styles.heroBanner)}>
      <div className="container">
        <div className="row">
          <div className="col col--6">
            <h1 className="hero__title">
              <span className={styles.titleEmphasis}>Tela Mentis</span>
            </h1>
            <div className={styles.typingContainer}>
              <TypeAnimation
                className="hero__subtitle"
                sequence={[
                  'AI agents need memory.',
                  1500,
                  'AI agents need temporal awareness.',
                  1500,
                  'AI agents need structured knowledge.',
                  1500,
                  'AI agents need Tela Mentis.',
                  3000,
                ]}
                speed={50}
                repeat={Infinity}
                style={{ fontSize: '1.5rem' }}
              />
            </div>
            <p className={styles.heroDescription}>
              Real-time, temporally-aware, multi-tenant knowledge graphs for AI agents
              <br />Built with a Rust core and pluggable architecture
            </p>
            <div className={styles.buttons}>
              <Link
                className="button button--primary button--lg"
                to="/docs/getting_started">
                Get Started
              </Link>
              <Link
                className="button button--secondary button--outline button--lg"
                to="https://github.com/ProdFact/TelaMentis">
                GitHub
              </Link>
              <Link
                className="button button--secondary button--outline button--lg"
                to="docs/temporal_semantics">
                Learn About Bitemporal Features
              </Link>
            </div>
          </div>
          <div className="col col--6">
            <div className={styles.heroIllustration}>
              <img 
                src={useBaseUrl('/img/hero-illustration.svg')} 
                alt="Temporal Knowledge Graph Illustration" 
                className={styles.heroImage}
              />
            </div>
          </div>
        </div>
      </div>
    </header>
  );
}

export default function Home(): JSX.Element {
  const {siteConfig} = useDocusaurusContext();
  return (
    <Layout
      title={siteConfig.title}
      description="Real-time, temporally-aware, multi-tenant knowledge graphs for AI agents">
      <HomepageHeader />
      <main>
        <FeatureSection />
        <DemoSection />
        <ArchitectureSection />
        <HomepageFeatures />
      </main>
    </Layout>
  );
}