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

// src/format.ts
var TYPE_LABELS = {
  AMENDED: "změna znění (novela)",
  UPCOMING_AMENDMENT: "připravovaná změna, zatím nenabyla účinnosti",
  ADDED: "nové ustanovení",
  CHANGED: "změna znění",
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
function watchSetPageUrl(appUrl, watchSetId) {
  return `${appUrl.replace(/\/+$/, "")}/watch-sets/${nodeId("WatchSet", watchSetId)}`;
}
function provisionTitle(law, citation, label) {
  const name = law?.trim() ? `${law.trim()} (${citation})` : citation;
  return label?.trim() ? `${name}, ${label.trim()}` : name;
}
function fold(value) {
  return value.normalize("NFD").replace(/\p{M}/gu, "").toLowerCase();
}
function sectionKey(value) {
  return fold(value).replace(/§|par\.|paragraf/g, "").replace(/\s+/g, "");
}
function isoDate(value) {
  if (!(value instanceof Date))
    return value;
  const pad = (part) => String(part).padStart(2, "0");
  return `${value.getFullYear()}-${pad(value.getMonth() + 1)}-${pad(value.getDate())}`;
}

// src/sources.ts
import { readFileSync as readFileSync2 } from "node:fs";
import { join as join2 } from "node:path";
var SOURCES_CAP = 20;
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
function cite(entries) {
  for (const value of entries) {
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
  const entries = [...pending.values()];
  pending.clear();
  hooked = false;
  if (entries.length === 0)
    return;
  const target = daemon();
  if (!target)
    return;
  try {
    const response = await fetch(target.url, {
      method: "POST",
      headers: { Authorization: target.authorization, "Content-Type": "application/json" },
      body: JSON.stringify({ sources: entries }),
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
  async alerts(query = {}) {
    const raw = await this.all("/alerts", {
      watchSetId: query.watchSet,
      acknowledged: query.unacknowledged ? false : undefined
    });
    const titles = await this.titles(new Set(raw.map((alert) => alert.watchSetId)));
    const alerts = raw.map((alert) => this.alertOf(alert, titles));
    const law = query.law ? fold(query.law) : undefined;
    const section = query.section ? sectionKey(query.section) : undefined;
    const from = query.effectiveFrom ? isoDate(query.effectiveFrom) : undefined;
    const to = query.effectiveTo ? isoDate(query.effectiveTo) : undefined;
    const filtered = alerts.filter((alert) => !law || matchesLaw(law, alert.citation, alert.lawTitle)).filter((alert) => !section || alert.label !== null && sectionKey(alert.label) === section).filter((alert) => !query.types || query.types.includes(alert.type)).filter((alert) => inRange(alert.effectiveFrom, from, to));
    cite(filtered.map(alertSource));
    return filtered;
  }
  async alert(id) {
    const raw = await this.get(`/alerts/${id}`);
    const alert = this.alertOf(raw, await this.titles(new Set([raw.watchSetId])));
    cite([alertSource(alert)]);
    return alert;
  }
  async changes(query = {}) {
    const [raw, documents] = await Promise.all([
      this.all("/changes", {
        from: query.from ? isoDate(query.from) : undefined,
        to: query.to ? isoDate(query.to) : undefined,
        by: query.by,
        watchSetId: query.watchSet
      }),
      this.watchedDocuments(query.watchSet)
    ]);
    const law = query.law ? fold(query.law) : undefined;
    const section = query.section ? sectionKey(query.section) : undefined;
    const changes = raw.map((change) => this.changeOf(change, documents)).filter((change) => !law || matchesLaw(law, change.citation, change.title)).filter((change) => !section || change.caption !== null && sectionKey(change.caption) === section).filter((change) => query.watched === undefined || change.watched === query.watched);
    cite(this.watchSetSources(changes));
    return changes;
  }
  alertOf(raw, titles) {
    const alert = strip(raw);
    alert.lawTitle = titles.get(alert.documentId) ?? null;
    alert.turbolealUrl = alertPageUrl(this.appUrl, alert.id);
    return alert;
  }
  changeOf(raw, documents) {
    const change = strip(raw, ["codexisUrl"]);
    change.watchSetIds = documents.get(change.documentId) ?? [];
    const [watchSet] = change.watchSetIds;
    change.turbolealUrl = watchSet ? watchSetPageUrl(this.appUrl, watchSet) : null;
    return change;
  }
  watchSetSources(changes) {
    return changes.filter((change) => change.turbolealUrl).map((change) => entry(change.turbolealUrl, "Sledovaná legislativa v Turbolealu"));
  }
  async watchedDocuments(watchSet) {
    const ids = watchSet ? [watchSet] : (await this.watchSets()).map((set) => set.id);
    const lists = await Promise.all(ids.map((id) => this.documents(id).catch(() => [])));
    const watched = new Map;
    lists.forEach((documents, index) => {
      for (const document of documents) {
        watched.set(document.documentId, [...watched.get(document.documentId) ?? [], ids[index]]);
      }
    });
    return watched;
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
function strip(raw, fields = HIDDEN_FIELDS) {
  const copy = { ...raw };
  for (const field of fields)
    delete copy[field];
  return copy;
}
function matchesLaw(law, citation, title) {
  return fold(citation).includes(law) || title !== null && fold(title).includes(law);
}
function inRange(date, from, to) {
  if (!from && !to)
    return true;
  if (!date)
    return false;
  return (!from || date >= from) && (!to || date <= to);
}
function alertSource(alert) {
  return entry(alert.turbolealUrl, `Upozornění v Turbolealu: ${provisionTitle(alert.lawTitle, alert.citation, alert.label)}`, changeSnippet(alert.type, alert.effectiveFrom, alert.amendments));
}
function changeSnippet(type, effectiveFrom, amendments) {
  const label = typeLabel(type);
  const parts = [`${label.charAt(0).toUpperCase()}${label.slice(1)}.`];
  if (effectiveFrom)
    parts.push(`Účinnost od ${czechDate(effectiveFrom)}.`);
  if (amendments.length)
    parts.push(`Novela č. ${amendments.map((amendment) => amendment.citation).join(", ")}.`);
  return parts.join(" ");
}
function summarize(item) {
  const isAlert = "sectionId" in item;
  const lawTitle = isAlert ? item.lawTitle : item.title;
  const summary = {
    type: typeLabel(item.type),
    law: lawTitle?.trim() || null,
    citation: item.citation,
    section: (isAlert ? item.label : item.caption) ?? null,
    effectiveFrom: czechDate(item.effectiveFrom),
    detectedAt: czechDate(item.detectedAt),
    amendments: item.amendments.map((amendment) => amendment.citation),
    turbolealUrl: item.turbolealUrl,
    diffMarkdown: item.diffMarkdown ?? (isAlert ? item.diff : null)
  };
  if (isAlert)
    summary.acknowledged = item.acknowledgedAt !== null;
  else
    summary.watched = item.watched;
  return summary;
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
var changes = (query) => client().changes(query);
export {
  watchSets,
  summarize,
  publishSources,
  me,
  documents,
  configure,
  client,
  changes,
  alerts,
  alert,
  TurbolealError,
  Turboleal
};
