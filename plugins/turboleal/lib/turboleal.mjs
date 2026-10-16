// src/config.ts
import { readFileSync } from "node:fs";
import { join } from "node:path";
var APP_ID = "turboleal";
var API_KEY_VAR = "TURBOLEAL_API_KEY";
var API_URL_VAR = "CODEXIS_PLUGIN_TURBOLEAL_API_URL";
var DEFAULT_API_URL = "https://turboleal-profidata-main.onprem.agrp.dev";
function readEnv(name) {
  const value = process.env[name]?.trim();
  return value ? value : undefined;
}
function userHome() {
  return readEnv("CODEXIS_PUBLIC_USER_HOME") ?? readEnv("CDX_USER_HOME") ?? process.env.HOME ?? "/";
}
function envPath() {
  return join(userHome(), ".cdx", "apps", APP_ID, ".env");
}
function readEnvVar(path, key) {
  let contents;
  try {
    contents = readFileSync(path, "utf8");
  } catch {
    return;
  }
  for (const raw of contents.split(`
`)) {
    const line = raw.trim();
    if (!line || line.startsWith("#"))
      continue;
    const at = line.indexOf("=");
    if (at < 0)
      continue;
    if (line.slice(0, at).trim() !== key)
      continue;
    const value = line.slice(at + 1).trim();
    return value || undefined;
  }
  return;
}
function apiUrl() {
  return (readEnv(API_URL_VAR) ?? DEFAULT_API_URL).replace(/\/+$/, "");
}
function apiKey() {
  return readEnv(API_KEY_VAR) ?? readEnvVar(envPath(), API_KEY_VAR);
}

