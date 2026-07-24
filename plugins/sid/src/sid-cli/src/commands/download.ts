import { parseArgs } from "node:util"
import { createWriteStream, mkdirSync, statSync } from "node:fs"
import { Readable } from "node:stream"
import { pipeline } from "node:stream/promises"
import type { ReadableStream as WebReadableStream } from "node:stream/web"
import { resolve, basename, dirname } from "node:path"
import { apiGet, apiRaw, CliError, type FetchLike } from "../http"
import { EXIT } from "../config"

/**
 * Last path segment of the `content-disposition` filename.
 *
 * The server controls that header, so without `basename()` a filename of
 * `../../.bashrc` would escape the working directory. A user-supplied
 * `--out` is deliberately not restricted this way.
 */
function filenameFrom(res: Response, fileId: string): string {
  const cd = res.headers.get("content-disposition") ?? ""
  const m = /filename="?([^";]+)"?/.exec(cd)
  const raw = m?.[1] ?? fileId
  const safe = basename(raw.trim().replace(/\\/g, "/"))
  // basename("..") and basename("/") yield nothing usable.
  return safe && safe !== "." && safe !== ".." ? safe : basename(fileId) || "stazeny-soubor"
}

/**
 * A document's current file, or null when the id isn't a document this user
 * may read.
 *
 * SID gates this endpoint exactly like the download itself, so the lookup
 * grants nothing extra — a document the caller can't view answers 403 here
 * too and the caller keeps the original refusal. Anything else (expired key,
 * rate limit, network) is the real answer and propagates.
 */
async function currentFileIdOf(documentId: string, f?: FetchLike): Promise<string | null> {
  try {
    const doc = (await apiGet(`/api/v1/documents/${encodeURIComponent(documentId)}`, f)) as {
      currentFile?: { id?: string } | null
    }
    return doc.currentFile?.id ?? null
  } catch (e) {
    if (e instanceof CliError && (e.exitCode === EXIT.FORBIDDEN || e.exitCode === EXIT.NOT_FOUND)) {
      return null
    }
    throw e
  }
}

/**
 * Fetch the file, accepting either id.
 *
 * Every listing and search result yields documentIds while this endpoint
 * wants a fileId, and both are bare UUIDs — so the mix-up is the norm, not an
 * edge case. SID answers an unknown file id with 403 rather than 404 (it
 * won't confirm which ids exist), which is indistinguishable from a genuine
 * refusal; hence "403 → try resolving it as a document, once". If that
 * lookup comes back empty the original 403 stands, because then it really was
 * about access.
 */
async function fetchFile(
  id: string,
  f?: FetchLike
): Promise<{ res: Response; fileId: string }> {
  try {
    return { res: await apiRaw(`/api/files/download/${encodeURIComponent(id)}`, f), fileId: id }
  } catch (e) {
    if (!(e instanceof CliError) || e.exitCode !== EXIT.FORBIDDEN) throw e

    const resolved = await currentFileIdOf(id, f)
    if (!resolved) throw e

    // Deliberately not wrapped: a refusal on the resolved file is final, so
    // there is exactly one retry and no chance of a loop.
    return {
      res: await apiRaw(`/api/files/download/${encodeURIComponent(resolved)}`, f),
      fileId: resolved,
    }
  }
}

export async function run(argv: string[], f?: FetchLike): Promise<unknown> {
  const { values, positionals } = parseArgs({
    args: argv,
    allowPositionals: true,
    options: { out: { type: "string" } },
  })
  const id = positionals[0]
  if (!id) {
    throw new CliError("Použití: sid-cli download <fileId|documentId> [--out CESTA]", EXIT.USAGE)
  }

  const { res, fileId } = await fetchFile(id, f)
  const target = resolve(values.out ?? filenameFrom(res, fileId))
  if (!res.body) throw new CliError("Prázdná odpověď serveru.", EXIT.ERROR)

  // `--out ./sub/file.pdf` should work without creating the directory first.
  mkdirSync(dirname(target), { recursive: true })

  // Streamed - documents run to tens of megabytes. The cast goes through
  // `unknown` because the DOM and node:stream/web ReadableStream types are
  // structurally different declarations of the same runtime object.
  await pipeline(Readable.fromWeb(res.body as unknown as WebReadableStream), createWriteStream(target))
  return { savedTo: target, bytes: statSync(target).size }
}
