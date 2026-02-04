import type { Config } from 'tailwindcss'

export default {
  content: [
    './templates/**/*.html',
    './crates/**/templates/**/*.html',
    './static/**/*.js',
  ],
  plugins: [
    require('daisyui'),
  ],
  daisyui: {
    themes: [
      {
        corporate: {
          'primary': '#667eea',
          'secondary': '#764ba2',
          'accent': '#4ade80',
          'neutral': '#2a2e37',
          'base-100': '#ffffff',
          'base-200': '#f3f4f6',
          'base-300': '#e5e7eb',
          'info': '#3b82f6',
          'success': '#10b981',
          'warning': '#f59e0b',
          'error': '#ef4444',
        },
      },
      'dark',
      'business',
    ],
    darkTheme: 'business',
    base: true,
    styled: true,
    utils: true,
    logs: false,
  },
} satisfies Config
