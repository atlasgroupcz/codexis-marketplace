import { fileURLToPath, URL } from 'node:url'
import { createAppConfig } from '@workspace/vite-config/app'

export default createAppConfig({
  port: 3011,
  apiPath: '/rest/plugin-components/turboleal%40turboleal/turboleal/api',
  extra: {
    server: {
      fs: {
        strict: false,
        allow: [fileURLToPath(new URL('../../', import.meta.url))],
      },
    },
  },
})
