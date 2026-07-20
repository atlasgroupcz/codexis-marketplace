import { parseArgs } from "node:util"
import { createWriteStream, mkdirSync, statSync } from "node:fs"
import { Readable } from "node:stream"
import { pipeline } from "node:stream/promises"
import type { ReadableStream as WebReadableStream } from "node:stream/web"
import { resolve, basename, dirname } from "node:path"
import { apiRaw, CliError, type FetchLike } from "../http"
import { EXIT } from "../config"

/**
 * Jméno souboru z `content-disposition` — ale jen jeho poslední složka.
 *
 * Hlavičku řídí server, takže bez `basename()` by `filename="../../.bashrc"`
 * po `resolve()` zapsalo mimo pracovní adresář. Uživatelem zadané `--out`
 * se tímhle neomezuje; tam si cíl volí sám vědomě.
 */
function filenameFrom(res: Response, fileId: string): string {
  const cd = res.headers.get("content-disposition") ?? ""
  const m = /filename="?([^";]+)"?/.exec(cd)
  const raw = m?.[1] ?? fileId
  const safe = basename(raw.trim().replace(/\\/g, "/"))
  // basename(".."), basename("/") apod. nedají použitelné jméno.
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

  // `--out ./stazene/smernice.pdf` má fungovat, aniž by uživatel adresář
  // zakládal ručně.
  mkdirSync(dirname(target), { recursive: true })

  // Streamem — dokumenty mohou mít desítky MB, žádné bufferování do RAM.
  // Přes `unknown`: DOM ReadableStream (typ `res.body`) a node:stream/web
  // ReadableStream (co bere fromWeb) jsou strukturálně různé deklarace téhož
  // runtime objektu, takže přímý cast TS odmítne.
  await pipeline(Readable.fromWeb(res.body as unknown as WebReadableStream), createWriteStream(target))
  return { savedTo: target, bytes: statSync(target).size }
}
