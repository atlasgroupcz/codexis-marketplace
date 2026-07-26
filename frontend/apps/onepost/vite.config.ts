import { createAppConfig } from '@workspace/vite-config/app'
import { fileURLToPath, URL } from 'node:url'

export default createAppConfig({
  port: 3007,
  apiPath: '/rest/plugin-components/onepost%40onepost/onepost/api',
  extra: {
    server: {
      fs: {
        strict: false,
        allow: [fileURLToPath(new URL('../../', import.meta.url))],
      },
    },
  },
})
