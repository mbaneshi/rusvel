import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

export default defineConfig({
  integrations: [
    starlight({
      title: 'RUSVEL',
      description: 'The Solo Builder\'s AI-Powered Virtual Agency',
      social: {
        github: 'https://github.com/mbaneshi/all-in-one-rusvel',
      },
      sidebar: [
        { label: 'Getting Started', autogenerate: { directory: 'getting-started' } },
        { label: 'Concepts', autogenerate: { directory: 'concepts' } },
        { label: 'Departments', autogenerate: { directory: 'departments' } },
        { label: 'Reference', autogenerate: { directory: 'reference' } },
        { label: 'Architecture', autogenerate: { directory: 'architecture' } },
      ],
    }),
  ],
});
