/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      fontFamily: {
        sans: ['Roboto', 'sans-serif'],
      },
      colors: {
        'custom': {
          light: '#f0f0f0',
          DEFAULT: '#074368',
          dark: '#d0d0d0',
        }
      }
    },
  },
  plugins: [],
}