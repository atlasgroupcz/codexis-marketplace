import { parseAsString, parseAsStringLiteral } from 'nuqs'

export const viewParser = parseAsStringLiteral(['list', 'detail'] as const).withDefault('list')

export const uuidParser = parseAsString

export const groupParser = parseAsString
