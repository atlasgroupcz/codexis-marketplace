import { resolve } from 'node:path'
import babel from '@rolldown/plugin-babel'
import tailwindcss from '@tailwindcss/vite'
import viteReact, { reactCompilerPreset } from '@vitejs/plugin-react'
import type { Plugin, UserConfig } from 'vite'
import { defineConfig, loadEnv, mergeConfig } from 'vite'

/**
 * Strip `crossorigin` attributes from <script> and <link> tags in the built HTML.
 * The app runs inside a sandboxed iframe served by the backend CGI endpoint;
 * crossorigin triggers CORS mode that the backend does not handle.
 */
function stripCrossorigin(): Plugin {
  return {
    name: 'strip-crossorigin',
    enforce: 'post',
    transformIndexHtml(html) {
      return html.replace(/ crossorigin/g, '')
    },
  }
}

export interface AppConfigOptions {
  /** Dev server port. One per app, so several can run side by side. */
  port: number
  /** Where the daemon serves this component's API, for the dev proxy to reach. */
  apiPath: string
  /** Anything this app alone needs, merged over the shared config. */
  extra?: UserConfig
}

/**
 * The build every plugin component UI shares. The apps differ only in their dev
 * port and the daemon path their `/api` proxy points at - everything else was
 * copied into eight files, which is how one of them ended up with its own
 * chunk-size limit and the others with none.
 */
export function createAppConfig({ port, apiPath, extra }: AppConfigOptions) {
  return defineConfig(({ command, mode }) => {
    const env = loadEnv(mode, process.cwd(), '')
    const backendUrl = env.BACKEND_URL || 'http://localhost:8086'

    const base: UserConfig = {
      // Relative for the build: the daemon serves the bundle from a nested path,
      // so index.html must reach its assets relatively. Only for the build - a
      // relative base makes vite 7.3.2+ resolve module ids from the filesystem
      // root, and vitest then cannot find its own test files.
      base: command === 'build' ? './' : '/',
      // The daemon serves these bundles from disk inside the user's VM, so vite's
      // default 500 kB warning does not describe a cost anyone pays. Kept as a
      // ceiling that still catches a bundle growing out of proportion.
      build: {
        chunkSizeWarningLimit: 1000,
      },
      plugins: [
        viteReact(),
        // React Compiler. Under vite 8 the react plugin no longer runs babel
        // itself, so the preset goes through @rolldown/plugin-babel.
        babel({ presets: [reactCompilerPreset()] }),
        tailwindcss(),
        stripCrossorigin(),
      ],
      resolve: {
        alias: {
          '@': resolve(process.cwd(), 'src'),
        },
      },
      server: {
        host: true,
        port,
        proxy: {
          '/api': {
            target: backendUrl,
            changeOrigin: true,
            rewrite: (path) => path.replace(/^\/api/, apiPath),
          },
        },
      },
    }

    return extra ? mergeConfig(base, extra) : base
  })
}
