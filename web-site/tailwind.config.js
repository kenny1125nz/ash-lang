/** @type {import('tailwindcss').Config} */
export default {
  content: ['./src/**/*.{astro,js,ts}'],
  theme: {
    extend: {
      colors: {
        bg: {
          primary: '#fafbfc',
          secondary: '#ffffff',
          tertiary: '#f0f2f5',
        },
        border: '#dde1e6',
        text: {
          primary: '#1a1d23',
          secondary: '#68707a',
        },
        accent: {
          green: '#10a08a',
          blue: '#4d8cf5',
          yellow: '#c4890e',
          red: '#e0554a',
        },
      },
      fontFamily: {
        mono: ['JetBrains Mono', 'monospace'],
        sans: [
          '-apple-system', 'BlinkMacSystemFont',
          '"Segoe UI"', 'Roboto', '"Helvetica Neue"',
          'Arial', 'sans-serif',
        ],
      },
    },
  },
};
