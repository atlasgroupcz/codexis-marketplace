import type { Plugin } from 'vite'
import { defineConfig, loadEnv } from 'vite'
import viteReact from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'
import { fileURLToPath, URL } from 'node:url'
import { inlineDeps } from '@workspace/vitest-config/base'

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
      // Which deps must be inlined (and why) lives in @workspace/vitest-config.
      server: { deps: { inline: inlineDeps } },
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
