import { l as createVNode, i as Fragment, _ as __astro_tag_component__ } from './astro/server_DLHsrwWI.mjs';
import 'clsx';

const frontmatter = {
  "title": "RUSVEL",
  "description": "The Solo Builder's AI-Powered Virtual Agency",
  "template": "splash",
  "hero": {
    "tagline": "One binary, one human, infinite leverage. Rust + SvelteKit monorepo.",
    "actions": [{
      "text": "Get Started",
      "link": "/getting-started/installation/",
      "icon": "right-arrow"
    }, {
      "text": "View on GitHub",
      "link": "https://github.com/mbaneshi/all-in-one-rusvel",
      "icon": "external",
      "variant": "minimal"
    }]
  }
};
function getHeadings() {
  return [];
}
function _createMdxContent(props) {
  return createVNode(Fragment, {});
}
function MDXContent(props = {}) {
  const {wrapper: MDXLayout} = props.components || ({});
  return MDXLayout ? createVNode(MDXLayout, {
    ...props,
    children: createVNode(_createMdxContent, {
      ...props
    })
  }) : _createMdxContent();
}

const url = "src/content/docs/index.mdx";
const file = "/Users/bm/all-in-one-rusvel/docs-site/src/content/docs/index.mdx";
const Content = (props = {}) => MDXContent({
  ...props,
  components: { Fragment: Fragment, ...props.components, },
});
Content[Symbol.for('mdx-component')] = true;
Content[Symbol.for('astro.needsHeadRendering')] = !Boolean(frontmatter.layout);
Content.moduleId = "/Users/bm/all-in-one-rusvel/docs-site/src/content/docs/index.mdx";
__astro_tag_component__(Content, 'astro:jsx');

export { Content, Content as default, file, frontmatter, getHeadings, url };
