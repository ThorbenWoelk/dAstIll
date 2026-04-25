import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { resolveRuntimeMode } from "../../scripts/resolve-runtime-mode.mjs";

const currentDir = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(currentDir, "../..");
const runtimeModeFile = resolve(repoRoot, ".github/runtime-mode.env");

export const PLAYWRIGHT_RUNTIME_MODE = resolveRuntimeMode(runtimeModeFile);
export const PLAYWRIGHT_MAINTENANCE_MODE =
  PLAYWRIGHT_RUNTIME_MODE === "maintenance";
