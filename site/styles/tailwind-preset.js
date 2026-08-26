module.exports = {
  theme: {
    extend: {
      colors: {
        accent: {
          200: "rgb(var(--ac-200) / <alpha-value>)",
          300: "rgb(var(--ac-300) / <alpha-value>)",
          400: "rgb(var(--ac-400) / <alpha-value>)",
          500: "rgb(var(--ac-500) / <alpha-value>)",
          600: "rgb(var(--ac-600) / <alpha-value>)",
          700: "rgb(var(--ac-700) / <alpha-value>)",
          900: "rgb(var(--ac-900) / <alpha-value>)",
        },
      },

      opacity: {
        14: "0.14",
        42: "0.42",
        56: "0.56",
        62: "0.62",
        68: "0.68",
        72: "0.72",
        78: "0.78",
      },
      boxShadow: {
        "glow-2xl": "0 2px 75px -15px var(--tw-shadow-color)",
      },
      fontFamily: {
        sans: [
          "Inter",
          "system-ui",
          "-apple-system",
          "Segoe UI",
          "Helvetica",
          "Arial",
          "sans-serif",
        ],
        mono: [
          "ui-monospace",
          "SFMono-Regular",
          "Menlo",
          "Consolas",
          "monospace",
        ],
      },
    },
  },
};
