import { parseAsString, parseAsStringLiteral } from 'nuqs'

export const viewParser = parseAsStringLiteral(['list', 'detail'] as const).withDefault('list')

export const cisloRizeniParser = parseAsString

export const filterParser = parseAsStringLiteral(['all', 'unconfirmed'] as const).withDefault('all')
