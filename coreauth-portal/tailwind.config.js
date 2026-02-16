/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        // core.auth brand colors (from brand guidelines)
        brand: {
          black: '#0A0A0A',
          white: '#FFFFFF',
          grey: {
            900: '#111111',
            700: '#555555',
            500: '#777777',
            400: '#999999',
            300: '#AAAAAA',
            200: '#CCCCCC',
            100: '#EEEEEE',
            50: '#F7F7F7',
          },
        },
        // Keep primary for compatibility
        primary: {
          50: '#F7F7F7',
          100: '#EEEEEE',
          200: '#CCCCCC',
          300: '#AAAAAA',
          400: '#999999',
          500: '#777777',
          600: '#555555',
          700: '#111111',
          800: '#111111',
          900: '#0A0A0A',
        },
      },
      fontFamily: {
        sans: ['Inter', '-apple-system', 'BlinkMacSystemFont', 'Segoe UI', 'sans-serif'],
        mono: ['SF Mono', 'Fira Code', 'Monaco', 'Consolas', 'monospace'],
      },
    },
  },
  plugins: [],
}
