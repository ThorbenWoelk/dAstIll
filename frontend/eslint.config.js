import eslint from "@eslint/js";
import prettier from "eslint-config-prettier";
import svelte from "eslint-plugin-svelte";
import globals from "globals";
import tseslint from "typescript-eslint";
import svelteConfig from "./svelte.config.js";

export default tseslint.config(
  {
    ignores: [
      ".svelte-kit/**",
      "build/**",
      "node_modules/**",
      "playwright-report/**",
      "test-results/**",
    ],
  },
  eslint.configs.recommended,
  ...tseslint.configs.recommended,
  ...svelte.configs["flat/recommended"],
  {
    languageOptions: {
      globals: {
        ...globals.browser,
        ...globals.node,
      },
    },
  },
  {
    files: ["**/*.svelte", "**/*.svelte.ts", "**/*.svelte.js"],
    languageOptions: {
      parserOptions: {
        parser: tseslint.parser,
        extraFileExtensions: [".svelte"],
        svelteConfig,
      },
    },
  },
  {
    rules: {
      // ESLint 10: opt in later with `new Error(msg, { cause })` at rethrow sites.
      "preserve-caught-error": "off",
      "svelte/no-navigation-without-resolve": "off",
      "svelte/no-at-html-tags": "off",
      "svelte/no-unused-svelte-ignore": "off",
      "@typescript-eslint/no-unused-expressions": "off",
      "svelte/require-each-key": "off",
      // Svelte 5 reactivity guidance; warn-only (does not fail `eslint` unless --max-warnings 0).
      "svelte/prefer-svelte-reactivity": "warn",
      "svelte/prefer-writable-derived": "warn",
      "@typescript-eslint/no-unused-vars": [
        "error",
        {
          argsIgnorePattern: "^_",
          varsIgnorePattern: "^_",
          caughtErrorsIgnorePattern: "^_",
        },
      ],
    },
  },
  // Large Svelte pages accumulate stale imports; `.ts` / `.svelte.ts` stay strict.
  {
    files: ["**/*.svelte"],
    rules: {
      "@typescript-eslint/no-unused-vars": "off",
    },
  },
  {
    files: ["static/sw.js"],
    languageOptions: {
      globals: {
        ...globals.worker,
        ...globals.serviceworker,
      },
    },
  },
  prettier,
);
