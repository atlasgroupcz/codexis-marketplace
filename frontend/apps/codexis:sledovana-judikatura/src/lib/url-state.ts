import { parseAsString, parseAsStringLiteral } from 'nuqs'

export const viewParser = parseAsStringLiteral(['list', 'detail', 'report'] as const).withDefault('list')

export const uuidParser = parseAsString

export const reportParser = parseAsString
