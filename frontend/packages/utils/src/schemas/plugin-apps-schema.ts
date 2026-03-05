import { z } from 'zod'

// Zod schema for plugin apps state (persisted in URL)
export const pluginAppsSchema = z.object({
  plugin: z.string(),
  component: z.string(),
})

export type PluginAppsState = z.infer<typeof pluginAppsSchema>