// src/diff.ts
var INSERTED = /\{\+\+([\s\S]*?)\+\+\}/g;
var DELETED = /\{--([\s\S]*?)--\}/g;
var ENUMERATOR = /^(\(\d+[a-z]*\)|[a-z]{1,2}\)|\d+[a-z]*\.)\s*/;
var REFERENCE = /(§|odst\.|odstavc[eiůích]*|odstavec|písm\.|písmen[eaou]?|bod[uůy]?|čl\.|článk[uůy])\s*(\d+[a-z]*|[a-z]{1,2}\))/g;
var SIMILAR = 0.6;
function diffOverview(diffMarkdown) {
  if (!diffMarkdown)
    return null;
  const blocks = diffMarkdown.split(/\n\s*\n/).filter((block) => !block.trimStart().startsWith("#"));
  const marked = blocks.map((block) => block.includes("{--") || block.includes("{++"));
  const before = entries(blocks.map((block) => clean(block.replace(INSERTED, "").replace(DELETED, "$1"))));
  const after = entries(blocks.map((block) => clean(block.replace(DELETED, "").replace(INSERTED, "$1"))));
  const items = [];
  const usedBefore = new Set;
  const usedAfter = new Set;
  const pair = (old, fresh) => {
    usedBefore.add(old);
    usedAfter.add(fresh);
    if (old.enumerator !== fresh.enumerator && (marked[old.block] || marked[fresh.block])) {
      items.push({ kind: "renumbered", before: old.where, after: fresh.where, text: fresh.text });
    }
  };
  for (const sameBlock of [true, false]) {
    for (const old of before) {
      if (usedBefore.has(old))
        continue;
      const fresh = after.find((candidate) => !usedAfter.has(candidate) && candidate.body === old.body && (!sameBlock || candidate.block === old.block));
      if (fresh)
        pair(old, fresh);
    }
  }
  for (const old of before) {
    if (usedBefore.has(old))
      continue;
    const fresh = after.find((candidate) => !usedAfter.has(candidate) && candidate.block === old.block);
    if (fresh && similarity(old.body, fresh.body) >= SIMILAR) {
      usedBefore.add(old);
      usedAfter.add(fresh);
      items.push({
        kind: "changed",
        where: fresh.where,
        before: old.text,
        after: fresh.text,
        referencesOnly: neutral(old.text) === neutral(fresh.text)
      });
    }
  }
  for (const old of before)
    if (!usedBefore.has(old))
      items.push({ kind: "deleted", where: old.where, text: old.text });
  for (const fresh of after)
    if (!usedAfter.has(fresh))
      items.push({ kind: "inserted", where: fresh.where, text: fresh.text });
  items.sort((left, right) => blockOf(left, before, after) - blockOf(right, before, after));
  const renumberingOnly = items.length > 0 && items.every((item) => item.kind === "renumbered" || item.kind === "changed" && item.referencesOnly);
  return { items, renumberingOnly };
}
function blockOf(item, before, after) {
  const text = item.kind === "changed" ? item.after : item.text;
  return (after.find((entry) => entry.text === text) ?? before.find((entry) => entry.text === text))?.block ?? 0;
}
function entries(texts) {
  let paragraph = null;
  let letter = null;
  let point = null;
  const result = [];
  texts.forEach((text, block) => {
    if (!text)
      return;
    const footnote = /^(\d+)\)\s*/.exec(text);
    if (footnote) {
      result.push({ block, text, enumerator: null, body: text.slice(footnote[0].length), where: `poznámka ${footnote[1]})` });
      return;
    }
    const enumerator = ENUMERATOR.exec(text)?.[1] ?? null;
    if (enumerator?.startsWith("(")) {
      paragraph = enumerator.slice(1, -1);
      letter = null;
      point = null;
    } else if (enumerator?.endsWith(")")) {
      letter = enumerator;
      point = null;
    } else if (enumerator) {
      point = enumerator.slice(0, -1);
    }
    const where = [paragraph && `odst. ${paragraph}`, letter && `písm. ${letter}`, point && `bod ${point}`].filter(Boolean).join(" ");
    result.push({ block, text, enumerator, body: text.replace(ENUMERATOR, ""), where: where || null });
  });
  return result;
}
function clean(text) {
  return text.replace(/\\([\\`*_{}[\]()#+\-.!])/g, "$1").replace(/\s+/g, " ").trim();
}
function neutral(text) {
  return text.replace(ENUMERATOR, "#").replace(REFERENCE, "$1 #").replace(/\b[a-z]{1,2}\)/g, "#)");
}
function similarity(left, right) {
  const a = new Set(left.toLowerCase().split(/\s+/));
  const b = new Set(right.toLowerCase().split(/\s+/));
  const shared = [...a].filter((word) => b.has(word)).length;
  return shared / Math.max(1, Math.min(a.size, b.size));
}

// src/format.ts
var TYPE_LABELS = {
  AMENDED: "změna znění (novela)",
  UPCOMING_AMENDMENT: "připravovaná změna, zatím nenabyla účinnosti",
  ADDED: "nové ustanovení",
  REMOVED: "zrušené ustanovení",
  REPEALED: "zrušený předpis"
};
function typeLabel(type) {
  return TYPE_LABELS[type] ?? type;
}
function czechDate(value) {
  if (!value)
    return null;
  const match = /^(\d{4})-(\d{2})-(\d{2})/.exec(value);
  if (!match)
    return value;
  const [, year, month, day] = match;
  return `${Number(day)}. ${Number(month)}. ${year}`;
}
function nodeId(typeName, id) {
  return Buffer.from(`${typeName}:${id}`).toString("base64url");
}
function alertPageUrl(appUrl, alertId) {
  const state = JSON.stringify({ acknowledged: "all", alert: nodeId("Alert", alertId) });
  return `${appUrl.replace(/\/+$/, "")}/alerts?${new URLSearchParams({ alerts: state })}`;
}
function provisionTitle(law, citation, label) {
  const name = law?.trim() ? `${law.trim()} (${citation})` : citation;
  return label?.trim() ? `${name}, ${label.trim()}` : name;
}
function fold(value) {
  return value.normalize("NFD").replace(/\p{M}/gu, "").toLowerCase();
}
function sectionKey(value) {
  return fold(value.split(" - ")[0]).replace(/§|par\.|paragraf/g, "").replace(/\s+/g, "");
}
var PRAGUE = new Intl.DateTimeFormat("sv-SE", { timeZone: "Europe/Prague", year: "numeric", month: "2-digit", day: "2-digit" });
function pragueDate(value) {
  if (typeof value === "string" && /^\d{4}-\d{2}-\d{2}$/.test(value))
    return value;
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? String(value) : PRAGUE.format(date);
}
function isoDate(value) {
  return value instanceof Date ? pragueDate(value) : value;
}

// src/sources.ts
import { readFileSync as readFileSync2 } from "node:fs";
import { join as join2 } from "node:path";
var SOURCES_CAP = 50;
var SNIPPET_MAX_CHARS = 300;
var PUBLISH_TIMEOUT_MS = 1e4;
function entry(url, title, snippet) {
  const value = { __kind: "url", url, title };
  if (snippet)
    value.snippet = [...snippet].slice(0, SNIPPET_MAX_CHARS).join("");
  return value;
}
function daemon() {
  const daemonUrl = readEnv("CODEXIS_PUBLIC_DAEMON_URL");
  const chat = readEnv("CODEXIS_PUBLIC_SESSION_ID");
  const toolCall = readEnv("CODEXIS_PUBLIC_TOOL_CALL_ID");
  const home = readEnv("HOME");
  if (!daemonUrl || !chat || !toolCall || !home)
    return;
  let secret;
  try {
    secret = readFileSync2(join2(home, ".cdx", "env", "secret"), "utf8").trim();
  } catch {
    return;
  }
  if (!secret)
    return;
  const base = daemonUrl.replace(/\/+$/, "").replace(/\/graphql$/, "").replace(/\/+$/, "");
  const run = readEnv("CODEXIS_PUBLIC_SUBAGENT_RUN_ID");
  const url = run ? `${base}/rest/v1/plugin/chats/${chat}/subagents/${run}/tool-calls/${toolCall}/sources` : `${base}/rest/v1/plugin/chats/${chat}/tool-calls/${toolCall}/sources`;
  return { url, authorization: authorization(secret) };
}
function authorization(token) {
  if (/^authorization:/i.test(token))
    return token.slice("authorization:".length).trim();
  if (/^bearer /i.test(token))
    return token;
  const parts = token.split(".");
  return parts.length === 3 && parts.every(Boolean) && !token.includes(" ") ? `Bearer ${token}` : token;
}
var pending = new Map;
var hooked = false;
function cite(entries2) {
  for (const value of entries2) {
    if (pending.size >= SOURCES_CAP)
      break;
    if (!pending.has(value.url))
      pending.set(value.url, value);
  }
  if (!hooked && pending.size > 0) {
    hooked = true;
    process.once("beforeExit", () => void publishSources());
  }
}
async function publishSources() {
  const entries2 = [...pending.values()];
  pending.clear();
  hooked = false;
  if (entries2.length === 0)
    return;
  const target = daemon();
  if (!target)
    return;
  try {
    const response = await fetch(target.url, {
      method: "POST",
      headers: { Authorization: target.authorization, "Content-Type": "application/json" },
      body: JSON.stringify({ sources: entries2 }),
      signal: AbortSignal.timeout(PUBLISH_TIMEOUT_MS)
    });
    if (!response.ok)
      process.stderr.write(`turboleal: failed to attach sources to chat: HTTP ${response.status}
`);
  } catch (error) {
    process.stderr.write(`turboleal: failed to attach sources to chat: ${error.message}
`);
  }
}

// src/client.ts
var PAGE_LIMIT = 200;
var REQUEST_TIMEOUT_MS = 60000;
var HIDDEN_FIELDS = ["codexisUrl", "diffHtml", "remindedAt"];
var UNAUTHORIZED_HINT = "API klíč Turbolealu chybí nebo byl zneplatněn. Vytvořte nový v Turbolealu (API klíče) " + "a uložte ho přes `turboleal-cli auth login`.";

class TurbolealError extends Error {
  status;
  constructor(message, status) {
    super(message);
    this.status = status;
    this.name = "TurbolealError";
  }
}

class Turboleal {
  appUrl;
  key;
  fetcher;
  documentCache = new Map;
  watchSetCache;
  constructor(options = {}) {
    this.appUrl = (options.apiUrl ?? apiUrl()).replace(/\/+$/, "");
    this.key = options.apiKey ?? apiKey();
    this.fetcher = options.fetch ?? ((input, init) => fetch(input, init));
  }
  me() {
    return this.get("/me");
  }
  watchSets() {
    this.watchSetCache ??= this.all("/watch-sets", {});
    return this.watchSetCache;
  }
  documents(watchSetId) {
    let documents = this.documentCache.get(watchSetId);
    if (!documents) {
      documents = this.get(`/watch-sets/${watchSetId}/documents`);
      documents.catch(() => this.documentCache.delete(watchSetId));
      this.documentCache.set(watchSetId, documents);
    }
    return documents;
  }
  async watched(query = {}) {
    const law = query.law ? fold(query.law) : undefined;
    const section = query.section ? sectionKey(query.section) : undefined;
    const sets = await this.watchSets();
    const lists = await Promise.all(sets.map((set) => this.documents(set.id)));
    return sets.flatMap((set, index) => lists[index].filter((document) => !law || matchesLaw(law, document.citation, document.title)).map((document) => ({
      watchSetId: set.id,
      watchSetName: set.name,
      citation: document.citation,
      title: document.title,
      wholeDocument: document.wholeDocument,
      sections: document.sections.flatMap((item) => item.label ? [item.label] : [])
    })).filter((document) => !section || document.wholeDocument || document.sections.some((label) => sectionKey(label) === section)));
  }
  async alerts(query = {}) {
    const raw = await this.all("/alerts", {
      watchSetId: query.watchSet,
      acknowledged: query.unacknowledged ? false : undefined
    });
    const titles = await this.titles(new Set(raw.map((alert) => alert.watchSetId)));
    const law = query.law ? fold(query.law) : undefined;
    const section = query.section ? sectionKey(query.section) : undefined;
    const effective = range(query.effectiveFrom, query.effectiveTo);
    const detected = range(query.detectedFrom, query.detectedTo);
    const alerts = raw.map((alert) => this.alertOf(alert, titles)).filter((alert) => !law || matchesLaw(law, alert.citation, alert.lawTitle)).filter((alert) => !section || alert.label !== null && sectionKey(alert.label) === section).filter((alert) => !query.types || query.types.includes(alert.type)).filter((alert) => inRange(alert.effectiveFrom, effective)).filter((alert) => inRange(pragueDate(alert.detectedAt), detected)).sort((left, right) => (left.effectiveFrom ?? "9999").localeCompare(right.effectiveFrom ?? "9999"));
    cite(alerts.map(alertSource));
    return alerts;
  }
  async alert(id) {
    const raw = await this.get(`/alerts/${id}`);
    const alert = this.alertOf(raw, await this.titles(new Set([raw.watchSetId])));
    cite([alertSource(alert)]);
    return alert;
  }
  alertOf(raw, titles) {
    const alert = strip(raw);
    alert.lawTitle = titles.get(alert.documentId) ?? null;
    alert.turbolealUrl = alertPageUrl(this.appUrl, alert.id);
    return alert;
  }
  async titles(watchSets) {
    const lists = await Promise.all([...watchSets].filter(Boolean).map((id) => this.documents(id).catch(() => [])));
    return new Map(lists.flat().map((document) => [document.documentId, document.title]));
  }
  async all(path, query) {
    const items = [];
    let after;
    do {
      const page = await this.get(path, { ...query, after, limit: PAGE_LIMIT });
      items.push(...page.items);
      after = page.nextCursor ?? undefined;
    } while (after);
    return items;
  }
  async get(path, query = {}) {
    if (!this.key)
      throw new TurbolealError(`Turboleal není připojený. ${UNAUTHORIZED_HINT}`, 401);
    const url = new URL(`${this.appUrl}/api/v1${path}`);
    for (const [name, value] of Object.entries(query)) {
      if (value !== undefined)
        url.searchParams.set(name, String(value));
    }
    const key = this.key.replace(/^bearer /i, "");
    let response;
    try {
      response = await this.fetcher(url, {
        headers: { Authorization: `Bearer ${key}`, Accept: "application/json" },
        signal: AbortSignal.timeout(REQUEST_TIMEOUT_MS)
      });
    } catch (error) {
      throw new TurbolealError(`Turboleal není dostupný: ${error.message}`, 0);
    }
    if (!response.ok)
      throw await problem(response);
    return await response.json();
  }
}
async function problem(response) {
  let detail = "";
  try {
    const body = await response.json();
    detail = body.detail ?? body.title ?? "";
  } catch {}
  const hint = response.status === 401 ? ` ${UNAUTHORIZED_HINT}` : "";
  return new TurbolealError(`HTTP ${response.status}${detail ? `: ${detail}` : ""}${hint}`, response.status);
}
function strip(raw) {
  const copy = { ...raw };
  for (const field of HIDDEN_FIELDS)
    delete copy[field];
  return copy;
}
function matchesLaw(law, citation, title) {
  return fold(citation).includes(law) || title !== null && fold(title).includes(law);
}
function range(from, to) {
  return { from: from ? isoDate(from) : undefined, to: to ? isoDate(to) : undefined };
}
function inRange(date, { from, to }) {
  if (!from && !to)
    return true;
  if (!date)
    return false;
  return (!from || date >= from) && (!to || date <= to);
}
function alertSource(alert) {
  return entry(alert.turbolealUrl, `Upozornění v Turbolealu: ${provisionTitle(alert.lawTitle, alert.citation, alert.label)}`, changeSnippet(alert));
}
function changeSnippet(alert) {
  const label = typeLabel(alert.type);
  const parts = [`${label.charAt(0).toUpperCase()}${label.slice(1)}.`];
  if (alert.effectiveFrom)
    parts.push(`Účinnost od ${czechDate(alert.effectiveFrom)}.`);
  if (alert.amendments.length) {
    parts.push(`Novela č. ${alert.amendments.map((amendment) => amendment.citation).join(", ")}.`);
  }
  return parts.join(" ");
}
function summarize(alert, today = new Date) {
  return {
    type: typeLabel(alert.type),
    law: alert.lawTitle?.trim() || null,
    citation: alert.citation,
    section: alert.label ?? null,
    effectiveFrom: czechDate(alert.effectiveFrom),
    inForce: alert.effectiveFrom ? alert.effectiveFrom <= isoDate(today) : null,
    detectedAt: czechDate(pragueDate(alert.detectedAt)),
    amendments: alert.amendments.map((amendment) => amendment.citation),
    acknowledged: alert.acknowledgedAt !== null,
    turbolealUrl: alert.turbolealUrl,
    overview: diffOverview(alert.diffMarkdown),
    diffMarkdown: alert.diffMarkdown ?? alert.diff
  };
}

// src/index.ts
var shared;
function client() {
  shared ??= new Turboleal;
  return shared;
}
function configure(options = {}) {
  shared = new Turboleal(options);
  return shared;
}
var me = () => client().me();
var watchSets = () => client().watchSets();
var documents = (watchSetId) => client().documents(watchSetId);
var alerts = (query) => client().alerts(query);
var alert = (id) => client().alert(id);
var watched = (query) => client().watched(query);
export {
  watched,
  watchSets,
  summarize,
  publishSources,
  me,
  documents,
  diffOverview,
  configure,
  client,
  alerts,
  alert,
  TurbolealError,
  Turboleal
};
