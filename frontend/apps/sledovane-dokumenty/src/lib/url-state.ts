import { parseAsString, parseAsStringLiteral } from 'nuqs'

export const viewParser = parseAsStringLiteral(['list', 'detail'] as const).withDefault('list')

export const uuidParser = parseAsString

export const groupParser = parseAsString

export const tabParser = parseAsStringLiteral(['changes', 'documents'] as const).withDefault(
  'changes',
)

// Top-level section: track CODEXIS acts by id, or watch the documents of own projects.
export const sectionParser = parseAsStringLiteral(['documents', 'projects'] as const).withDefault(
  'documents',
)

// Id of the watched project open in the detail view.
export const projectParser = parseAsString
