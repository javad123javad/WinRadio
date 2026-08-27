/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{vue,js,ts,jsx,tsx}",
  ],
  darkMode: 'class',
  theme: {
    // DESIGN.md tokens. Colors are CSS custom properties (see src/styles/main.css)
    // so a single class name (e.g. `bg-surface-raised`) resolves to the dark
    // palette by default and the light palette under `.light` — no `dark:`
    // variants needed anywhere in markup.
    extend: {
      colors: {
        'surface-base': 'rgb(var(--color-surface-base) / <alpha-value>)',
        'surface-raised': 'rgb(var(--color-surface-raised) / <alpha-value>)',
        'surface-raised-high': 'rgb(var(--color-surface-raised-high) / <alpha-value>)',
        'on-surface': 'rgb(var(--color-on-surface) / <alpha-value>)',
        'on-surface-variant': 'rgb(var(--color-on-surface-variant) / <alpha-value>)',
        outline: 'rgb(var(--color-outline) / <alpha-value>)',
        primary: 'rgb(var(--color-primary) / <alpha-value>)',
        'on-primary': 'rgb(var(--color-on-primary) / <alpha-value>)',
        secondary: 'rgb(var(--color-secondary) / <alpha-value>)',
        'on-secondary': 'rgb(var(--color-on-secondary) / <alpha-value>)',
        error: 'rgb(var(--color-error) / <alpha-value>)',
        'on-error': 'rgb(var(--color-on-error) / <alpha-value>)',
      },
      fontFamily: {
        sans: ["'Segoe UI Variable'", "'Segoe UI'", 'system-ui', 'sans-serif'],
      },
      fontSize: {
        display: ['32px', { lineHeight: '1.2', fontWeight: '300' }],
        headline: ['20px', { lineHeight: '1.3', fontWeight: '600' }],
        body: ['14px', { lineHeight: '1.5', fontWeight: '400' }],
        'label-link': ['14px', { lineHeight: '1.4', fontWeight: '500' }],
        'label-caps': ['11px', { lineHeight: '1.3', fontWeight: '600', letterSpacing: '0.06em' }],
        caption: ['12px', { lineHeight: '1.4', fontWeight: '400' }],
      },
      borderRadius: {
        DEFAULT: '2px',
        sm: '2px',
        md: '4px',
      },
      spacing: {
        rail: '280px',
      },
      screens: {
        // Below this, the (reserved, empty) Info Tile grid drops 3-across to
        // 2-across; the Station List rail never collapses (EXPERIENCE.md,
        // Responsive & Platform). Sits between the enforced window minWidth
        // (800px, tauri.conf.json) and the default width (1000px).
        tiles: '900px',
      },
    },
  },
  plugins: [],
}
