import { apiGet, type FetchLike } from "./http"
import { getBaseUrl, readEnvFile, writeEnvValue } from "./config"

/**
 * Links back into the SID web app.
 *
 * The plugin is meant to lead people into SID, not to replace it, so every
 * result that names a document carries the URL where it can actually be
 * opened, read and confirmed.
 *
 * The route depends on the role: SID's `(user)` layout redirects admins to
 * the admin dashboard, so an admin handed a `/files/<id>` link would land
 * nowhere. Admins get `/admin/files/<id>`.
 */

export type Role = "ADMIN" | "USER"

export function documentUrl(documentId: string, role: Role | null): string {
  const base = role === "ADMIN" ? "/admin/files" : "/files"
  return `${getBaseUrl()}${base}/${encodeURIComponent(documentId)}`
}

/** Where pending confirmations live in the app. */
export function confirmationsUrl(role: Role | null): string {
  return `${getBaseUrl()}${role === "ADMIN" ? "/admin/confirmations" : "/confirmations"}`
}

/**
 * The signed-in user's role, cached in the env file.
 *
 * It is written when the key is stored, so this normally costs nothing. For
 * configs predating that, it is fetched once and cached. A failed lookup
 * returns null and callers fall back to the regular-user route rather than
 * dropping the link.
 */
export async function resolveRole(f?: FetchLike): Promise<Role | null> {
  const cached = readEnvFile().SID_USER_ROLE
  if (cached === "ADMIN" || cached === "USER") return cached
  try {
    const me = (await apiGet("/api/v1/users/me", f)) as { role?: unknown }
    const role = me?.role === "ADMIN" ? "ADMIN" : "USER"
    writeEnvValue("SID_USER_ROLE", role)
    return role
  } catch {
    return null
  }
}
