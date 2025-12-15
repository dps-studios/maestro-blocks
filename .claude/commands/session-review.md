# Session Review

End-of-session workflow to capture work, update tracking, and commit changes.

## Instructions

When this command is executed, perform the following steps in order:

### 1. Summarize Session Work
Provide a brief summary of what was accomplished:
- Features implemented
- Bugs fixed
- Refactors completed
- Key decisions made

### 2. Close Completed Fizzy Tickets
Check if any existing Fizzy tickets were completed this session:

1. Fetch current cards from the project board using `fizzy_get_cards`
2. Compare against work completed in the conversation
3. Present matches to user:

```
## Completed Tickets

These tickets appear to have been resolved this session:

- #92 "Fix ghost note alignment" — Ghost note now snaps correctly to staff lines
- #95 "Multi-line staff rendering" — Implemented 4 measures per system

Close these tickets? (all / numbers / skip)
```

4. For approved tickets, use `fizzy_close_card` to mark as done

### 3. Extract New Fizzy Tickets
Scan the conversation for NEW actionable items to log:

**Categories:**
- **Bugs** — Issues discovered, errors, unexpected behavior
- **Features** — New functionality requested or deferred
- **Tasks** — Action items, follow-ups, technical debt
- **Ideas** — Suggestions worth capturing for later

**Present as numbered list:**
```
## Proposed New Tickets

1. [Bug] Title — brief description
2. [Feature] Title — brief description
3. [Task] Title — brief description
```

**Ask user:** "Which items should I create? (all / numbers / skip)"

For approved items, create cards on the appropriate Fizzy board with detailed descriptions.

### 4. Update In-Progress Tickets (optional)
If work was started but not completed on existing tickets:
- Add a comment summarizing progress
- Note any blockers or next steps
- Ask user before adding comments

### 5. Git Status Check
Run `git status` and `git diff --stat` to show:
- Modified files
- Untracked files
- Staged changes

**Ask user:** "Ready to commit and push? (yes / no / review changes first)"

### 6. Commit and Push (if approved)
- Stage all relevant changes with `git add .`
- Generate a descriptive commit message based on actual changes:
  - Use imperative mood ("Add feature" not "Added feature")
  - Be specific about what changed
  - Never mention AI/Claude
- Commit and push to remote
- Report success/failure

### 7. Document Key Learnings (optional)
If significant patterns, gotchas, or architectural decisions were discovered:
- Ask if they should be added to `AGENTS.md` or project documentation
- Propose specific additions

### 8. Session Handoff
Provide a brief "next session" summary:
- What's ready to work on next
- Any blockers or dependencies
- Links to relevant Fizzy tickets

---

## Example Usage

```
/session-review
```

## Notes
- Always confirm with user before creating Fizzy tickets or committing
- Skip sections that don't apply (e.g., no bugs found = skip bugs)
- Keep summaries concise but actionable
