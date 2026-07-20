import { parseArgs } from "node:util"
import { createWriteStream, statSync } from "node:fs"
import { Readable } from "node:stream"
import { pipeline } from "node:stream/promises"
import type { ReadableStream as WebReadableStream } from "node:stream/web"
import { resolve } from "node:path"
import { apiRaw, CliError, type FetchLike } from "../http"
import { EXIT } from "../config"

function filenameFrom(res: Response, fileId: string): string {
  const cd = res.headers.get("content-disposition") ?? ""
  const m = /filename="?([^";]+)"?/.exec(cd)
  return m?.[1] ?? fileId
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

  // Streamem — dokumenty mohou mít desítky MB, žádné bufferování do RAM.
  // Přes `unknown`: DOM ReadableStream (typ `res.body`) a node:stream/web
  // ReadableStream (co bere fromWeb) jsou strukturálně různé deklarace téhož
  // runtime objektu, takže přímý cast TS odmítne.
  await pipeline(Readable.fromWeb(res.body as unknown as WebReadableStream), createWriteStream(target))
  return { savedTo: target, bytes: statSync(target).size }
}
