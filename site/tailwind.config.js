/** @type {import('tailwindcss').Config} */
module.exports = {
  presets: [require("./styles/tailwind-preset.js")],
  content: ["./index.html", "./src/**/*.rs"],
};
