import typography from "@tailwindcss/typography";
import type { Config } from "tailwindcss";

const config: Config = {
  content: ["./src/**/*.{html,js,svelte,ts}"],
  theme: {
    extend: {
      colors: {
        background: "var(--background)",
        foreground: "var(--foreground)",
        surface: "var(--surface)",
        accent: "var(--accent)",
        muted: "var(--muted)",
        border: "var(--border)",
      },
      spacing: {
        xs: "var(--space-xs)",
        sm: "var(--space-sm)",
        md: "var(--space-md)",
        lg: "var(--space-lg)",
        xl: "var(--space-xl)",
      },
      borderRadius: {
        card: "1.5rem",
      },
      boxShadow: {
        soft: "0 20px 40px -25px rgba(84, 36, 36, 0.45)",
      },
    },
  },
  darkMode: "class",
  plugins: [typography],
};

export default config;
