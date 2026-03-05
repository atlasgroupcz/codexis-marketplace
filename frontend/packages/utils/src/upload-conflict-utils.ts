import type { FileConflict, ResolvedConflict } from './file-upload'
import type { FileNode } from './types/file-system'

/**
 * Extract the root folder name from a relative path
 * e.g., "myFolder/subfolder/file.txt" -> "myFolder"
 */
function getRootFolderName(relativePath: string): string | null {
  const parts = relativePath.split('/')
  // If there's more than one part, the first part is the root folder
  if (parts.length > 1 && parts[0]) {
    return parts[0]
  }
  return null
}

/**
 * Detect conflicts between files to upload and existing items in destination
 *
 * For folder uploads (files with relativePaths), we check if the ROOT FOLDER
 * conflicts with an existing item, not individual files within the folder.
 *
 * For regular file uploads, we check each file name against existing items.
 */
export function detectUploadConflicts(
  filesToUpload: Array<File>,
  existingItems: Array<FileNode>,
  relativePaths?: Array<string | undefined>,
): Array<FileConflict> {
  const existingMap = new Map<string, FileNode>()
  for (const item of existingItems) {
    existingMap.set(item.name.toLowerCase(), item)
  }

  const conflicts: Array<FileConflict> = []

  // Check if this is a folder upload by looking at relativePaths
  const isFolderUpload = relativePaths?.some((p) => p && p.includes('/'))

  if (isFolderUpload && relativePaths) {
    // For folder uploads, check root folder conflicts only
    const rootFolders = new Set<string>()

    for (const relativePath of relativePaths) {
      if (relativePath) {
        const rootFolder = getRootFolderName(relativePath)
        if (rootFolder) {
          rootFolders.add(rootFolder)
        }
      }
    }

    // Check each unique root folder for conflicts
    for (const rootFolder of rootFolders) {
      const existing = existingMap.get(rootFolder.toLowerCase())
      if (existing) {
        // Create a synthetic file for the conflict dialog
        // We use the first file in the upload as a representative
        const representativeFile = filesToUpload[0]
        if (!representativeFile) {
          continue
        }
        conflicts.push({
          file: representativeFile,
          name: rootFolder,
          existingType: existing.type === 'directory' ? 'directory' : 'file',
          existingSize: existing.size,
          existingModifiedTime: existing.modifiedTime,
        })
      }
    }
  } else {
    // For regular file uploads, check each file name
    for (const file of filesToUpload) {
      const existing = existingMap.get(file.name.toLowerCase())
      if (existing) {
        conflicts.push({
          file,
          name: file.name,
          existingType: existing.type === 'directory' ? 'directory' : 'file',
          existingSize: existing.size,
          existingModifiedTime: existing.modifiedTime,
        })
      }
    }
  }

  return conflicts
}

/**
 * Generate a unique filename by appending (1), (2), etc.
 * Example: "file.txt" -> "file (1).txt" -> "file (2).txt"
 */
export function generateUniqueName(
  originalName: string,
  existingNames: Set<string>,
): string {
  // Case-insensitive check
  const lowerExisting = new Set(
    Array.from(existingNames).map((n) => n.toLowerCase()),
  )

  if (!lowerExisting.has(originalName.toLowerCase())) {
    return originalName
  }

  // Split into base and extension
  const lastDot = originalName.lastIndexOf('.')
  const hasExtension = lastDot > 0 // Don't treat ".hidden" as extension-only
  const base = hasExtension ? originalName.slice(0, lastDot) : originalName
  const ext = hasExtension ? originalName.slice(lastDot) : ''

  let counter = 1
  let newName = `${base} (${counter})${ext}`

  while (lowerExisting.has(newName.toLowerCase())) {
    counter++
    newName = `${base} (${counter})${ext}`
  }

  return newName
}

/**
 * Result of preparing files for upload, including potentially modified relative paths
 */
export interface PreparedUpload {
  files: Array<File>
  relativePaths: Array<string | undefined>
}

