/** @type {import('tailwindcss').Config} */
export default {
  content: ['./src/**/*.{html,js,svelte,ts}'],
  theme: {
    extend: {
      colors: {
        'bg-primary': '#ffffff',
        'bg-secondary': '#f5f5f5',
        'bg-hover': '#e8e8e8',
        'text-primary': '#333333',
        'text-secondary': '#666666',
        'border-color': '#e0e0e0',
        'primary-color': '#3b82f6',
      }
    },
  },
  plugins: [],
}
