// @template deep-research
// @use-when A broad open question whose answer must be assembled from many sources, and where every
//   claim has to survive an attempt to refute it before it reaches the user.
// @args question (required) — the research question, in the user's own words
//
// Reference example of a deep-research workflow — the worked "fan-out + adversarial verification" shape.
// A workflow is `export const meta = {...}` plus `export default async function (args) { ... return result }`.
// The `agent(prompt, {label, schema, agentType, skills})` primitive runs an isolated subagent that sees NOTHING but the
// prompt string, using whatever capabilities that agent has (installed skills/tools, the user's files and
// shell, or the internet) and knows today's date. That is why BRIEF below is built once and pasted into
// every prompt, and why each confirmed claim travels on with its source, quote and verification notes
// rather than as a bare sentence — copy that, it is the point of the example.
// parallel(thunks) is a barrier (waits for all) and keeps every position, putting null where a branch
// failed; pipeline(items, produce, consume) streams each item independently, hands (result, item, index)
// to consume, and follows the same null-on-failure contract. Author your own workflow in this shape, tuned to the
// task — tune angle count, verification votes and schemas per task.
export const meta = {
  name: "deep-research",
  description: "Fan-out research: decompose into angles, gather+extract cited claims from the available sources, adversarially verify, synthesize a cited report.",
  phases: [
    { title: "Scope", detail: "Decompose the question into 8-12 research angles" },
    { title: "Research", detail: "Parallel agents extract falsifiable, cited claims per angle from the available sources" },
    { title: "Verify", detail: "Multi-vote adversarial verification per claim" },
    { title: "Synthesize", detail: "Merge, rank by confidence, cite" }
  ]
}

const ANGLES = { type: "object", required: ["angles"], properties: { angles: { type: "array", items: {
  type: "object", required: ["label", "query"], properties: {
    label: { type: "string" }, query: { type: "string" }, rationale: { type: "string" } } } } } }
const CLAIMS = { type: "object", required: ["claims"], properties: { claims: { type: "array", items: {
  type: "object", required: ["claim", "source", "importance"], properties: {
    claim: { type: "string" }, source: { type: "string" }, quote: { type: "string" },
    importance: { enum: ["central", "supporting", "tangential"] } } } } } }
// Four states, never a boolean: "could not check" must never be able to pass for "checked".
const VERDICT = { type: "object", required: ["status", "evidence"], properties: {
  status: { enum: ["supported", "refuted", "unverified", "source_unreachable"] },
  evidence: { type: "string" }, counterSource: { type: "string" },
  relevance: { enum: ["high", "medium", "low"] } } }
const REPORT = { type: "object", required: ["summary", "findings", "caveats"], properties: {
  summary: { type: "string" },
  findings: { type: "array", items: { type: "object", required: ["claim", "confidence", "sources"], properties: {
    claim: { type: "string" }, confidence: { enum: ["high", "medium", "low"] },
    sources: { type: "array", items: { type: "string" } } } } },
  caveats: { type: "string" }, openQuestions: { type: "array", items: { type: "string" } } } }

