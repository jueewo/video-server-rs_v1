import { defineConfig } from "astro/config";
import vue from "@astrojs/vue";
import tailwindcss from "@tailwindcss/vite";
import mdx from "@astrojs/mdx";
import sitemap from "@astrojs/sitemap";
import preact from "@astrojs/preact";
import remarkMath from "remark-math";
import rehypeKatex from "rehype-katex";
import { remarkMermaidBlocks } from "./src/plugins/rehype-mermaid-blocks.mjs";
import siteConfig from "./src/website.redirects.mjs";

const siteUrl = process.env.SITE_URL || "http://localhost:4321";
const astroBase = process.env.ASTRO_BASE;

const base = astroBase ? astroBase.replace(/\/$/, "") : "";
const redirects = base
  ? Object.fromEntries(
      Object.entries(siteConfig.redirects).map(([from, to]) => [from, base + to])
    )
  : siteConfig.redirects;

export default defineConfig({
  redirects,
  outDir: "./dist",
  publicDir: "./public",
  site: siteUrl,
  compressHTML: true,
  ...(astroBase ? { base: astroBase } : {}),

  server: {
    host: "localhost",
    port: 4321,
  },

  image: {
    service: {
      entrypoint: "astro/assets/services/sharp",
    },
  },

  integrations: [
    preact({
      include: ["**/preact/*", "**/icons/*"],
      compat: true,
    }),
    vue(),
    mdx({
      extendMarkdownConfig: false,
      remarkPlugins: [remarkMath, remarkMermaidBlocks],
      rehypePlugins: [
        [rehypeKatex, {}],
      ],
    }),
    sitemap({
      i18n: {
        defaultLocale: siteConfig.defaultLocale,
        locales: siteConfig.locales,
      },
    }),
  ],

  markdown: {
    remarkPlugins: [remarkMath, remarkMermaidBlocks],
    rehypePlugins: [
      [rehypeKatex, {}],
    ],
  },

  vite: {
    plugins: [tailwindcss()],
    resolve: {
      alias: {
        "~/": new URL("./src/", import.meta.url).pathname,
      },
    },
  },
});
