import adapter from "@sveltejs/adapter-node";
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";
import { transformWithEsbuild } from "vite";

const scriptPreprocess = {
  async script({ attributes, content, filename = "" }) {
    if (attributes.lang !== "ts") {
      return undefined;
    }

    const { code, map } = await transformWithEsbuild(content, filename, {
      loader: "ts",
      target: "esnext",
      tsconfigRaw: {
        compilerOptions: {
          preserveValueImports: true,
          verbatimModuleSyntax: true,
        },
      },
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