/**
 * Prepare files for upload based on conflict resolutions
 * - skip: file is excluded
 * - replace: file is included as-is
 * - rename: file is renamed before upload
 *
 * For folder uploads, the resolution applies to the root folder:
 * - skip: exclude ALL files from that folder
 * - replace: include ALL files (backend will merge)
 * - rename: rename the root folder in all relative paths
 */
export async function prepareFilesForUpload(
  originalFiles: Array<File>,
  resolutions: Array<ResolvedConflict>,
  existingNames: Set<string>,
  relativePaths?: Array<string | undefined>,
): Promise<PreparedUpload> {
  const resolutionMap = new Map<string, ResolvedConflict>()
  for (const r of resolutions) {
    resolutionMap.set(r.originalName.toLowerCase(), r)
  }

  // Track names we're going to use (for generating unique names)
  const usedNames = new Set(existingNames)

  // Track renamed folders to ensure all files from same folder get same new name
  const renamedFolders = new Map<string, string>()

  const resultFiles: Array<File> = []
  const resultPaths: Array<string | undefined> = []

  // Check if this is a folder upload
  const isFolderUpload = relativePaths?.some((p) => p && p.includes('/'))

  for (let i = 0; i < originalFiles.length; i++) {
    const file = originalFiles[i]
    if (!file) {
      continue
    }
    const relativePath = relativePaths?.[i]

    if (isFolderUpload && relativePath) {
      // For folder uploads, check if the root folder has a resolution
      const [rootFolder = ''] = relativePath.split('/')
      const resolution = rootFolder
        ? resolutionMap.get(rootFolder.toLowerCase())
        : undefined

      if (!resolution) {
        // No conflict for this folder, include as-is
        resultFiles.push(file)
        resultPaths.push(relativePath)
        continue
      }

      switch (resolution.resolution) {
        case 'skip':
          // Don't include files from this folder
          break

        case 'replace':
          // Include as-is (backend will merge/overwrite)
          resultFiles.push(file)
          resultPaths.push(relativePath)
          break

        case 'rename': {
          if (!rootFolder) {
            resultFiles.push(file)
            resultPaths.push(relativePath)
            break
          }

          // Check if we already renamed this folder
          const rootFolderLower = rootFolder.toLowerCase()
          let newRootName = renamedFolders.get(rootFolderLower)

          if (!newRootName) {
            // First time renaming this folder - generate new name
            newRootName =
              resolution.newName ?? generateUniqueName(rootFolder, usedNames)
            usedNames.add(newRootName.toLowerCase())
            renamedFolders.set(rootFolderLower, newRootName)
          }

          // Replace the root folder name in the relative path
          const resolvedRootName = newRootName
          const pathParts = relativePath.split('/')
          pathParts[0] = resolvedRootName
          const newRelativePath = pathParts.join('/')

          resultFiles.push(file)
          resultPaths.push(newRelativePath)
          break
        }
      }
    } else {
      // Regular file upload - check file name
      const resolution = resolutionMap.get(file.name.toLowerCase())

      if (!resolution) {
        // No conflict for this file, include as-is
        resultFiles.push(file)
        resultPaths.push(relativePath)
        usedNames.add(file.name.toLowerCase())
        continue
      }

      switch (resolution.resolution) {
        case 'skip':
          // Don't include this file
          break

        case 'replace':
          // Include as-is (backend will overwrite)
          resultFiles.push(file)
          resultPaths.push(relativePath)
          break

        case 'rename': {
          // Generate unique name and create new File object
          const newName =
            resolution.newName ?? generateUniqueName(file.name, usedNames)
          // Use arrayBuffer() to get the raw content, avoiding Bun runtime bug
          // where passing File/Blob as BlobParts keeps the original name
          const buffer = await file.arrayBuffer()
          const renamedFile = new File([buffer], newName, {
            type: file.type,
            lastModified: file.lastModified,
          })
          resultFiles.push(renamedFile)
          resultPaths.push(relativePath) // Keep original relative path for non-folder uploads
          usedNames.add(newName.toLowerCase())
          break
        }
      }
    }
  }

  return { files: resultFiles, relativePaths: resultPaths }
}

/**
 * Get set of existing file/folder names from FileNode array
 */
export function getExistingNames(items: Array<FileNode>): Set<string> {
  return new Set(items.map((item) => item.name))
}
