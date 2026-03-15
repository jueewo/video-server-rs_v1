import { defineConfig } from "astro/config";
import vue from "@astrojs/vue";
import tailwindcss from "@tailwindcss/vite";
import mdx from "@astrojs/mdx";
import sitemap from "@astrojs/sitemap";
import preact from "@astrojs/preact";
import remarkMath from "remark-math";
import rehypeKatex from "rehype-katex";

// Site URL: set SITE_URL env var in CI for correct sitemap generation.
const siteUrl = process.env.SITE_URL || "http://localhost:4321";

export default defineConfig({
  // srcDir defaults to "./src" — generator writes pages/data/content into src/
  outDir: "./dist",
  publicDir: "./public",
  site: siteUrl,

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
    }),
    sitemap({
      i18n: {
        defaultLocale: "en",
        locales: {
          en: "en",
          de: "de",
        },
      },
    }),
  ],

  markdown: {
    remarkPlugins: [remarkMath],
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
