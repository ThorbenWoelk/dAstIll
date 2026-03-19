import { sveltekit } from "@sveltejs/kit/vite";
import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "vite";

export default defineConfig(({ command }) => ({
  plugins: [tailwindcss(), sveltekit()],
  devtools: command === "serve",
  resolve: {
    tsconfigPaths: true,
  },
  server: {
    port: 3000,
  },
}));
