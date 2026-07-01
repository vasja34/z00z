# Repository Agent Rules and Guidance

- Use repository docs, planning files, and source code directly.
- Keep instructions local to this repository and avoid external documentation graph workflows.
- Validate changes with the repository's normal build, test, and review commands.

## Token discipline

Use compact output by default.

Before producing long text, classify the task:

- simple: answer directly
- medium: short plan + concrete steps
- complex: summary first, then structured sections

For implementation work:

- Prefer code changes over prose.
- Do not explain unchanged code.
- Do not paste full files unless requested.
- Use paths and line references where possible.
- When tests fail, show only the failing command, error essence, and next action.

## Forbidden verbosity

Avoid:

- "Sure, here is..."
- restating the task
- generic caveats
- duplicate bullet points
- long background explanations
- full tutorials unless requested

## Expansion protocol

If more detail is needed, end with:

`EXPANDABLE: details available for <topic>.`

Do not expand unless the user asks.
