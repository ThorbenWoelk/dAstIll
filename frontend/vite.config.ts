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
  },
}));
