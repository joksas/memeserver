/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./src/*.rs"],
  plugins: [require("@tailwindcss/forms"), require("@tailwindcss/typography")],
  theme: {
    extend: {
      typography: {
        DEFAULT: {
          css: {
            strong: "none",
            img: "none",
            figure: "none",
            a: "none",
            code: "none",
            "code::before": {
              content: "none",
            },
            "code::after": {
              content: "none",
            },
            pre: "none",
            "pre code": {
              "white-space": "pre-wrap",
            },
          },
        },
      },
    },
  },
}
