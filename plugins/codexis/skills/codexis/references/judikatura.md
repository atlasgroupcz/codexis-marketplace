## Judikatura Workflow

For case-law or judikatura research, prefer a dedicated decision-finding workflow over generic legal search.
Some phrases meaning case law research - `najdi judikaturu`, `najdi rozsudek`, `existuje rozhodnutí k ...`, `jak rozhodl Nejvyšší soud / Ústavní soud / vrchní soud`, atp.

1. Start in `JD`, not `ALL`, when the user is asking for Czech court decisions.
2. Use `cdx-cli search JD --help` if you need to confirm available filters.
3. When doing case law research ALWAYS delegate JD candidate discovery to subagent if available. Instruct subagent to deliver meaningful snippets and `cdx://doc/...` links.
4. Wait for agents first, then verify. Do not run your own search in parallel with the agent; it will likely be redundant and less effective than the agent's targeted search.
5. After collection promising candidates use subagent to verify the actual content of the decision text and the metadata. Pay attention to:
   - what conduct actually was at issue in the case,
   - whether the cited statement is ratio, context, or only quoted argument,
   - whether the higher court affirmed, reversed, remanded, or decided only a procedural issue.
6. Do not confuse engagement metrics, quoted third-party comments, or procedural background with the punishable conduct or the holding.
7. If `JD` does not surface an exact match, use `ALL`, `COMMENT`, or `LT` only to identify candidate references, then verify back in `JD` before citing.
8. If no exact higher-court decision is verified, say so explicitly and provide the closest verified cases instead of overstating the result.

### Extracting information from specific case law document

To get specific information from a concrete case law use subagent instead of retrieving the whole text. Instruct subagent to retrieve full text of the decision and extract the relevant information from it by first reading the metadata and then reading the text as a whole. Only when court decission text is too large for agent to process let it use tools like `rg` to find relevant passages. 
