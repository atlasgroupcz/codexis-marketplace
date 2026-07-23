import type { Plugin } from 'vite'
import { defineConfig, loadEnv } from 'vite'
import viteReact from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'
import { fileURLToPath, URL } from 'node:url'

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

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), '')
  const backendUrl = env.BACKEND_URL || 'http://localhost:8086'

  return {
    base: './',
    build: {
      chunkSizeWarningLimit: 700,
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
      port: 3001,
      proxy: {
        '/api': {
          target: backendUrl,
          changeOrigin: true,
          rewrite: (path) =>
            path.replace(
              /^\/api/,
              '/rest/plugin-components/codexis%40codexis-plugins/sledovane-dokumenty',
            ),
        },
      },
    },
  }
})
