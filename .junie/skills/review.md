# Code Review Skill

## Purpose
This is a **code-and-docs comparison skill** — it compares the implementation against all documentation sources and surfaces discrepancies.
Its output is a structured review summary listing every issue found, its severity, and the resolution taken.

## Process

1. **Read all documentation sources**
   - `docs/openapi.yaml` — API routes, request/response schemas, status codes
   - `docs/game_rules.md` — physics constants, game mechanics, spawn/death rules
   - `docs/architecture/data_model.md` — DB schema, in-memory model field names and types
   - `docs/architecture/configuration.md` — config struct shape, field names, validation rules
   - `docs/user_stories/*.md` — acceptance criteria per user story (US01–US12 and any additions)
   - Any other architecture docs in `docs/architecture/`

2. **Do multiple review cycles — one per area**
   - Work through each area in order (see Review Areas below)
   - Collect **all** issues before presenting any; number them clearly

3. **Present issues one by one using the ask tool**
   - For each issue: what the docs say, what the code says, and options (fix code / fix docs / skip)
   - Skip TODO-dependent issues unless the user explicitly asks to address them

4. **Apply the chosen resolution**
   - Fix code or docs as directed; skip if instructed
   - Record the resolution for the summary

5. **Write the review summary**
   - Save to `.ai/review/<YYYY-MM-DD>-<N>-review.md` (zero-padded sequence, e.g. `2026-05-01-01-review.md`) to support multiple reviews on the same date
   - Structure: one section per area, with a table of issues → severity → resolution

## Review Areas (in order)
1. **Physics constants** — gravity, player dimensions, jump strength, bounce schedule, death condition, spawn position
2. **API endpoints** — routes present/missing, request/response field names, status codes, state guards
3. **Configuration** — struct shape (flat vs nested), field names, validation rules, missing fields
4. **Database schema** — table columns vs data_model.md, column names, types, defaults
5. **In-memory models** — struct field names, types, enums vs docs
6. **User stories** — check that each story's acceptance criteria is reflected in the code (routes exist, behaviour matches, edge cases handled)

## Issue Presentation Format

```
**Issue N of M — <short title>**

- **Docs** (`<file> §<section>`): <what docs say>
- **Code** (`<file>:<line>`): <what code does>

**Options:**
1. Fix the code → <description>
2. Fix the docs → <description>
3. Skip / leave as-is
```

## Severity Classification
- 🔴 High — breaks correctness, security, or API contract
- 🟡 Medium — inconsistency that causes confusion or partial breakage
- 🟢 Low — cosmetic or minor naming difference

## Report Format

```markdown
# Code Review — <YYYY-MM-DD>

## Summary
- Total issues found: N
- 🔴 High: X  |  🟡 Medium: Y  |  🟢 Low: Z
- Issues fixed: A  |  Skipped: B

## Physics
| # | Issue | Severity | Resolution |
|---|-------|----------|------------|
| 1 | GRAVITY was -9.8, docs say -10.0 | 🔴 High | Fixed code |

## API
...

## Configuration
...

## Schema
...

## Models
...

## User Stories
...

## Skipped Issues
- Issue N: <reason> (e.g., TODO-dependent)
```
