import { parse as parseYaml } from 'yaml'

export type SkillFrontmatter = {
  name?: string
  description?: string
  version?: string
  license?: string
  compatibility?: string
  metadata?: Record<string, string>
  'allowed-tools'?: string
}

export type ParsedMarkdownWithFrontmatter = {
  frontmatter: SkillFrontmatter | null
  body: string
}

const FRONTMATTER_REGEX = /^---\s*\n([\s\S]*?)\n---\s*\n?([\s\S]*)$/

export function parseMarkdownFrontmatter(
  content: string,
): ParsedMarkdownWithFrontmatter {
  const match = content.match(FRONTMATTER_REGEX)

  if (!match) {
    return { frontmatter: null, body: content }
  }

  const yamlContent = match[1]
  const body = match[2] ?? ''
  if (!yamlContent) {
    return { frontmatter: null, body: body.trim() }
  }

  try {
    const frontmatter = parseYaml(yamlContent) as SkillFrontmatter
    return { frontmatter, body: body.trim() }
  } catch {
    // If YAML parsing fails, treat as no frontmatter
    return { frontmatter: null, body: content }
  }
}
