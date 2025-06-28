import React from 'react';
import ComponentCreator from '@docusaurus/ComponentCreator';

export default [
  {
    path: '/TelaMentis/__docusaurus/debug',
    component: ComponentCreator('/TelaMentis/__docusaurus/debug', '0e2'),
    exact: true
  },
  {
    path: '/TelaMentis/__docusaurus/debug/config',
    component: ComponentCreator('/TelaMentis/__docusaurus/debug/config', '904'),
    exact: true
  },
  {
    path: '/TelaMentis/__docusaurus/debug/content',
    component: ComponentCreator('/TelaMentis/__docusaurus/debug/content', 'e59'),
    exact: true
  },
  {
    path: '/TelaMentis/__docusaurus/debug/globalData',
    component: ComponentCreator('/TelaMentis/__docusaurus/debug/globalData', '14f'),
    exact: true
  },
  {
    path: '/TelaMentis/__docusaurus/debug/metadata',
    component: ComponentCreator('/TelaMentis/__docusaurus/debug/metadata', '8a4'),
    exact: true
  },
  {
    path: '/TelaMentis/__docusaurus/debug/registry',
    component: ComponentCreator('/TelaMentis/__docusaurus/debug/registry', 'd5b'),
    exact: true
  },
  {
    path: '/TelaMentis/__docusaurus/debug/routes',
    component: ComponentCreator('/TelaMentis/__docusaurus/debug/routes', 'bb6'),
    exact: true
  },
  {
    path: '/TelaMentis/markdown-page',
    component: ComponentCreator('/TelaMentis/markdown-page', '408'),
    exact: true
  },
  {
    path: '/TelaMentis/search',
    component: ComponentCreator('/TelaMentis/search', 'e80'),
    exact: true
  },
  {
    path: '/TelaMentis/docs',
    component: ComponentCreator('/TelaMentis/docs', '957'),
    routes: [
      {
        path: '/TelaMentis/docs',
        component: ComponentCreator('/TelaMentis/docs', '545'),
        routes: [
          {
            path: '/TelaMentis/docs',
            component: ComponentCreator('/TelaMentis/docs', '1c8'),
            routes: [
              {
                path: '/TelaMentis/docs/core_concepts',
                component: ComponentCreator('/TelaMentis/docs/core_concepts', 'c1f'),
                exact: true
              },
              {
                path: '/TelaMentis/docs/security_hardening_guide',
                component: ComponentCreator('/TelaMentis/docs/security_hardening_guide', '80a'),
                exact: true
              }
            ]
          }
        ]
      }
    ]
  },
  {
    path: '/TelaMentis/',
    component: ComponentCreator('/TelaMentis/', '66b'),
    exact: true
  },
  {
    path: '*',
    component: ComponentCreator('*'),
  },
];
