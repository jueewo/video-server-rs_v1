import type { Config } from 'tailwindcss'

export default {
  content: [
    './templates/**/*.html',
    './crates/**/templates/**/*.html',
    './static/**/*.js',
  ],
  safelist: [
    // Gradient backgrounds for document type previews
    'bg-gradient-to-br',
    'from-red-200', 'via-red-300', 'to-pink-300',
    'from-green-200', 'via-emerald-300', 'to-teal-300',
    'from-purple-200', 'via-purple-300', 'to-indigo-300',
    'from-red-200', 'via-orange-300', 'to-amber-300',
    'from-emerald-200', 'via-green-300', 'to-lime-300',
    'from-amber-200', 'via-yellow-300', 'to-orange-300',
    'from-blue-200', 'via-cyan-300', 'to-sky-300',
    'from-slate-200', 'via-gray-300', 'to-zinc-300',
    'from-gray-200',
    // Large text and effects
    'text-8xl',
    'drop-shadow-lg',
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
      'synthwave',
    ],
    darkTheme: 'synthwave',
    base: true,
    styled: true,
    utils: true,
    logs: false,
  },
} satisfies Config
