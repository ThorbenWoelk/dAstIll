import { readFileSync } from "node:fs";

export function normalizeRuntimeMode(content) {
  const configuredMode = content
    .split(/\r?\n/)
    .map((line) => line.replace(/\s+#.*$/, ""))
    .map((line) => line.match(/^\s*APP_RUNTIME_MODE\s*=\s*(.*?)\s*$/)?.[1] ?? null)
    .filter((value) => value !== null)
    .at(-1)
    ?.trim()
    .toLowerCase();

  return configuredMode && configuredMode.length > 0 ? configuredMode : "live";
}

export function resolveRuntimeMode(filePath = ".github/runtime-mode.env") {
  try {
    return normalizeRuntimeMode(readFileSync(filePath, "utf8"));
  } catch (error) {
    if (error && typeof error === "object" && "code" in error && error.code === "ENOENT") {
      return "live";
    }
    throw error;
  }
}

if (import.meta.url === new URL(`file://${process.argv[1]}`).href) {
  process.stdout.write(`${resolveRuntimeMode(process.argv[2])}\n`);
}
