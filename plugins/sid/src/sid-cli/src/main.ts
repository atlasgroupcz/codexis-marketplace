#!/usr/bin/env node
import { CliError } from "./http"
import { EXIT } from "./config"

type CommandModule = { run: (argv: string[]) => Promise<unknown> }

const COMMANDS: Record<string, () => Promise<CommandModule>> = {
  search: () => import("./commands/search"),
  docs: () => import("./commands/docs"),
}

const HELP = `sid-cli — přístup k SID (Systém interních dokumentů) přes API klíč

Příkazy:
  search <dotaz> [--size N]                fulltextové hledání
  docs list [--query|--folder|--sort…]     výpis dokumentů s filtry
  docs get <id>                            metadata dokumentu

Klíč: vygenerujte v SID na /profile/api-klice, vložte v panelu pluginu.
Ukládá se do ~/.cdx/apps/sid/.env — nikdy ho nevkládejte do chatu.`

async function main(): Promise<void> {
  const [cmd, ...rest] = process.argv.slice(2)
  if (!cmd || cmd === "--help" || cmd === "help") {
    console.log(HELP)
    return
  }
  const loader = COMMANDS[cmd]
  if (!loader) throw new CliError(`Neznámý příkaz '${cmd}'.\n\n${HELP}`, EXIT.USAGE)
  const result = await (await loader()).run(rest)
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
