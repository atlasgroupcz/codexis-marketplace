import { readFileSync } from "node:fs"
import { join } from "node:path"
import { readEnv } from "./config"

export const SOURCES_CAP = 50
const SNIPPET_MAX_CHARS = 300
const PUBLISH_TIMEOUT_MS = 10_000

export type SourceEntry = { __kind: "url"; url: string; title: string; snippet?: string }

export function entry(url: string, title: string, snippet?: string | null): SourceEntry {
  const value: SourceEntry = { __kind: "url", url, title }
  if (snippet) value.snippet = [...snippet].slice(0, SNIPPET_MAX_CHARS).join("")
  return value
}

type Daemon = { url: string; authorization: string }

/** The daemon endpoint of the current tool call, or nothing outside a chat VM. */
export function daemon(): Daemon | undefined {
  const daemonUrl = readEnv("CODEXIS_PUBLIC_DAEMON_URL")
  const chat = readEnv("CODEXIS_PUBLIC_SESSION_ID")
  const toolCall = readEnv("CODEXIS_PUBLIC_TOOL_CALL_ID")
  const home = readEnv("HOME")
  if (!daemonUrl || !chat || !toolCall || !home) return undefined
  let secret: string
  try {
    secret = readFileSync(join(home, ".cdx", "env", "secret"), "utf8").trim()
  } catch {
    return undefined
  }
  if (!secret) return undefined
  const base = daemonUrl.replace(/\/+$/, "").replace(/\/graphql$/, "").replace(/\/+$/, "")
  const run = readEnv("CODEXIS_PUBLIC_SUBAGENT_RUN_ID")
  const url = run
    ? `${base}/rest/v1/plugin/chats/${chat}/subagents/${run}/tool-calls/${toolCall}/sources`
    : `${base}/rest/v1/plugin/chats/${chat}/tool-calls/${toolCall}/sources`
  return { url, authorization: authorization(secret) }
}

function authorization(token: string): string {
  if (/^authorization:/i.test(token)) return token.slice("authorization:".length).trim()
  if (/^bearer /i.test(token)) return token
  const parts = token.split(".")
  return parts.length === 3 && parts.every(Boolean) && !token.includes(" ") ? `Bearer ${token}` : token
}

const pending = new Map<string, SourceEntry>()
let hooked = false

/** Queues Turboleal pages for the chat "Sources" panel; they go out once, when the script ends. */
export function cite(entries: SourceEntry[]): void {
  for (const value of entries) {
    if (pending.size >= SOURCES_CAP) break
    if (!pending.has(value.url)) pending.set(value.url, value)
  }
  if (!hooked && pending.size > 0) {
    hooked = true
    process.once("beforeExit", () => void publishSources())
  }
}

/** Sends the queued sources now. Best effort: never throws, at most a line on stderr. */
export async function publishSources(): Promise<void> {
  const entries = [...pending.values()]
  pending.clear()
  hooked = false
  if (entries.length === 0) return
  const target = daemon()
  if (!target) return
  try {
    const response = await fetch(target.url, {
      method: "POST",
      headers: { Authorization: target.authorization, "Content-Type": "application/json" },
      body: JSON.stringify({ sources: entries }),
      signal: AbortSignal.timeout(PUBLISH_TIMEOUT_MS),
    })
    if (!response.ok) process.stderr.write(`turboleal: failed to attach sources to chat: HTTP ${response.status}\n`)
  } catch (error) {
    process.stderr.write(`turboleal: failed to attach sources to chat: ${(error as Error).message}\n`)
  }
}
