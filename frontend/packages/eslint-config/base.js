import js from "@eslint/js";
import eslintConfigPrettier from "eslint-config-prettier";
import turboPlugin from "eslint-plugin-turbo";
import tseslint from "typescript-eslint";
import onlyWarn from "eslint-plugin-only-warn";

/**
 * A shared ESLint configuration for the repository.
 *
 * @type {import("eslint").Linter.Config}
 * */
export const config = [
  js.configs.recommended,
  eslintConfigPrettier,
  ...tseslint.configs.recommended,
  {
    plugins: {
      turbo: turboPlugin,
    },
    rules: {
      "turbo/no-undeclared-env-vars": "warn",
    },
  },
  {
    plugins: {
      onlyWarn,
    },
  },
  {
    ignores: [
      // Ignore dotfiles
      ".*.?(c)js",
      "*.config*.?(c)js",
      ".*.ts",
      "*.config*.ts",
      "*.d.ts",
      "**/dist/**",
      ".git",
      "**/node_modules/**",
      "**/build/**",
      "**/.next/**",
      "*rollup*",
      "**/public/**",
      "**/.features-gen/**",
      "**/generated/**",
    ],
  },
];
