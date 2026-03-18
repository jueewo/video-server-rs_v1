# Site Generator — Development Notes

## Base Path Prefix (`siteBase`)

Local preview builds serve the site under a subpath (e.g. `/site-builds/{workspace}/{slug}/dist/`).
All asset URLs that are absolute (starting with `/`) must be prefixed with the `siteBase` value,
otherwise they resolve against the server root and 404.

### Where this matters

- **Astro components** (`.astro`): use `import { b } from '../../utils/siteBase'` and prefix with `${b}`.
- **Preact/client components** (`.tsx`): cannot use Astro imports — pass `base` as a prop from the
  parent `.astro` component and prefix URLs in JS.
- **Affected paths**: images, links, fetch URLs to prerendered API routes, favicon, apple-touch-icon,
  fonts, vendor scripts (KaTeX, Mermaid, HLS.js).

### NewsBanner / bannerposts pattern

`NewsBanner` is a Preact client component (`client:visible`). It fetches `/api/const/bannerposts`
at runtime. The `bannerposts.js` API route is prerendered to a static JSON file during `astro build`.

- The fetch URL must be prefixed: `fetch(base + "/api/const/bannerposts")`
- The returned image/link paths must also be prefixed before rendering:
  ```ts
  img: p.img && !p.img.startsWith("http") ? base + p.img : p.img,
  link: p.link && !p.link.startsWith("http") ? base + p.link : p.link,
  ```
- `base` is passed as a prop from `ElementRenderer.astro` using `base={b}`.
- In production `base` is empty (`""`), so paths stay as-is.

### ASTRO_BASE and folder names

`ASTRO_BASE` must not contain spaces — Astro's prerender route resolver breaks with
"Missing parameter" errors when the base path has spaces. The workspace handler sanitises
the folder slug: `clean.replace('/', "_").replace(' ', "-")`.

## Self-Hosted Assets (GDPR)

Generated sites must never contact external servers. All fonts and JS libraries are self-hosted:

- Fonts: `public/fonts/` (woff2 files + `google-fonts.css`)
- KaTeX: `public/vendor/katex/`
- Mermaid: `public/vendor/mermaid/mermaid.min.js` (UMD build, single file)
- HLS.js: `public/vendor/hls/hls.min.js`
- Reveal.js: `public/vendor/reveal/`

Never use CDN links (Google Fonts, cdn.jsdelivr.net, esm.sh, etc.).

## Publish vs Build & Preview

- **Publish Site**: generates merged Astro source (no build) and pushes to Forgejo.
  CI pipeline runs `bun install && bun run build`. Skips `node_modules/`, `dist/`, `.astro/`, `bun.lock`.
- **Build & Preview**: generates, builds locally (`bun run build`), serves `dist/` via `/site-builds/` route.
  No git push.
