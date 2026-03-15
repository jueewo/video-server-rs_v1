/**
 * siteBase.ts
 *
 * Site base path for constructing internal links.
 * import.meta.env.BASE_URL is "/" in production and "/storage/.../dist/"
 * in preview/subdirectory builds (when ASTRO_BASE is set).
 *
 * We normalize to NO trailing slash so usage is always explicit:
 *   "" in production  →  `${b}/en/home` = "/en/home"
 *   "/storage/.../dist" in preview  →  `${b}/en/home` = "/storage/.../dist/en/home"
 *
 * Usage:
 *   import { b } from '~/utils/siteBase';
 *   href={`${b}/${lang}/home`}
 *   href={b + "/" + lang + menu.link}
 */
const raw = import.meta.env.BASE_URL; // Astro should provide trailing slash, but be defensive
export const b: string = raw === '/' ? '' : raw.replace(/\/$/, '');
