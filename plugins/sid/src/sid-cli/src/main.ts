#!/usr/bin/env node
import { CliError } from "./http"
import { EXIT } from "./config"

type CommandModule = { run: (argv: string[]) => Promise<unknown> }

const COMMANDS: Record<string, () => Promise<CommandModule>> = {
  search: () => import("./commands/search"),
  docs: () => import("./commands/docs"),
  folders: () => import("./commands/folders"),
  access: () => import("./commands/access"),
  attributes: () => import("./commands/attributes"),
  download: () => import("./commands/download"),
  confirmations: () => import("./commands/confirmations"),
  notifications: () => import("./commands/notifications"),
  check: () => import("./commands/check"),
  settings: () => import("./commands/settings"),
}

const HELP = `sid-cli — přístup k SID (Systém interních dokumentů) přes API klíč

Příkazy:
  search <dotaz> [--size N]                fulltextové hledání
  docs list [--query|--folder|--attribute…] výpis dokumentů s filtry
  docs get <id>                            metadata dokumentu
  docs legislation <id>                    sledovaná legislativa dokumentu
  docs set-validity <id> --from|--to       datum platnosti (nutné EDITOR právo)
  folders list                             složky, do kterých vidím
  access user|group <email|název|id>       (admin) jaká práva má user/skupina ke složkám
  attributes list [--all]                  katalog atributů pro filtrování
  download <fileId|documentId> [--out CESTA]  stažení souboru
  confirmations list [--state pending]     co mám potvrdit (potvrzuje se v SID)
  notifications [--all] [--limit N]        notifikace (default nepřečtené)
  check [--notify]                         digest: novinky + čekající potvrzení
  settings set-key|test|status             správa klíče (set-key čte STDIN)
  settings whoami                          kdo jsem a jestli jsem admin

Klíč: vygenerujte v SID na /api-klice, vložte v panelu pluginu.
Ukládá se do ~/.cdx/apps/sid/.env — nikdy ho nevkládejte do chatu.`

async function main(): Promise<void> {
  const [cmd, ...rest] = process.argv.slice(2)
  if (!cmd || cmd === "--help" || cmd === "help") {
    console.log(HELP)
    return
  }
  const loader = COMMANDS[cmd]
  if (!loader) throw new CliError(`Neznámý příkaz '${cmd}'.\n\n${HELP}`, EXIT.USAGE)

  const command = await loader()
  let result: unknown
  try {
    result = await command.run(rest)
  } catch (e) {
    // `parseArgs` reports bad arguments as a plain Error with an
    // ERR_PARSE_ARGS_* code; without this, misuse would exit 1 instead of
    // the 2 promised by the exit-code table in SKILL.md.
    const code = (e as { code?: string }).code
    if (typeof code === "string" && code.startsWith("ERR_PARSE_ARGS")) {
      throw new CliError(`${(e as Error).message}\n\nsid-cli ${cmd} --help`, EXIT.USAGE)
    }
    throw e
  }
  if (result !== undefined) process.stdout.write(JSON.stringify(result, null, 2) + "\n")
}

main().catch((err: unknown) => {
  if (err instanceof CliError) {
    process.stderr.write(err.message + "\n")
    process.exitCode = err.exitCode
  } else {
    process.stderr.write(`Neočekávaná chyba: ${err instanceof Error ? err.message : String(err)}\n`)
    process.exitCode = EXIT.ERROR
  }
})
