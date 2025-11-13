import type { Config } from 'tailwindcss'

const config: Config = {
  content: [
    './pages/**/*.{js,ts,jsx,tsx,mdx}',
    './components/**/*.{js,ts,jsx,tsx,mdx}',
    './app/**/*.{js,ts,jsx,tsx,mdx}',
  ],
  theme: {
    extend: {
      colors: {
        arc: {
          primary: '#00E5FF',
          secondary: '#0066FF',
          dark: '#0A0E27',
          darker: '#050814',
        },
      },
      backgroundImage: {
        'gradient-radial': 'radial-gradient(var(--tw-gradient-stops))',
        'arc-gradient': 'linear-gradient(135deg, #00E5FF 0%, #0066FF 100%)',
      },
    },
  },
  plugins: [],
}
export default config
