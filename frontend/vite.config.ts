import { sveltekit } from "@sveltejs/kit/vite";
import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "vite";

export default defineConfig(() => ({
  envPrefix: ["VITE_", "PUBLIC_", "FIREBASE_AUTH_EMULATOR_HOST"],
  plugins: [tailwindcss(), sveltekit()],
  resolve: {
    tsconfigPaths: true,
  },
  server: {
    port: 3000,
    proxy: {
      "/api": {
        target: process.env.VITE_API_BASE || "http://localhost:3544",
        changeOrigin: true,
      },
    },
  },
}));
