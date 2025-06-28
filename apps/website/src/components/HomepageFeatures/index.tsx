import React from 'react';
import clsx from 'clsx';
import styles from './styles.module.css';

type FeatureItem = {
  title: string;
  icon: React.ReactNode;
  description: JSX.Element;
};

const FeatureList: FeatureItem[] = [
  {
    title: 'Millisecond-Scale Performance',
    icon: <div className={styles.iconWrap}><span role="img" aria-label="brain">🧠</span></div>,
    description: (
      <>
        <p className={styles.featureDescription}>
          Powered by a high-performance Rust core for <strong>millisecond-latency</strong> graph operations.
        </p>
        <div className={styles.featureDetail}>
          Ultra-fast response times essential for natural AI conversations and real-time decision making.
        </div>
      </>
    ),
  },
  {
    title: 'Adapt to Any Stack',
    icon: <div className={styles.iconWrap}><span role="img" aria-label="plug">🔌</span></div>,
    description: (
      <>
        <p className={styles.featureDescription}>
          <strong>Pluggable architecture</strong> with adapters for storage, transport, and LLMs.
        </p>
        <div className={styles.featureDetail}>
          Works with Neo4j, In-Memory, HTTP, gRPC, UDS, OpenAI, Anthropic, and Google Gemini.
        </div>
      </>
    ),
  },
  {
    title: 'Time-Aware Reasoning',
    icon: <div className={styles.iconWrap}><span role="img" aria-label="hourglass">⏳</span></div>,
    description: (
      <>
        <p className={styles.featureDescription}>
          <strong>Bitemporal knowledge model</strong> tracks both when facts were true and when they were recorded.
        </p>
        <div className={styles.featureDetail}>
          Enables "as-of" queries, temporal reasoning, and change tracking for advanced agent memory.
        </div>
      </>
    ),
  },
  {
    title: 'Secure Multi-Tenancy',
    icon: <div className={styles.iconWrap}><span role="img" aria-label="building">🏢</span></div>,
    description: (
      <>
        <p className={styles.featureDescription}>
          <strong>Enterprise-grade isolation</strong> between tenants in a single deployment.
        </p>
        <div className={styles.featureDetail}>
          Perfect for SaaS applications, serving multiple customers, or isolating different agent ecosystems.
        </div>
      </>
    ),
  },
  {
    title: 'Developer-Friendly Tools',
    icon: <div className={styles.iconWrap}><span role="img" aria-label="tools">🛠️</span></div>,
    description: (
      <>
        <p className={styles.featureDescription}>
          <strong>Comprehensive CLI tool</strong> (kgctl) for all operations and management tasks.
        </p>
        <div className={styles.featureDetail}>
          Easily ingest data from CSV, manage tenants, run queries, and export/import graph data.
        </div>
      </>
    ),
  },
  {
    title: 'AI-Native Integration',
    icon: <div className={styles.iconWrap}><span role="img" aria-label="robot">🤖</span></div>,
    description: (
      <>
        <p className={styles.featureDescription}>
          <strong>Built for AI agents</strong> with first-class LLM extraction pipeline.
        </p>
        <div className={styles.featureDetail}>
          Transform unstructured text into structured graph knowledge through seamless LLM integration.
        </div>
      </>
    ),
  },
];

function Feature({title, icon, description}: FeatureItem) {
  return (
    <div className={clsx('col col--4')}>
      <div className={styles.featureItem}>
        <div className={styles.featureIcon}>{icon}</div>
        <h3 className={styles.featureTitle}>{title}</h3>
        <div className="padding-horiz--md">
          {description}
        </div>
      </div>
    </div>
  );
}

export default function HomepageFeatures(): JSX.Element {
  return (
    <section className={styles.features}>
      <div className="container padding-vert--lg">
        <h2 className={styles.sectionTitle}>Key Capabilities</h2>
        <div className="row">
          {FeatureList.map((props, idx) => (
            <Feature key={idx} {...props} />
          ))}
        </div>
      </div>
    </section>
  );
}