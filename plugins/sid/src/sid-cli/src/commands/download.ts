import { parseArgs } from "node:util"
import { createWriteStream, mkdirSync, statSync } from "node:fs"
import { Readable } from "node:stream"
import { pipeline } from "node:stream/promises"
import type { ReadableStream as WebReadableStream } from "node:stream/web"
import { resolve, basename, dirname } from "node:path"
import { apiRaw, CliError, type FetchLike } from "../http"
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

export async function run(argv: string[], f?: FetchLike): Promise<unknown> {
  const { values, positionals } = parseArgs({
    args: argv,
    allowPositionals: true,
    options: { out: { type: "string" } },
  })
  const fileId = positionals[0]
  if (!fileId) throw new CliError("Použití: sid-cli download <fileId> [--out CESTA]", EXIT.USAGE)

  const res = await apiRaw(`/api/files/download/${encodeURIComponent(fileId)}`, f)
  const target = resolve(values.out ?? filenameFrom(res, fileId))
  if (!res.body) throw new CliError("Prázdná odpověď serveru.", EXIT.ERROR)

  // `--out ./sub/file.pdf` should work without creating the directory first.
  mkdirSync(dirname(target), { recursive: true })

  // Streamed — documents run to tens of megabytes. The cast goes through
  // `unknown` because the DOM and node:stream/web ReadableStream types are
  // structurally different declarations of the same runtime object.
  await pipeline(Readable.fromWeb(res.body as unknown as WebReadableStream), createWriteStream(target))
  return { savedTo: target, bytes: statSync(target).size }
}
