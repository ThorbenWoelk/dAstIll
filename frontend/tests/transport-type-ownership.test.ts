import { describe, expect, it } from "bun:test";
import { readdirSync, readFileSync } from "node:fs";
import path from "node:path";

const FRONTEND_ROOT = path.resolve(import.meta.dir, "..");
const SOURCE_ROOT = path.join(FRONTEND_ROOT, "src");
const TYPES_PATH = path.join(SOURCE_ROOT, "lib", "types.ts");

function listTypeScriptFiles(root: string): string[] {
  const results: string[] = [];

  for (const entry of readdirSync(root, { withFileTypes: true })) {
    const entryPath = path.join(root, entry.name);
    if (entry.isDirectory()) {
      results.push(...listTypeScriptFiles(entryPath));
      continue;
    }
    if (entry.isFile() && entry.name.endsWith(".ts")) {
      results.push(entryPath);
    }
  }

  return results;
}

function extractBackendOwnedDtoNames(): string[] {
  const source = readFileSync(TYPES_PATH, "utf8");
  const match = source.match(
    /export type \{([\s\S]*?)\}\s+from "\.\/transport-types";/,
  );

  if (!match) {
    throw new Error("Could not find transport DTO re-export block in types.ts");
  }

  return match[1]
    .split(",")
    .map((name) => name.trim())
    .filter(Boolean);
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function relativeToFrontend(filePath: string): string {
  return path.relative(FRONTEND_ROOT, filePath).replaceAll(path.sep, "/");
}

function extractImportedNames(source: string, importPath: string): string[] {
  const importedNames: string[] = [];

  for (const match of source.matchAll(
    /import\s+type\s*\{([\s\S]*?)\}\s+from\s+["']([^"']+)["'];/g,
  )) {
    if (match[2] !== importPath) {
      continue;
    }
    importedNames.push(
      ...match[1]
        .split(",")
        .map((name) => name.trim())
        .filter(Boolean),
    );
  }

  return importedNames;
}

describe("transport DTO ownership guardrail", () => {
  const backendOwnedDtoNames = extractBackendOwnedDtoNames();
  const dtoNamePattern = backendOwnedDtoNames.map(escapeRegExp).join("|");

  it("keeps backend-owned DTO declarations inside transport-types.ts", () => {
    const declarationPattern = new RegExp(
      String.raw`^\s*(?:export\s+)?(?:interface|type)\s+(${dtoNamePattern})\b`,
      "gm",
    );
    const violations: string[] = [];

    for (const filePath of listTypeScriptFiles(SOURCE_ROOT)) {
      const relativePath = relativeToFrontend(filePath);
      if (
        relativePath === "src/lib/transport-types.ts" ||
        relativePath.startsWith("src/lib/bindings/")
      ) {
        continue;
      }

      const source = readFileSync(filePath, "utf8");
      for (const match of source.matchAll(declarationPattern)) {
        const offset = match.index ?? 0;
        const line = source.slice(0, offset).split("\n").length;
        violations.push(`${relativePath}:${line} redeclares ${match[1]}`);
      }
    }

    expect(violations).toEqual([]);
  });

  it("keeps core transport consumers importing backend DTOs from transport-types", () => {
    const coreTransportFiles = [
      "src/lib/api.ts",
      "src/lib/chat/requests.ts",
      "src/lib/ssr-bootstrap.ts",
      "src/lib/server/load-workspace-bootstrap.ts",
      "src/lib/workspace-cache.ts",
    ];

    for (const relativePath of coreTransportFiles) {
      const source = readFileSync(
        path.join(FRONTEND_ROOT, relativePath),
        "utf8",
      );

      const importedFromTypes = [
        ...extractImportedNames(source, "$lib/types"),
        ...extractImportedNames(source, "./types"),
        ...extractImportedNames(source, "../types"),
      ];
      const leakedDtoImports = importedFromTypes.filter((name) =>
        backendOwnedDtoNames.includes(name),
      );

      expect(
        source.includes('from "$lib/transport-types"') ||
          source.includes("from '$lib/transport-types'") ||
          source.includes('from "./transport-types"') ||
          source.includes("from './transport-types'") ||
          source.includes('from "../transport-types"') ||
          source.includes("from '../transport-types'"),
      ).toBeTrue();
      expect(leakedDtoImports).toEqual([]);
    }
  });
});
