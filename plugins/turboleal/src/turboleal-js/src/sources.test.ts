import { afterEach, beforeEach, describe, expect, it } from "bun:test"
import { mkdirSync, mkdtempSync, writeFileSync } from "node:fs"
import { tmpdir } from "node:os"
import { join } from "node:path"
import { cite, daemon, entry, publishSources, SOURCES_CAP } from "./sources"

const ENV = ["HOME", "CODEXIS_PUBLIC_DAEMON_URL", "CODEXIS_PUBLIC_SESSION_ID", "CODEXIS_PUBLIC_TOOL_CALL_ID", "CODEXIS_PUBLIC_SUBAGENT_RUN_ID"]
const saved = Object.fromEntries(ENV.map((name) => [name, process.env[name]]))
const realFetch = globalThis.fetch

beforeEach(async () => {
  delete process.env.CODEXIS_PUBLIC_DAEMON_URL
  await publishSources()
  const home = mkdtempSync(join(tmpdir(), "turboleal-js-"))
  mkdirSync(join(home, ".cdx", "env"), { recursive: true })
  writeFileSync(join(home, ".cdx", "env", "secret"), "daemon-secret\n")
  process.env.HOME = home
  process.env.CODEXIS_PUBLIC_DAEMON_URL = "http://daemon.local/graphql"
  process.env.CODEXIS_PUBLIC_SESSION_ID = "chat-1"
  process.env.CODEXIS_PUBLIC_TOOL_CALL_ID = "call-1"
  delete process.env.CODEXIS_PUBLIC_SUBAGENT_RUN_ID
})

afterEach(() => {
  globalThis.fetch = realFetch
  for (const name of ENV) {
    if (saved[name] === undefined) delete process.env[name]
    else process.env[name] = saved[name]
  }
})

describe("sources", () => {
  it("posts the queued Turboleal pages once to the tool call endpoint", async () => {
    const posts: { url: string; headers: Headers; body: unknown }[] = []
    globalThis.fetch = (async (url: string, init: RequestInit) => {
      posts.push({ url, headers: new Headers(init.headers), body: JSON.parse(String(init.body)) })
      return new Response("{}")
    }) as unknown as typeof fetch

    cite([entry("https://t/a", "A", "x".repeat(400)), entry("https://t/a", "A again")])
    cite(Array.from({ length: 60 }, (_, index) => entry(`https://t/${index}`, "B")))
    await publishSources()
    await publishSources()

    expect(posts).toHaveLength(1)
    expect(posts[0].url).toBe("http://daemon.local/rest/v1/plugin/chats/chat-1/tool-calls/call-1/sources")
    expect(posts[0].headers.get("authorization")).toBe("daemon-secret")
    const sources = (posts[0].body as { sources: unknown[] }).sources
    expect(sources).toHaveLength(SOURCES_CAP)
    expect(sources[0]).toEqual({ __kind: "url", url: "https://t/a", title: "A", snippet: "x".repeat(300) })
  })

  it("never throws when the daemon is unreachable", async () => {
    globalThis.fetch = (async () => {
      throw new Error("connection refused")
    }) as unknown as typeof fetch

    cite([entry("https://t/a", "A")])

    await expect(publishSources()).resolves.toBeUndefined()
  })

  it("stays silent outside a chat and routes subagent calls", () => {
    process.env.CODEXIS_PUBLIC_SUBAGENT_RUN_ID = "run-1"
    expect(daemon()?.url).toBe("http://daemon.local/rest/v1/plugin/chats/chat-1/subagents/run-1/tool-calls/call-1/sources")

    delete process.env.CODEXIS_PUBLIC_DAEMON_URL
    expect(daemon()).toBeUndefined()
  })
})
