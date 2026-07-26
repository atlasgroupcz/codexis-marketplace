import { parseAsString, parseAsStringLiteral } from 'nuqs'

export const viewParser = parseAsStringLiteral(['list', 'detail'] as const).withDefault('list')

export const uuidParser = parseAsString

export const groupParser = parseAsString

export const tabParser = parseAsStringLiteral(['changes', 'documents'] as const).withDefault(
  'changes',
)

// Top-level section: track CODEXIS acts by id, or watch folders of own documents.
export const sectionParser = parseAsStringLiteral(['documents', 'folders'] as const).withDefault(
  'documents',
)

// Absolute path of the watched folder open in the detail view.
export const rootParser = parseAsString
