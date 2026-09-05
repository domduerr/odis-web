/** @type {import('tailwindcss').Config} */
module.exports = {
  content:[
    "index.html",
    "./src/**/*.rs",
  ],
  theme: {
    extend: {
      colors: {
        dhbw: {
          gray: '#5C6971',
          red: '#E2001A',
          'gray-25': 'rgba(92, 105, 113, 0.25)',
          'gray-50': 'rgba(92, 105, 113, 0.5)',
          'gray-75': 'rgba(92, 105, 113, 0.75)',
        }
      }
    }
  },
  plugins:[],
}