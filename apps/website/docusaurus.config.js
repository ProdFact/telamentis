// @ts-check
// `@type` JSDoc annotations allow IDE autocompletion and type checking

const {themes} = require('prism-react-renderer');
const lightCodeTheme = themes.github;
const darkCodeTheme = themes.dracula;

// @ts-ignore
/** @type {import('@docusaurus/types').Config} */
const config = {
  title: 'Tela Mentis - AI Memory System',
  tagline: 'Real-time, temporally-aware, multi-tenant knowledge graphs for AI agents',
  favicon: 'img/favicon.ico',

  // Set the production url of your site here
  url: 'https://prodfact.github.io',
  // Set the /<baseUrl>/ pathname under which your site is served
  // For GitHub pages deployment, it is often '/<projectName>/'
  baseUrl: process.env.NODE_ENV === 'production' ? '/TelaMentis/' : '/',

  // GitHub pages deployment config
  organizationName: 'ProdFact',
  projectName: 'TelaMentis',
  trailingSlash: false,

  onBrokenLinks: 'throw',
  onBrokenMarkdownLinks: 'warn',
  
  // Even if you don't use internationalization, you can use this field to set
  // useful metadata like html lang
  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },

  presets: [
    [
      'classic', 
      // @ts-ignore
      /** @type {import('@docusaurus/preset-classic').Options} */
      ({
        docs: {
          sidebarPath: require.resolve('./sidebars.js'),
          path: '../../docs',
          routeBasePath: 'docs',
          editUrl: 'https://github.com/ProdFact/TelaMentis/edit/main/docs/',
          showLastUpdateTime: true,
          showLastUpdateAuthor: true,
        },
        blog: false,
        theme: {
          customCss: require.resolve('./src/css/custom.css'),
        },
      }),
    ], 
  ],

  themes: ['@docusaurus/theme-mermaid'],
  
  markdown: {
    mermaid: true,
  },
  
  themeConfig:
    // @ts-ignore
    /** @type {import('@docusaurus/preset-classic').ThemeConfig} */
    ({
      image: 'img/social-card.svg',
      navbar: {
        title: 'Tela Mentis',
        logo: { 
          alt: 'Tela Mentis Logo',
          src: 'img/logo.svg',
        },
        items: [
          {
            type: 'docSidebar', 
            sidebarId: 'tutorialSidebar', 
            position: 'left', 
            label: 'Docs', 
          },
          {
            to: '/docs/architecture',
            label: 'Architecture',
            position: 'left',
          },
          {
            to: '/docs/llm_extraction',
            label: 'LLM Integration',
            position: 'left',
          },
          {
            href: 'https://github.com/ProdFact/TelaMentis',
            label: 'GitHub',
            position: 'right',
          },
        ],
      },
      footer: {
        style: 'dark',
        links: [
          {
            title: 'Documentation',
            items: [
              {
                label: 'Installation',
                to: '/docs/getting_started#installation',
              },
              {
                label: 'Getting Started',
                to: '/docs/getting_started',
              },
              {
                label: 'Core Concepts',
                to: '/docs/core_concepts',
              },
              {
                label: 'Temporal Semantics',
                to: '/docs/temporal_semantics',
              },
            ],
          },
          {
            title: 'Community',
            items: [
              {
                label: 'GitHub Discussions',
                href: 'https://github.com/ProdFact/TelaMentis/discussions',
              },
              {
                label: 'Issue Tracker',
                href: 'https://github.com/ProdFact/TelaMentis/issues',
              },
            ],
          },
          {
            title: 'More',
            items: [
              {
                label: 'Project Roadmap',
                href: 'https://github.com/ProdFact/TelaMentis/blob/main/ROADMAP.md',
              },
              {
                label: 'GitHub',
                href: 'https://github.com/ProdFact/TelaMentis',
              },
            ],
          },
        ],
        copyright: `Copyright © ${new Date().getFullYear()} Prodfact OU. Built with Docusaurus.`,
      },
      prism: {
        theme: lightCodeTheme,
        darkTheme: darkCodeTheme,
        additionalLanguages: ['rust', 'bash', 'toml'],
        magicComments: [
          {
            className: 'theme-code-block-highlighted-line',
            line: 'highlight-next-line',
            block: {start: 'highlight-start', end: 'highlight-end'},
          },
        ],
      },
      colorMode: {
        defaultMode: 'light',
        disableSwitch: false,
        respectPrefersColorScheme: true,
      },
      announcementBar: {
        id: 'beta_announcement',
        content:
          '🚀 Tela Mentis is currently in Phase 2 (Beta) | <a href="https://github.com/ProdFact/TelaMentis">Star us on GitHub</a>',
        backgroundColor: '#6366F1',
        textColor: '#fff',
        isCloseable: true,
      },
      metadata: [
        {name: 'keywords', content: 'knowledge graph, AI memory, temporal, rust, graph database, LLM, agent, multi-tenant'},
        {name: 'description', content: 'Tela Mentis: Real-time, temporally-aware, multi-tenant knowledge graphs for AI agents – Rust core, pluggable everything.'},
      ],
    }),
};

module.exports = config;