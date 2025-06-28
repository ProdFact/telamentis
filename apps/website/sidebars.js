/**
 * Creating a sidebar enables you to:
 - create an ordered group of docs
 - render a sidebar for each doc of that group
 - provide next/previous navigation

 The sidebars can be generated from the filesystem, or explicitly defined here.

 Create as many sidebars as you want.
 */

/** @type {import('@docusaurus/plugin-content-docs').SidebarsConfig} */
const sidebars = {
  tutorialSidebar: [
    {
      type: 'category',
      label: 'Getting Started',
      items: ['getting_started'],
    },
    {
      type: 'category',
      label: 'Core Concepts',
      items: ['core_concepts', 'schema_design_guide', 'temporal_semantics'],
    },
    {
      type: 'category',
      label: 'Architecture & Design',
      items: ['architecture', 'multi_tenancy', 'advanced_temporal_reasoning', 'request_processing_pipeline', 'lifecycle-and-plugins', 'middleware'],
    },
    {
      type: 'category',
      label: 'LLM & Agent Integration',
      items: ['llm_extraction', 'agent_integration_patterns', 'recall-plugin'],
    },
    {
      type: 'category',
      label: 'Operations',
      items: ['security_hardening_guide', 'observability_guide'],
    },
    {
      type: 'category',
      label: 'Development',
      items: ['plugin_development'],
    },
  ],
};

module.exports = sidebars;