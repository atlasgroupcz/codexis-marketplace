import { apiGet, type FetchLike } from "./http"
import { getBaseUrl, readEnvFile, writeEnvValue } from "./config"

/**
 * Links back into the SID web app — the plugin leads people there rather than
 * standing in for it.
 *
 * The route depends on the role: SID's `(user)` layout redirects admins to
 * the admin dashboard, so an admin handed a `/files/<id>` link lands nowhere.
 */

export type Role = "ADMIN" | "USER"

export function documentUrl(documentId: string, role: Role | null): string {
  const base = role === "ADMIN" ? "/admin/files" : "/files"
  return `${getBaseUrl()}${base}/${encodeURIComponent(documentId)}`
}

/**
 * SID's front door. The landing route differs per role, but SID sorts that
 * out itself — this is for the case where there is no one specific place to
 * send someone (a digest with notifications but nothing to confirm).
 */
export function homeUrl(): string {
  return getBaseUrl()
}

export function confirmationsUrl(role: Role | null): string {
  return `${getBaseUrl()}${role === "ADMIN" ? "/admin/confirmations" : "/confirmations"}`
}

/** The file browser scoped to one folder. Admins land on the admin tree. */
export function folderUrl(folderId: string, role: Role | null): string {
  const base = role === "ADMIN" ? "/admin/files" : "/files"
  return `${getBaseUrl()}${base}?folderId=${encodeURIComponent(folderId)}`
}

/**
 * The signed-in user's role. Written when the key is stored, so this normally
 * costs nothing; otherwise fetched once and cached. A failed lookup returns
 * null and the caller falls back to the regular-user route.
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
