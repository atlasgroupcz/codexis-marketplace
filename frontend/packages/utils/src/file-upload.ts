import { backendUrl } from '@workspace/utils/url'

export interface UploadedFile {
  name: string
  path: string
  absolutePath: string
  size: number
}

export interface UploadResponse {
  success: boolean
  message: string
  files: Array<UploadedFile>
}

export interface UploadOptions {
  onProgress?: (progress: number) => void
  signal?: AbortSignal
  /** For folder uploads: override relativePaths when files don't have webkitRelativePath */
  relativePaths?: Array<string | undefined>
}

// Conflict detection types
export interface FileConflict {
  file: File
  name: string
  existingType: 'file' | 'directory'
  existingSize?: number | null
  existingModifiedTime?: string | null
}

export type ConflictResolution = 'skip' | 'replace' | 'rename'

export interface ResolvedConflict {
  file: File
  originalName: string
  resolution: ConflictResolution
  newName?: string
}

/**
 * Upload multiple files to a destination folder
 *
 * @param files - Array of File objects to upload
 * @param destination - The destination folder path
 * @param options - Optional settings for progress tracking and cancellation
 * @returns Promise with upload response including file details
 */
export async function uploadFilesToFolder(
  files: Array<File>,
  destination: string,
  options?: UploadOptions,
): Promise<UploadResponse> {
  const formData = new FormData()

  for (const file of files) {
    formData.append('files', file)
  }

  return new Promise((resolve, reject) => {
    const xhr = new XMLHttpRequest()

    xhr.upload.addEventListener('progress', (e) => {
      if (e.lengthComputable) {
        const progress = Math.round((e.loaded / e.total) * 100)
        options?.onProgress?.(progress)
      }
    })

    xhr.addEventListener('load', () => {
      if (xhr.status >= 200 && xhr.status < 300) {
        try {
          resolve(JSON.parse(xhr.responseText))
        } catch {
          reject(new Error('Invalid response from server'))
        }
      } else {
        try {
          const errorResponse = JSON.parse(xhr.responseText)
          reject(new Error(errorResponse.message || 'Upload failed'))
        } catch {
          reject(new Error(xhr.statusText || 'Upload failed'))
        }
      }
    })

    xhr.addEventListener('error', () => reject(new Error('Network error')))
    xhr.addEventListener('abort', () => reject(new Error('Upload cancelled')))

    if (options?.signal) {
      options.signal.addEventListener('abort', () => xhr.abort())
    }

    const encodedDestination = encodeURIComponent(destination)
    xhr.open(
      'POST',
      backendUrl(
        `/rest/files/upload-multiple?destination=${encodedDestination}`,
      ),
    )
    xhr.send(formData)
  })
}

/**
 * Upload a folder (with nested structure) to a destination folder
 *
 * Uses the webkitRelativePath property from the File API to preserve
 * the folder structure. The backend creates subdirectories as needed.
 *
 * @param files - Array of File objects from a folder selection (with webkitRelativePath)
 * @param destination - The destination folder path
 * @param options - Optional settings for progress tracking and cancellation
 * @returns Promise with upload response including file details
 */
export async function uploadFolderToFolder(
  files: Array<File>,
  destination: string,
  options?: UploadOptions,
): Promise<UploadResponse> {
  const formData = new FormData()

  for (let i = 0; i < files.length; i++) {
    const file = files[i]
    if (!file) continue
    formData.append('files', file)
    // Use provided relativePath if available, otherwise use webkitRelativePath from file
    // Fall back to filename if neither is available
    const providedRelativePath = options?.relativePaths?.[i]
    const webkitRelativePath = (file as File & { webkitRelativePath?: string })
      .webkitRelativePath
    const relativePath = providedRelativePath || webkitRelativePath || file.name
    formData.append('relativePaths', relativePath)
  }

  return new Promise((resolve, reject) => {
    const xhr = new XMLHttpRequest()

    xhr.upload.addEventListener('progress', (e) => {
      if (e.lengthComputable) {
        const progress = Math.round((e.loaded / e.total) * 100)
        options?.onProgress?.(progress)
      }
    })

    xhr.addEventListener('load', () => {
      if (xhr.status >= 200 && xhr.status < 300) {
        try {
          resolve(JSON.parse(xhr.responseText))
        } catch {
          reject(new Error('Invalid response from server'))
        }
      } else {
        try {
          const errorResponse = JSON.parse(xhr.responseText)
          reject(new Error(errorResponse.message || 'Upload failed'))
        } catch {
          reject(new Error(xhr.statusText || 'Upload failed'))
        }
      }
    })

    xhr.addEventListener('error', () => reject(new Error('Network error')))
    xhr.addEventListener('abort', () => reject(new Error('Upload cancelled')))

    if (options?.signal) {
      options.signal.addEventListener('abort', () => xhr.abort())
    }

    const encodedDestination = encodeURIComponent(destination)
    xhr.open(
      'POST',
      backendUrl(
        `/rest/files/upload-folder?destination=${encodedDestination}`,
      ),
    )
    xhr.send(formData)
  })
}

/**
 * Check if files are from a folder upload (have webkitRelativePath)
 */
export function isFolderUpload(files: Array<File>): boolean {
  return files.some((file) => {
    const relativePath = (file as File & { webkitRelativePath?: string })
      .webkitRelativePath
    return relativePath && relativePath.length > 0
  })
}
