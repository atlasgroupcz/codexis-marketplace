function normalizedExtension(extension?: string | null): string {
  return extension?.toLowerCase().replace(/^\./, '') ?? ''
}

function isEnvFile(extension?: string | null, name?: string): boolean {
  if (name) {
    const lowerName = name.toLowerCase()
    if (lowerName === '.env' || lowerName.startsWith('.env.')) {
      return true
    }
  }

  return normalizedExtension(extension) === 'env'
}

function isMarkdownFile(mimeType: string, extension?: string | null): boolean {
  if (mimeType === 'text/markdown' || mimeType === 'text/x-markdown') {
    return true
  }

  return ['md', 'markdown', 'mdx'].includes(normalizedExtension(extension))
}

function isJsonFile(mimeType: string, extension?: string | null): boolean {
  if (mimeType === 'application/json') return true

  return ['json', 'jsonc', 'json5'].includes(normalizedExtension(extension))
}

function isPlainTextFile(mimeType: string, extension?: string | null): boolean {
  if (mimeType === 'text/plain') return true

  return ['txt', 'text', 'log', 'cfg', 'conf', 'ini'].includes(
    normalizedExtension(extension),
  )
}

export function isEditableFile(
  mimeType: string,
  extension?: string | null,
  name?: string,
): boolean {
  return (
    isEnvFile(extension, name) ||
    isMarkdownFile(mimeType, extension) ||
    isJsonFile(mimeType, extension) ||
    isPlainTextFile(mimeType, extension)
  )
}
