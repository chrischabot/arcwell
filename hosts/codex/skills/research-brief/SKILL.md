# Research Brief

Use this skill when turning already-collected source material into a concise research brief.

Rules:

- Search the wiki before drafting.
- Use `research_brief_from_wiki` for the first local draft.
- Read the cited wiki pages and check that the draft did not overstate them.
- Add a short contradiction/gaps section when sources disagree or freshness is uncertain.
- Do not cite generated `Research Brief:` pages as primary sources.
- If the brief is for publishing, also apply the user's style and voice guidance before final prose.

Typical commands:

```sh
agent wiki search <query>
agent research brief <query> --no-write
agent research brief <query>
agent research runs
```

MCP tools:

- `wiki_search`
- `wiki_read`
- `research_brief_from_wiki`
- `research_runs`
- `research_tasks`
