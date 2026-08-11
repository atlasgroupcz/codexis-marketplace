import { fileURLToPath, URL } from 'node:url'
import { createAppConfig } from '@workspace/vite-config/app'

export default createAppConfig({
  port: 3010,
  apiPath: '/rest/plugin-components/datify%40datify/datify/api',
  extra: {
    server: {
      fs: {
        strict: false,
        allow: [fileURLToPath(new URL('../../', import.meta.url))],
      },
    },
  },
})
