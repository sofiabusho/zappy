# Handoff note — `C02` `command-pipeline`

> Short turn summary. Not a PR.

## Summary

- Added `zappy_client/pipeline.py`: exact subject command syntax (RQ11) via a
  `Command` StrEnum for zero-arg verbs plus `pick`/`drop`/`broadcast` builders,
  and a `CommandPipeline` that enforces the ≤10 outstanding-request window (RQ12).
- Responses pair FIFO with the oldest outstanding command; server pushes
  (`death`, `message <K>,<text>`, `moving <K>`) are routed out as `ServerEvent`s
  so a push can never desynchronise command/response pairing or underflow the
  window.
- `try_send` refuses the 11th in-flight request (returns `False`); `send` raises
  `PipelineFull`. A response frees a slot. No gameplay/AI logic yet — that is C03+.

## Files touched

- `client/zappy_client/pipeline.py` (new)
- `client/tests/test_pipeline.py` (new — 15 tests)
- `client/zappy_client/__init__.py` (docstring only)
- `docs/ticket-tracker.md` (C02 → ✅, Who's up, summaries)

## How to verify

```bash
cd client
python -m pytest -q          # 34 passed (incl. 15 new pipeline tests)
ruff check .                 # All checks passed!
ruff format --check .        # all files formatted
```

- AQ / RQ IDs checked:
  - **RQ11** — command strings match the subject table exactly
    (`requirements.md` §"The commands" / §"Client/server communication"):
    `advance`, `right`, `left`, `see`, `inventory`, `pick <object>`,
    `drop <object>`, `kick`, `broadcast <text>`, `enchantment`, `fork`,
    `connect_nbr`; each is `\n`-terminated on the wire. The client only emits
    valid verbs (e.g. `broadcast` rejects embedded newlines), so it never
    originates the unknown/malformed lines the server answers with `ko`.
  - **RQ12** — "up to 10 successful requests without a response … more than 10
    the server will no longer take them into account" (`requirements.md`
    §Client/server communication). `MAX_PENDING = 10`; the pipeline never sends
    an 11th in-flight request. (C02 has no AQ in its Coverage column.)

## Risks / follow-ups

- `enchantment` yields two server lines (`evolution in progress`, then
  `current level : K` after 300/t). The pipeline classifies both as responses;
  the first pairs with the `enchantment` command and the second arrives with an
  empty window → surfaced as `EventKind.UNSOLICITED`. C04 (enchantment attempts)
  should read the follow-up explicitly rather than treating it as a stray line.
- `see`/`inventory` response *parsing* is intentionally out of scope; C03 owns
  turning `{...}` payloads into game state.

## Next suggested ticket

- `C03` — Survival AI: move, see, pick food, avoid death (Deps C02 ✅, S10 ✅).
