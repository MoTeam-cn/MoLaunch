/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{vue,js,ts,jsx,tsx}",
  ],
  safelist: [
    // LoaderCard 动态颜色类
    'bg-orange-100', 'bg-orange-200', 'bg-orange-50',
    'text-orange-700', 'text-orange-800', 'text-orange-500',
    'border-orange-300', 'border-orange-400',
    'hover:bg-orange-200', 'hover:bg-orange-50/50',
    
    'bg-purple-100', 'bg-purple-200', 'bg-purple-50',
    'text-purple-700', 'text-purple-800', 'text-purple-500',
    'border-purple-300', 'border-purple-400',
    'hover:bg-purple-200', 'hover:bg-purple-50/50',
    
    'bg-blue-100', 'bg-blue-200', 'bg-blue-50',
    'text-blue-700', 'text-blue-800', 'text-blue-500',
    'border-blue-300', 'border-blue-400',
    'hover:bg-blue-200', 'hover:bg-blue-50/50',
    
    'bg-green-100', 'bg-green-200', 'bg-green-50',
    'text-green-700', 'text-green-800', 'text-green-500',
    'border-green-300', 'border-green-400',
    'hover:bg-green-200', 'hover:bg-green-50/50',
    
    'bg-teal-100', 'bg-teal-200', 'bg-teal-50',
    'text-teal-700', 'text-teal-800', 'text-teal-500',
    'border-teal-300', 'border-teal-400',
    'hover:bg-teal-200', 'hover:bg-teal-50/50',
  ],
  theme: {
    extend: {
      colors: {
        primary: {
          50: '#eff6ff',
          100: '#dbeafe',
          200: '#bfdbfe',
          300: '#93c5fd',
          400: '#60a5fa',
          500: '#3b82f6',
          600: '#2563eb',
          700: '#1d4ed8',
          800: '#1e40af',
          900: '#1e3a8a',
          950: '#172554',
        },
        page: '#f0f5ff',
      },
      animation: {
        'fade-in': 'fadeIn 0.3s ease-out',
        'slide-in': 'slideIn 0.3s ease-out',
        'slide-up': 'slideUp 0.3s ease-out',
      },
      keyframes: {
        fadeIn: {
          '0%': { opacity: '0' },
          '100%': { opacity: '1' },
        },
        slideIn: {
          '0%': { transform: 'translateX(-100%)' },
          '100%': { transform: 'translateX(0)' },
        },
        slideUp: {
          '0%': { transform: 'translateY(100%)', opacity: '0' },
          '100%': { transform: 'translateY(0)', opacity: '1' },
        },
      },
    },
  },
  plugins: [],
}
