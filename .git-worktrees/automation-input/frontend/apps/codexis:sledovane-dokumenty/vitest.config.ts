import { baseConfig } from '@workspace/vitest-config/base'
import { defineConfig, mergeConfig } from 'vitest/config'

// jsdom, globals, the react/tsconfig-paths plugins and the zod inline all come
// from the shared base; only this app's setup file, coverage scope and timeouts
// are local.
export default mergeConfig(
  baseConfig,
  defineConfig({
    test: {
      setupFiles: ['./src/test/setup.ts'],
      coverage: {
        provider: 'v8',
        reporter: ['text', 'html', 'lcov'],
        reportsDirectory: './coverage',
        include: ['src/components/**/*.{ts,tsx}', 'src/lib/**/*.ts', 'src/hooks/**/*.ts'],
        exclude: [
          '**/*.d.ts',
          '**/test/**',
          '**/*.test.{ts,tsx}',
          '**/*.spec.{ts,tsx}',
        ],
      },
      testTimeout: 10000,
      hookTimeout: 10000,
    },
  }),
)
