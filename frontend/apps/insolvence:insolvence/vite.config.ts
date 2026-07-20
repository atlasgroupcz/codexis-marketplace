import type { Plugin } from 'vite'
import { defineConfig, loadEnv } from 'vite'
import viteReact from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'
import { fileURLToPath, URL } from 'node:url'

function stripCrossorigin(): Plugin {
  return {
    name: 'strip-crossorigin',
    enforce: 'post',
    transformIndexHtml(html) {
      return html.replace(/ crossorigin/g, '')
    },
  }
}

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), '')
  const backendUrl = env.BACKEND_URL || 'http://localhost:8086'

  return {
    base: './',
    test: {
      globals: true,
      // Vitest externalizes deps for its SSR transform by default, which leaves
      // zod's ESM entry resolving to an empty module (`z.object` is undefined at
      // import time). Inlining makes Vitest bundle zod the way the app build
      // does. Same setting katastr already uses.
      server: { deps: { inline: ['zod'] } },
    },
    plugins: [
      viteReact({
        babel: {
          plugins: ['babel-plugin-react-compiler'],
        },
      }),
      tailwindcss(),
      stripCrossorigin(),
    ],
    resolve: {
      alias: {
        '@': fileURLToPath(new URL('./src', import.meta.url)),
      },
    },
    server: {
      host: true,
      port: 3004,
      proxy: {
        '/api': {
          target: backendUrl,
          changeOrigin: true,
          rewrite: (path) =>
            path.replace(
              /^\/api/,
              '/rest/plugin-components/insolvence%40insolvence/insolvence/api',
            ),
        },
      },
    },
  }
})
