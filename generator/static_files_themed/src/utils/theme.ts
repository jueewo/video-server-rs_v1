import Config from '../website.config.cjs';

// ─── Theme layout configuration ─────────────────────────────
// Each theme declares layout preferences here. Components read
// `layout.*` instead of checking theme names, so adding a new
// theme never requires touching layout/component files.

export interface ThemeLayout {
  /** Navbar shape: 'floating' pill or full-width 'bar' with accent stripe */
  navbar: 'floating' | 'bar';
  /** Mobile menu style */
  mobileMenu: 'slideout' | 'fullscreen';
  /** Global border-radius scale */
  borderRadius: 'round' | 'sharp';
  /** Heading typography style */
  headingStyle: 'editorial' | 'precision';
  /** Show accent bars in footer categories, TOC, date sections */
  accentBars: boolean;
  /** Breadcrumb separator character */
  breadcrumbSep: string;
  /** Logo hover effect */
  logoHover: 'scale' | 'opacity';
  /** Footer top decoration */
  footerTop: 'line' | 'stripe';
  /** Font files to preload (paths relative to site base) */
  fonts: string[];
}

// ─── Theme definitions ──────────────────────────────────────

const themes: Record<string, ThemeLayout> = {
  starter: {
    navbar: 'floating',
    mobileMenu: 'slideout',
    borderRadius: 'round',
    headingStyle: 'editorial',
    accentBars: false,
    breadcrumbSep: '·',
    logoHover: 'scale',
    footerTop: 'line',
    fonts: ['/fonts/sora-latin.woff2', '/fonts/dm-sans-latin.woff2'],
  },
  ventures: {
    navbar: 'bar',
    mobileMenu: 'fullscreen',
    borderRadius: 'sharp',
    headingStyle: 'precision',
    accentBars: true,
    breadcrumbSep: '/',
    logoHover: 'opacity',
    footerTop: 'stripe',
    fonts: ['/fonts/manrope-latin.woff2', '/fonts/ibm-plex-sans-latin.woff2'],
  },
  mustcato: {
    navbar: 'floating',
    mobileMenu: 'slideout',
    borderRadius: 'round',
    headingStyle: 'editorial',
    accentBars: false,
    breadcrumbSep: '·',
    logoHover: 'scale',
    footerTop: 'line',
    fonts: ['/fonts/sora-latin.woff2', '/fonts/dm-sans-latin.woff2'],
  },
};

// ─── Resolved theme + layout ────────────────────────────────

export const theme: string = Config.componentLib || 'starter';
export const layout: ThemeLayout = themes[theme] || themes.starter;

// ─── Derived helpers ────────────────────────────────────────
// Pre-computed class strings so layouts stay readable.

const isRound = layout.borderRadius === 'round';
const isEditorial = layout.headingStyle === 'editorial';

/** Border-radius classes by context */
export const r = {
  card:   isRound ? 'rounded-2xl' : 'rounded-lg',
  pill:   isRound ? 'rounded-full' : 'rounded-lg',
  dialog: isRound ? 'rounded-2xl' : 'rounded-lg',
  button: isRound ? 'rounded-full' : 'rounded',
  image:  isRound ? 'rounded-2xl' : 'rounded-lg',
  navItem: isRound ? 'rounded-full' : '',
  dropdown: isRound ? 'rounded-2xl' : 'rounded-lg',
  dropdownItem: isRound ? 'first:rounded-t-xl last:rounded-b-xl' : '',
};

/** Heading weight/tracking classes */
export const h = {
  h1Weight:   isEditorial ? 'font-extrabold' : 'font-bold',
  h1Tracking: isEditorial ? 'tracking-[-0.04em]' : 'tracking-[-0.02em]',
  h1Leading:  isEditorial ? 'leading-[1.05]' : 'leading-[1.1]',
  h2Weight:   isEditorial ? 'font-bold' : 'font-semibold',
  h2Tracking: isEditorial ? 'tracking-[-0.035em]' : 'tracking-[-0.02em]',
  h2Leading:  isEditorial ? 'leading-[1.1]' : 'leading-[1.15]',
  h3Weight:   isEditorial ? 'font-bold' : 'font-semibold',
  h3Tracking: isEditorial ? 'tracking-[-0.03em]' : 'tracking-[-0.015em]',
  h3Leading:  isEditorial ? 'leading-[1.15]' : 'leading-[1.2]',
  gradientH1: isEditorial,
  nameTracking: isEditorial ? 'tracking-tight' : 'tracking-wide',
};
