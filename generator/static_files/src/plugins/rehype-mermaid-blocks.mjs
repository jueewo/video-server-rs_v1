import { visit } from "unist-util-visit";

/**
 * Remark plugin: transforms ```mermaid code blocks into raw HTML
 * <pre class="mermaid"> BEFORE Shiki syntax highlighting runs.
 *
 * Must be used as a remarkPlugin, not a rehypePlugin.
 *
 * Input remark AST:  { type: 'code', lang: 'mermaid', value: '...' }
 * Output remark AST: { type: 'html', value: '<pre class="mermaid">...</pre>' }
 */
export function remarkMermaidBlocks() {
  return (tree) => {
    visit(tree, "code", (node, index, parent) => {
      if (node.lang === "mermaid") {
        parent.children.splice(index, 1, {
          type: "html",
          value: `<pre class="mermaid">${node.value}</pre>`,
        });
      }
    });
  };
}
