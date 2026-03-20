import adapter from "@sveltejs/adapter-node";
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";
import { transformWithOxc } from "vite";

const scriptPreprocess = {
  async script({ attributes, content, filename = "" }) {
    if (attributes.lang !== "ts") {
      return undefined;
    }

    const { code, map } = await transformWithOxc(content, filename, {
      lang: "ts",
      target: "esnext",
    });

    return {
      code,
      map,
      attributes: Object.fromEntries(
        Object.entries(attributes).filter(([key]) => key !== "lang"),
      ),
    };
  },
};

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: [scriptPreprocess, vitePreprocess()],
  kit: {
    adapter: adapter(),
  },
};

export default config;
