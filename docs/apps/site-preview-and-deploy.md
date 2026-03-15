# Site Preview & Subdirectory Deployment

## How Build & Preview Works

When you click **Build & Preview** in the site overview, the platform:

1. Generates the Astro project into `storage/site-builds/{workspace_id}/{folder_slug}/`
2. Runs `bun install && bun run build` with `ASTRO_BASE=/storage/site-builds/{workspace_id}/{folder_slug}/dist`
3. Serves the built `dist/` as static files under that same path
4. Redirects the browser directly to the home page (e.g. `.../dist/en/home/`) to skip Astro's root redirect

The `ASTRO_BASE` env var is picked up by `astro.config.mjs`:

```js
const astroBase = process.env.ASTRO_BASE;
export default defineConfig({
  ...(astroBase ? { base: astroBase } : {}),
  // ...
});
```

When `base` is set, Astro sets `import.meta.env.BASE_URL = "/storage/.../dist/"` (always with trailing slash). Vite uses this automatically for asset URLs (`_astro/` bundles, images processed by `<Image />`).

## Why Internal Links Also Need BASE_URL

Asset URLs go through Vite and are automatically prefixed. But `href` attributes that are plain string expressions (e.g. `"/" + lang + "/home"`) are **not** rewritten by Vite — they stay root-relative.

To fix this, all internal page link constructions in the Astro templates use a shared helper:

```ts
// src/utils/siteBase.ts
export const b: string = import.meta.env.BASE_URL; // always has trailing slash
```

Usage pattern:
```astro
---
import { b } from '~/utils/siteBase';
---
<!-- Instead of href={"/" + lang + "/home"} -->
<a href={b + lang + "/home"}>...</a>

<!-- Instead of `/${lang}/${collection}/${slug}` -->
<a href={`${b}${lang}/${collection}/${slug}`}>...</a>
```

**In production** (`base` not set): `b = "/"` → links are `/en/home` (unchanged)
**In preview**: `b = "/storage/.../dist/"` → links are `/storage/.../dist/en/home`
**On a subdomain path**: `b = "/site1/"` → links are `/site1/en/home`

Files using this pattern:
- `src/components/CardDefault2.astro`, `CardBlog.astro`, `CardInfo.astro` — collection card links
- `src/components/MyDrawerMenu.astro` — navigation menu and language switcher
- `src/layouts/main.astro` — logo/home link in navbar
- `src/layouts/contentlayout.astro` — navbar, breadcrumbs, TOC links
- `src/components/page-elements/Hero.astro`, `Hero2.astro` — internal button links
- `src/pages/404.astro` — back-to-home link

## Deploying to a Subdirectory Path

To deploy the site at `https://domain.com/site1/`:

1. Build with `ASTRO_BASE=/site1`
2. Serve `dist/` from `/site1/` on your web server

**nginx example:**
```nginx
location /site1/ {
    alias /path/to/dist/;
    try_files $uri $uri/ $uri.html =404;
}
```

**Caddy example:**
```caddy
handle /site1/* {
    root * /path/to/dist
    file_server
}
```

Set `SITE_URL=https://domain.com` at build time for correct sitemap generation (the sitemap uses the full site URL, not the base path).

## Redirect Destination Prefixing

Astro's `redirects` config (generated in `src/website.redirects.mjs`) creates HTML redirect pages like:

```
dist/home/index.html  →  redirects to /en/home
dist/index.html       →  redirects to /en/home
```

Astro prefixes redirect **sources** with `base` automatically, but does not prefix redirect **destinations**. `astro.config.mjs` fixes this explicitly:

```js
const base = astroBase ? astroBase.replace(/\/$/, "") : "";
const redirects = base
  ? Object.fromEntries(
      Object.entries(siteConfig.redirects).map(([from, to]) => [from, base + to])
    )
  : siteConfig.redirects;
```

So when `ASTRO_BASE=/site1`, `"/home" → "/en/home"` becomes `"/home" → "/site1/en/home"`, and short URLs like `domain.com/site1/home` correctly reach `domain.com/site1/en/home`.
