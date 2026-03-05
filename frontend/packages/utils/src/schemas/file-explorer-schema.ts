import { z } from 'zod'

// Zod schema for file explorer state (persisted in URL)
// All paths are absolute (e.g., /home/user/folder)
export const fileExplorerSchema = z.object({
  view: z.enum(['list', 'grid', 'columns', 'tabular']).default('list'),
  path: z.string(), // For list/grid single-pane views (absolute path)
  path1: z.string().optional(), // For multi-column left panel
  path2: z.string().optional(), // For multi-column right panel
  previewPath: z.string().optional(), // Path of file being previewed
  previewPage: z.number().optional(), // Page number for PDF preview scrolling
})

export type FileExplorerState = z.infer<typeof fileExplorerSchema>

/**
 * Creates a default file explorer state with the given home directory.
 * @param homeDirectory - Absolute path to the home directory
 */
export function createDefaultFileExplorerState(homeDirectory: string): FileExplorerState {
  return {
    view: 'list',
    path: homeDirectory,
  }
}
