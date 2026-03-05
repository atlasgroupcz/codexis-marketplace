import {
  FileArchiveIcon,
  FileAudioIcon,
  FileCode2Icon,
  FileIcon,
  FileImageIcon,
  FileJson2Icon,
  FileSpreadsheetIcon,
  FileTextIcon,
  FileVideoIcon,
  FolderIcon,
} from 'lucide-react'
import type { LucideIcon } from 'lucide-react'

/** Extension categories for file type detection */
const CODE_EXTENSIONS = new Set([
  'js',
  'jsx',
  'ts',
  'tsx',
  'py',
  'rb',
  'go',
  'rs',
  'java',
  'c',
  'cpp',
  'h',
  'hpp',
  'cs',
  'php',
  'swift',
  'kt',
  'scala',
  'sh',
  'bash',
  'zsh',
  'fish',
  'ps1',
  'vue',
  'svelte',
])

const JSON_EXTENSIONS = new Set(['json', 'jsonc', 'json5'])

const ARCHIVE_EXTENSIONS = new Set([
  'zip',
  'tar',
  'gz',
  'bz2',
  'xz',
  '7z',
  'rar',
  'tgz',
])

const SPREADSHEET_EXTENSIONS = new Set(['xlsx', 'xls', 'csv', 'ods'])

const IMAGE_EXTENSIONS = new Set([
  'png',
  'jpg',
  'jpeg',
  'gif',
  'webp',
  'svg',
  'ico',
  'bmp',
  'tiff',
  'avif',
])

const VIDEO_EXTENSIONS = new Set([
  'mp4',
  'webm',
  'mov',
  'avi',
  'mkv',
  'flv',
  'wmv',
])

const AUDIO_EXTENSIONS = new Set([
  'mp3',
  'wav',
  'ogg',
  'flac',
  'aac',
  'm4a',
  'wma',
])

const TEXT_EXTENSIONS = new Set([
  'txt',
  'md',
  'mdx',
  'rst',
  'doc',
  'docx',
  'pdf',
  'rtf',
  'tex',
  'log',
])

/** MIME type categories for code files */
const CODE_MIME_SUBTYPES = new Set([
  'javascript',
  'typescript',
  'x-python',
  'x-java',
  'x-c',
  'x-c++',
  'x-ruby',
  'x-go',
  'x-rust',
  'x-php',
  'x-shellscript',
  'x-sh',
  'css',
  'html',
  'xml',
  'x-yaml',
  'yaml',
])

const ARCHIVE_MIME_SUBTYPES = new Set([
  'zip',
  'x-tar',
  'gzip',
  'x-gzip',
  'x-bzip2',
  'x-xz',
  'x-7z-compressed',
  'x-rar-compressed',
  'vnd.rar',
])

const SPREADSHEET_MIME_SUBTYPES = new Set([
  'vnd.ms-excel',
  'vnd.openxmlformats-officedocument.spreadsheetml.sheet',
  'vnd.oasis.opendocument.spreadsheet',
])

const DOCUMENT_MIME_SUBTYPES = new Set([
  'pdf',
  'msword',
  'vnd.openxmlformats-officedocument.wordprocessingml.document',
  'rtf',
  'vnd.oasis.opendocument.text',
])

/**
 * Get file icon based on MIME type and/or extension
 *
 * @param mimeType - The MIME type of the file (e.g., "image/png")
 * @param extension - The file extension without the dot (e.g., "png")
 * @returns The appropriate LucideIcon component for the file type
 */
export function getFileIcon(mimeType?: string | null, extension?: string | null): LucideIcon {
  // Handle extension-based detection first if no MIME type
  if (!mimeType && extension) {
    return getIconByExtension(extension)
  }

  // If no MIME type at all, return generic file icon
  if (!mimeType) {
    return FileIcon
  }

  // Parse MIME type
  const parts = mimeType.split('/')
  const type = parts[0]
  const subtype = parts[1] ?? ''

  // Primary type detection
  if (type === 'image') return FileImageIcon
  if (type === 'video') return FileVideoIcon
  if (type === 'audio') return FileAudioIcon

  // Text types
  if (type === 'text') {
    if (CODE_MIME_SUBTYPES.has(subtype)) return FileCode2Icon
    return FileTextIcon
  }

  // Application types
  if (type === 'application') {
    // JSON (including +json suffix like application/ld+json)
    if (subtype === 'json' || subtype.endsWith('+json')) {
      return FileJson2Icon
    }
    if (ARCHIVE_MIME_SUBTYPES.has(subtype)) return FileArchiveIcon
    if (CODE_MIME_SUBTYPES.has(subtype)) return FileCode2Icon
    if (SPREADSHEET_MIME_SUBTYPES.has(subtype)) return FileSpreadsheetIcon
    if (DOCUMENT_MIME_SUBTYPES.has(subtype)) return FileTextIcon
  }

  // Fallback to extension if available
  if (extension) {
    return getIconByExtension(extension)
  }

  return FileIcon
}

/**
 * Get file icon based on extension only
 */
function getIconByExtension(extension: string): LucideIcon {
  const ext = extension.toLowerCase()

  if (CODE_EXTENSIONS.has(ext)) return FileCode2Icon
  if (JSON_EXTENSIONS.has(ext)) return FileJson2Icon
  if (ARCHIVE_EXTENSIONS.has(ext)) return FileArchiveIcon
  if (SPREADSHEET_EXTENSIONS.has(ext)) return FileSpreadsheetIcon
  if (IMAGE_EXTENSIONS.has(ext)) return FileImageIcon
  if (VIDEO_EXTENSIONS.has(ext)) return FileVideoIcon
  if (AUDIO_EXTENSIONS.has(ext)) return FileAudioIcon
  if (TEXT_EXTENSIONS.has(ext)) return FileTextIcon

  return FileIcon
}

/**
 * Get icon for a file or directory based on type
 *
 * @param isDirectory - Whether this is a directory
 * @param mimeType - The MIME type of the file
 * @param extension - The file extension without the dot
 * @returns The appropriate LucideIcon component
 */
export function getEntryIcon(
  isDirectory: boolean,
  mimeType?: string | null,
  extension?: string | null,
): LucideIcon {
  if (isDirectory) {
    return FolderIcon
  }
  return getFileIcon(mimeType, extension)
}
