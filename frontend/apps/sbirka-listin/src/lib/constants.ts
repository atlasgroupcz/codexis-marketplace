/** The subject's page in the official public register (or.justice.cz), keyed by
 * IČO. Linked from the detail view - the authoritative source of record, listing
 * the full Sbírka listin with working download links. */
export function REGISTER_SUBJECT_URL(ico: string): string {
  return `https://or.justice.cz/ias/ui/rejstrik-$firma?ico=${encodeURIComponent(ico)}`
}

/** Open a stored document on the daemon home page: it previews the file and seeds
 * it as an attachment on a new chat's composer. `path` is the file's absolute path
 * in the user's VM, as returned by the CGI; the state (`?explorer=`) is URL-encoded
 * JSON parsed by the daemon FE with nuqs' `parseAsJson`. The home route reads only
 * `previewPath`, but the schema requires `path` (the folder) to validate, so we
 * send it too. Navigated via `target="_top"` since the component runs in an iframe
 * within the daemon shell. */
export function DAEMON_FILE_URL(path: string): string {
  const slash = path.lastIndexOf('/')
  const parent = slash > 0 ? path.slice(0, slash) : '/'
  const state = { view: 'list', path: parent, previewPath: path }
  return `/?explorer=${encodeURIComponent(JSON.stringify(state))}`
}
