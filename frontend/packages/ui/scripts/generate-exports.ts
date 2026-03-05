/**
 * Scans src/components/, src/lib/, src/hooks/ and generates the explicit
 * exports map for package.json. Run with: bun run generate-exports
 *
 * Outputs the JSON exports block to stdout. Use --write to update package.json.
 */

import { readdirSync, readFileSync, writeFileSync } from "node:fs";
import { basename, dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const UI_ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
const SRC = join(UI_ROOT, "src");

function listFiles(dir: string, ext: string): string[] {
  try {
    return readdirSync(dir, { withFileTypes: true })
      .filter((e) => e.isFile() && e.name.endsWith(ext))
      .filter((e) => !e.name.endsWith(`.test${ext}`) && !e.name.endsWith(`.spec${ext}`))
      .map((e) => e.name)
      .sort();
  } catch {
    return [];
  }
}

function collectExports(): Record<string, string> {
  const exports: Record<string, string> = {};

  // Static exports that don't follow the pattern
  exports["./globals.css"] = "./src/styles/globals.css";
  exports["./postcss.config"] = "./postcss.config.mjs";

  // Scan lib/*.ts
  for (const file of listFiles(join(SRC, "lib"), ".ts")) {
    const name = basename(file, ".ts");
    exports[`./lib/${name}`] = `./src/lib/${file}`;
  }

  // Scan hooks/*.ts
  for (const file of listFiles(join(SRC, "hooks"), ".ts")) {
    const name = basename(file, ".ts");
    exports[`./hooks/${name}`] = `./src/hooks/${file}`;
  }

  // Scan components/*.tsx (root level)
  for (const file of listFiles(join(SRC, "components"), ".tsx")) {
    const name = basename(file, ".tsx");
    exports[`./components/${name}`] = `./src/components/${file}`;
  }

  // Scan components/ai-elements/*.tsx
  for (const file of listFiles(join(SRC, "components", "ai-elements"), ".tsx")) {
    const name = basename(file, ".tsx");
    exports[`./components/ai-elements/${name}`] = `./src/components/ai-elements/${file}`;
  }

  return exports;
}

const exports = collectExports();
const shouldWrite = process.argv.includes("--write");

if (shouldWrite) {
  const pkgPath = join(UI_ROOT, "package.json");
  const pkg = JSON.parse(readFileSync(pkgPath, "utf-8"));
  pkg.exports = exports;
  writeFileSync(pkgPath, JSON.stringify(pkg, null, 2) + "\n");
  console.log(`Updated package.json with ${Object.keys(exports).length} exports`);
} else {
  console.log(JSON.stringify(exports, null, 2));
  console.log(`\n${Object.keys(exports).length} exports total. Use --write to update package.json.`);
}
