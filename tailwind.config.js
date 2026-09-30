/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {
      fontFamily: {
        sans: [
          "Inter",
          "Segoe UI Variable",
          "Segoe UI",
          "system-ui",
          "sans-serif",
        ],
      },
      colors: {
        // Paleta inspirada no AMD Radeon Software Adrenalin / NVIDIA App
        graphite: {
          950: "#0B0C0E",
          900: "#0E0F11",
          850: "#111317",
          800: "#16181C",
          750: "#1B1E23",
          700: "#21242B",
          600: "#2A2E36",
          500: "#3A3F49",
        },
        amd: {
          DEFAULT: "#ED1C24",
          light: "#FF4B52",
          dark: "#B0131A",
          glow: "rgba(237, 28, 36, 0.35)",
        },
        status: {
          ok: "#3DD68C",
          warn: "#F5A623",
          bad: "#FF5C5C",
          info: "#4C9AFF",
        },
      },
      boxShadow: {
        panel: "0 10px 30px rgba(0,0,0,0.45)",
        card: "0 6px 20px rgba(0,0,0,0.35)",
        glow: "0 0 24px rgba(237, 28, 36, 0.25)",
      },
      borderRadius: {
        xl2: "18px",
      },
      keyframes: {
        "fade-in": {
          "0%": { opacity: "0", transform: "translateY(6px)" },
          "100%": { opacity: "1", transform: "translateY(0)" },
        },
        "pulse-red": {
          "0%, 100%": { boxShadow: "0 0 0 0 rgba(237,28,36,0.4)" },
          "50%": { boxShadow: "0 0 0 8px rgba(237,28,36,0)" },
        },
      },
      animation: {
        "fade-in": "fade-in 220ms ease-out",
        "pulse-red": "pulse-red 1.8s infinite",
      },
    },
  },
  plugins: [],
};
