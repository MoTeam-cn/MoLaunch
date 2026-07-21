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
          // 11 档色阶由 CSS 变量驱动（运行时由 settingsStore.primaryColor 通过 applyPrimaryColor 注入）
          // 默认值在 main.css 的 :root 中定义（Arco 蓝 #165dff 系列）
          50: 'rgb(var(--color-primary-rgb-50) / <alpha-value>)',
          100: 'rgb(var(--color-primary-rgb-100) / <alpha-value>)',
          200: 'rgb(var(--color-primary-rgb-200) / <alpha-value>)',
          300: 'rgb(var(--color-primary-rgb-300) / <alpha-value>)',
          400: 'rgb(var(--color-primary-rgb-400) / <alpha-value>)',
          500: 'rgb(var(--color-primary-rgb-500) / <alpha-value>)',
          600: 'rgb(var(--color-primary-rgb-600) / <alpha-value>)',
          700: 'rgb(var(--color-primary-rgb-700) / <alpha-value>)',
          800: 'rgb(var(--color-primary-rgb-800) / <alpha-value>)',
          900: 'rgb(var(--color-primary-rgb-900) / <alpha-value>)',
          950: 'rgb(var(--color-primary-rgb-950) / <alpha-value>)',
        },
        page: '#f0f5ff',
        // 主题配色
        brand: {
          1: '#343d4a',  // 深灰蓝 - 正文/默认文字/阴影
          2: '#0b5bcb',  // 主蓝 - 标题/Highlight 按钮
          3: '#1370f3',  // 亮蓝 - 悬停态边框
          4: '#4890f5',
          5: '#96c0f9',
          6: '#d5e6fd',
          7: '#e0eafd',  // 按钮悬停背景
          8: '#eaf2fe',
        },
        dialog: {
          bg: '#FBFBFB',           // 弹窗背景
          caption: '#5C5C5C',      // 正文文字（写死，不随主题变）
        },
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
