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
        {
          label: 'Getting Started',
          items: [
            { label: 'Installation', slug: 'getting-started/installation' },
            { label: 'First Run', slug: 'getting-started/first-run' },
            { label: 'First Mission', slug: 'getting-started/first-mission' },
          ],
        },
        {
          label: 'Concepts',
          items: [
            { label: 'Sessions', slug: 'concepts/sessions' },
            { label: 'Departments', slug: 'concepts/departments' },
            { label: 'Agents', slug: 'concepts/agents' },
            { label: 'Skills & Rules', slug: 'concepts/skills-rules' },
            { label: 'Workflows', slug: 'concepts/workflows' },
          ],
        },
        {
          label: 'Departments',
          items: [
            { label: 'Forge (Mission)', slug: 'departments/forge' },
            { label: 'Code', slug: 'departments/code' },
            { label: 'Content', slug: 'departments/content' },
            { label: 'Harvest', slug: 'departments/harvest' },
            { label: 'GTM', slug: 'departments/gtm' },
            { label: 'Finance', slug: 'departments/finance' },
            { label: 'Product', slug: 'departments/product' },
            { label: 'Growth', slug: 'departments/growth' },
            { label: 'Distro', slug: 'departments/distro' },
            { label: 'Legal', slug: 'departments/legal' },
            { label: 'Support', slug: 'departments/support' },
            { label: 'Infra', slug: 'departments/infra' },
          ],
        },
        {
          label: 'Reference',
          items: [
            { label: 'CLI', slug: 'reference/cli' },
            { label: 'API', slug: 'reference/api' },
            { label: 'Configuration', slug: 'reference/configuration' },
          ],
        },
        {
          label: 'Architecture',
          items: [
            { label: 'Overview', slug: 'architecture/overview' },
            { label: 'Decisions (ADRs)', slug: 'architecture/decisions' },
          ],
        },
      ],
    }),
  ],
});
