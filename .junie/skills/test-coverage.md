# Test Coverage Analysis Skill

## Purpose
This is a **read-only analysis skill** — it must never modify code or tests directly.
Its output is a structured gap report comparing existing tests against user story acceptance criteria.

## Process

1. **Inventory existing tests**
   - Search all `src/` and `migrations/` files for `#[test]` blocks
   - Note the file, test name, and what behaviour it exercises
   - Group by area: physics, world generation, API, auth, DB, config

2. **Read all user stories**
   - Read every file in `docs/user_stories/` (US01–US12 and any additions)
   - For each story, extract the numbered acceptance criteria verbatim

3. **Map criteria to tests**
   - For each acceptance criterion, determine whether an existing test covers it (fully, partially, or not at all)
   - Use this classification:
     - ✅ Covered — a test directly exercises this criterion
     - 🟡 Partial — a test touches the area but does not assert the criterion end-to-end
     - ❌ Missing — no test exists for this criterion

4. **Identify untested areas**
   - Flag entire layers with zero tests (e.g., all API handlers, auth middleware, DB methods)
   - Flag acceptance criteria that depend on integration between layers (e.g., API key auth + player lookup + state guard)

5. **Write the gap report**
   - Save to `.ai/coverage/<YYYY-MM-DD>-<N>-coverage.md` (zero-padded sequence, e.g. `2026-05-01-01-coverage.md`) to support multiple reports on the same date
   - Structure: one section per user story, with a table of criteria → coverage status → notes
   - End with a **Priority List** of the top missing tests, ordered by risk (security/auth gaps first, then game logic, then observability)

## Report Format

```markdown
# Test Coverage Report — <YYYY-MM-DD>

## Summary
- Total acceptance criteria: N
- ✅ Covered: X  |  🟡 Partial: Y  |  ❌ Missing: Z
- Test files: <list>
- Untested layers: <list>

## US01: Basic Movement
| # | Criterion | Status | Notes |
|---|-----------|--------|-------|
| 1 | Bot commands require X-API-Key; 401 if missing | ❌ Missing | No auth middleware test |
| 2 | ... | ... | ... |

## US02: Account Management
...

## Priority List (top gaps by risk)
1. 🔴 **Auth middleware** — no test verifies 401 on missing/invalid API key (US01, US03)
2. 🔴 **LANDED state guard** — no test verifies 409 when jumping while airborne (US01)
3. ...
```

## Coverage Classification Rules
- A criterion is ✅ **Covered** only if a test asserts the specific outcome (status code, field value, state transition)
- A criterion is 🟡 **Partial** if the code path is exercised but the assertion is incomplete or indirect
- A criterion is ❌ **Missing** if no `#[test]` block exercises it at all
- Integration criteria (e.g., "API key maps to one active player") require an integration-level test to be ✅ — a unit test on a sub-function is at most 🟡 Partial

## Priority Classification
- 🔴 High — security, auth, or data integrity (missing auth guard, unauthenticated access, data loss)
- 🟡 Medium — game logic correctness, API contract (wrong status code, missing field, wrong state transition)
- 🟢 Low — observability, edge cases, cosmetic (leaderboard ordering, display name update)
