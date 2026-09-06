/** @type {import('tailwindcss').Config} */
module.exports = {
  content:[
    "index.html",
    "./src/**/*.rs",
  ],
  theme: {
    extend: {
      fontFamily: {
        sans: ['ui-sans-serif', 'system-ui', 'Segoe UI', 'Roboto', 'Helvetica Neue', 'Arial', 'sans-serif'],
        mono: ['ui-monospace', 'SFMono-Regular', 'Menlo', 'Consolas', 'Liberation Mono', 'monospace'],
      },
      colors: {
        dhbw: {
          gray: '#5C6971',
          red: '#E2001A',
          'gray-25': 'rgba(92, 105, 113, 0.25)',
          'gray-50': 'rgba(92, 105, 113, 0.5)',
          'gray-75': 'rgba(92, 105, 113, 0.75)',
          'red-50': 'rgba(226, 0, 26, 0.5)',
        }
      }
    }
  },
  plugins:[],
}
