/**
 * siteBase.ts
 *
 * Site base path for constructing internal links.
 * import.meta.env.BASE_URL is "/" in production and "/storage/.../dist/"
 * in preview builds (when ASTRO_BASE is set). It always has a trailing slash.
 *
 * Usage:
 *   import { b } from '~/utils/siteBase';
 *   href={b + lang + "/home"}      // "/en/home" or "/storage/.../dist/en/home"
 *   href={`${b}${lang}/${slug}`}   // same
 */
export const b: string = import.meta.env.BASE_URL; // always has trailing slash
