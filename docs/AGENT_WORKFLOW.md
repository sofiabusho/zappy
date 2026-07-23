# Agent workflow — implement one ticket

Copy this prompt into a new agent session. Replace placeholders.

---

You are implementing **exactly one** Zappy ticket on `main` (no PRs, turn-based).

## Ticket

- **ID:** `{Ticket ID}`
- **Title / body:** `{Ticket}`

## Steps (mandatory order)

1. **Pull latest `main`**
   ```bash
   git pull origin main
   ```

2. **Read Source of Truth (in order)**
   - `AGENTS.md`
   - `docs/ticket-tracker.md` (confirm this ticket’s Status, Deps, Coverage)
   - `docs/PRD.md` and `docs/SDS.md` as needed for design
   - `docs/raw/audit.md` and `docs/raw/requirements.md` for acceptance (READ-ONLY)

3. **Claim**
   - Confirm all **Deps** are ✅
   - If not, stop and tell the human
   - Set this ticket to 🟢 and set **Claimed by** to your handle / `agent`
   - Commit tracker claim if the team expects mid-turn visibility; otherwise claim + implement in one push is OK for short tickets

4. **Implement only this ticket’s scope**
   - Do not start the next ticket
   - Do not edit `docs/raw/requirements.md` or `docs/raw/audit.md`
   - Do not expand into adjacent tickets “while you’re here”

5. **Tests + lint**
   - Add/update tests for the ticket’s behavior
   - Run the relevant checks from `AGENTS.md` / `scripts/check.sh`

6. **Verify Coverage IDs**
   - For each RQ*/AQ* listed on the ticket, confirm against `docs/raw/`
   - If an AQ cannot be answered yes with evidence, the ticket is not done

7. **Update tracker**
   - Status → ✅
   - Leave **Claimed by** as historical or clear per team preference
   - Refresh “Who’s up / last push” one-liner

8. **Optional handoff note** (not a PR)
   - `docs/handoff-notes/{Ticket ID}-{slug}.md` using `_template.md`
   - Include what changed, how to verify, what’s next

9. **Push to `main`**
   ```bash
   git push origin main
   ```

10. **Stop**
    - Do not claim or start another ticket unless the human explicitly asks
    - Respect the turn-based model

## Done checklist

- [ ] Code runs
- [ ] Tests pass
- [ ] Lint passes
- [ ] Covered RQ/AQ IDs verified
- [ ] Tracker ✅
- [ ] Pushed to `main`