export default async function (args) {
  const QUESTION = (typeof args === "object" && args && typeof args.question === "string" && args.question.trim()) || ""
  if (!QUESTION) {
    return { error: "No research question. Pass it as args: { question: '<question>' }." }
  }

  const BRIEF = "Výzkumná otázka: " + QUESTION +
    "\n(Odpověz v jazyce otázky. Každé tvrzení musí nést přesný zdroj a doslovnou citaci.)"

  phase("Scope")
  const scope = await agent(BRIEF + "\n\nDecompose this research question into 8 to 12 complementary " +
        "research angles (use more angles the broader the question is). This list drives the whole run: " +
        "each angle becomes its own researcher, so make the boundaries explicit and non-overlapping.\n\n" +
        "Return the angles as structured output.",
        { label: "scope", schema: ANGLES })
  log("Angles: " + scope.angles.map(a => a.label).join(", "))

  phase("Research")
  const perAngle = await parallel(scope.angles.map(a => () =>
    agent(BRIEF + "\n\nResearch this angle using the sources available to you and extract 2-5 FALSIFIABLE " +
          "claims bearing on the question.\n\n" +
          "Angle: " + a.label + "\nSuggested query: " + a.query +
          (a.rationale ? "\nWhy this angle: " + a.rationale : "") + "\n\n" +
          "Your claims go to adversarial verifiers who will re-open your sources, so give them something " +
          "checkable: each claim concrete, and for 'source' ONLY the exact source you actually retrieved " +
          "(a precise citation, document id, or URL) — never invent one. Include the supporting quote " +
          "verbatim. Rate each central/supporting/tangential. Structured output only.",
          { label: a.label, schema: CLAIMS })
      .catch(() => ({ claims: [] }))))

  const allClaims = perAngle.filter(Boolean).flatMap(r => r.claims)
  const impRank = { central: 0, supporting: 1, tangential: 2 }
  const ranked = [...allClaims].sort((x, y) => impRank[x.importance] - impRank[y.importance]).slice(0, 12)
  log("Extracted " + allClaims.length + " claims; verifying top " + ranked.length)
  if (ranked.length === 0) {
    return { question: QUESTION, summary: "No claims extracted.", findings: [], caveats: "Search returned nothing usable." }
  }

  phase("Verify")
  const relRank = { high: 0, medium: 1, low: 2 }
  const verified = await parallel(ranked.map(c => () =>
    parallel([0, 1, 2].map(v => () =>
      agent(BRIEF + "\n\nAdversarial verifier " + (v + 1) + "/3. Be SKEPTICAL; try to REFUTE this claim " +
            "using the sources available to you.\n\n" +
            "Claim: \"" + c.claim + "\"\nSource: " + c.source +
            "\nQuote: " + (c.quote || "") + "\n\n" +
            "Read the cited source in full — don't judge from the snippet alone. Check the claim is still " +
            "current: the source is in force and up to date. If the source is case law or another precedent, " +
            "confirm it is still good authority.\n\n" +
            "Choose status like this: 'supported' only when you opened the source and the claim held up; " +
            "'refuted' when you found a concrete reason it does not hold — overruled, superseded, " +
            "reversed, departed from, or simply not what the source says; 'source_unreachable' when you " +
            "could not open the source at all; 'unverified' when you opened it but could neither confirm " +
            "nor refute. Absence of evidence is NOT refutation, and lack of access is NOT support.\n\n" +
            "Also rate how relevant the claim is to the question (relevance: high/medium/low). Your " +
            "'evidence' is quoted to the writer of the final report, so put the deciding wording in it. " +
            "Structured output only.",
            { label: "verify #" + (v + 1) + ": " + c.claim.slice(0, 40), agentType: "codexis:verifier", schema: VERDICT })
        // A verifier that crashed has verified nothing — counting it as support would let a rate
        // limit promote an unchecked claim into the report.
        .catch(() => ({ status: "unverified", evidence: "verification unavailable" }))))
    .then(votes => {
      const cast = votes.filter(Boolean)
      const supports = cast.filter(x => x.status === "supported").length
      const refutes = cast.filter(x => x.status === "refuted").length
      // A claim has to be actively upheld twice. Silence, an unreachable source and a crashed
      // verifier all count as "not upheld", never as a vote in the claim's favour.
      const survives = supports >= 2 && refutes === 0
      const evidence = cast.filter(x => x.status === "supported" && x.evidence)
        .map(x => x.evidence).join(" | ")
      const relevance = cast.filter(x => x.relevance).map(x => x.relevance)
        .sort((a, b) => relRank[a] - relRank[b])[0] || "medium"
      log("\"" + c.claim.slice(0, 48) + "…\": " + supports + "-" + refutes
        + "/" + cast.length + (survives ? " ✓" : " ✗"))
      return { claim: c, survives, supports, refutes, evidence, relevance }
    })))

  const checked = verified.filter(Boolean)
  const confirmed = checked.filter(v => v.survives)
    .sort((a, b) => relRank[a.relevance] - relRank[b.relevance])
  const killed = checked.filter(v => !v.survives).map(v => v.claim)
  if (confirmed.length === 0) {
    return { question: QUESTION, summary: "All claims refuted by adversarial verification.", findings: [],
      caveats: "Sources were low-quality or claims overstated.",
      refuted: killed.map(c => ({ claim: c.claim, source: c.source })) }
  }

  // Pass the RICH context forward: each confirmed claim keeps its source, the gathering agent's supporting
  // quote, and the verifiers' corroboration — so the synthesizer writes grounded findings, not a re-statement.
  const block = confirmed.map((v, i) => {
    const c = v.claim
    return "[" + i + "] " + c.claim +
      "\n    zdroj: " + c.source +
      "\n    relevance: " + v.relevance +
      (c.quote ? "\n    citace: \"" + c.quote + "\"" : "") +
      (v.evidence ? "\n    ověření: " + v.evidence : "")
  }).join("\n\n")
  phase("Synthesize")
  const report = await agent(BRIEF + "\n\nSynthesize a cited research report." +
        "\n\nConfirmed claims (each with its source, relevance, supporting quote and verification notes):\n" + block +
        "\n\nRead ALL the material above in full before writing — every claim, quote and verification note, " +
        "not just the snippets. Ground every finding in the quotes and verification notes; this material is " +
        "everything you have, so introduce no claim that is not in it. Merge duplicates, group into findings " +
        "that answer the question, and ORDER the findings from most to least relevant/central to the " +
        "question. Give more weight to current, primary and still-authoritative sources; down-rank or caveat " +
        "anything outdated or superseded. Assign confidence, write a 3-5 sentence summary, note caveats and " +
        "2-4 open questions. In each finding's 'sources' put ONLY the exact sources (precise citation, " +
        "document id, or URL) carried by the confirmed claims — never invent any. Write findings as plain " +
        "statements without meta-commentary about the research process. Structured output only.",
        { label: "synthesize", agentType: "codexis:synthesizer", schema: REPORT })

  return { question: QUESTION, ...report,
    refuted: killed.map(c => ({ claim: c.claim, source: c.source })),
    stats: { angles: scope.angles.length, claims: allClaims.length, verified: checked.length, confirmed: confirmed.length } }
}
