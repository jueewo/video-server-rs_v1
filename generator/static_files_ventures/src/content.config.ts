// Astro 6 content layer — auto-discovers collections from ./src/data/ and ./src/content/
import { defineCollection, z } from "astro:content";
import { glob } from "astro/loaders";
import { readdirSync } from "fs";

// ── Schemas ───────────────────────────────────────────────────────────────────

const assetCardSchema = ({ image }: { image: () => any }) => z.object({
  title: z.string(),
  desc: z.string(),
  description: z.string().optional(),
  keywords: z.string().optional(),
  author: z.string().optional(),
  url: z.string().optional(),
  ext: z.boolean().default(false),
  linktxt: z.string().optional(),
  nolinktxt: z.string().optional(),
  typetags: z.array(z.string()),
  tags: z.array(z.string()),
  filtertags: z.array(z.string()).optional(),
  badge: z.string().optional(),
  content_msg: z.string().optional(),
  pubDate: z.coerce.date().optional(),
  updatedDate: z.coerce.date().optional(),
  image: image().optional(),
  showtoc: z.boolean().optional(),
  enablelatex: z.boolean().optional(),
  enablemermaid: z.boolean().optional(),
  featured: z.boolean(),
  draft: z.boolean(),
  draft_content: z.boolean().default(false),
  draft_content_msg: z.string().optional(),
  draft_content_msg_long: z.string().optional(),
  draft_content_clickable: z.boolean().default(true),
  // Page element renderer support
  faqdata: z.array(z.object({ quest: z.string(), ans: z.string() })).optional(),
  elements_above: z.array(z.any()).optional(),
  elements_below: z.array(z.any()).optional(),
});

const mdContentSchema = z.object({
  title: z.string(),
  pubDate: z.coerce.date().optional(),
  updatedDate: z.coerce.date().optional(),
  showtoc: z.boolean().optional(),
  draft: z.boolean().optional(),
});

const pageElementSchema = z.object({
  draft: z.boolean().optional().default(false),
  weight: z.number().default(0),
  element: z.string(),
  slot: z.string().nullable().optional(),
  wrapper: z.string().nullable().optional(),
  content: z.record(z.any()).optional(),
  props: z.record(z.any()).optional(),
});

// ── Auto-discovery helpers ─────────────────────────────────────────────────────

function dirs(path: string): string[] {
  try {
    return readdirSync(path, { withFileTypes: true })
      .filter((d) => d.isDirectory())
      .map((d) => d.name);
  } catch {
    return [];
  }
}

// Page element collections: ./src/data/page_*  (JSON files, no strict schema)
function discoverPageCollections() {
  const collections: Record<string, ReturnType<typeof defineCollection>> = {};
  for (const name of dirs("./src/data")) {
    if (name.startsWith("page_")) {
      collections[name] = defineCollection({
        loader: glob({ pattern: "**/page.json", base: `./src/data/${name}` }),
      });
    }
  }
  return collections;
}

// Content collections: ./src/content/*  (MDX files)
function discoverContentCollections() {
  const collections: Record<string, ReturnType<typeof defineCollection>> = {};
  for (const name of dirs("./src/content")) {
    const schema = name === "mdcontent" ? mdContentSchema : assetCardSchema;
    collections[name] = defineCollection({
      loader: glob({ pattern: "**/*.{md,mdx}", base: `./src/content/${name}` }),
      schema,
    });
  }
  return collections;
}

// ── Export ─────────────────────────────────────────────────────────────────────

export const collections = {
  ...discoverPageCollections(),
  ...discoverContentCollections(),
};
