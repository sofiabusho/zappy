# Handoff note — `A04` `readme-quickstart`

> Short turn summary. Not a PR.

## Summary

- Expanded root `README.md` with prerequisites, build/check, run stubs, audit quickstart, and localhost-only guidance.
- Documented RQ16 server constraints (Rust, multiplexed TCP, no `exec*`, never hang forever).
- Added strong `siege` warning matching the subject (own server only).
- Added `scripts/test_readme_quickstart.py`; wired into `scripts/check.sh`.

## Files touched

- `README.md`
- `scripts/test_readme_quickstart.py`, `scripts/check.sh`, `scripts/README.md`
- `docs/ticket-tracker.md`
- `docs/handoff-notes/A04-readme-quickstart.md`

## How to verify

```bash
python3 scripts/test_readme_quickstart.py
./scripts/check.sh
```

- AQ / RQ IDs checked:
  - **RQ16** — README documents Rust server, multiplexed TCP, no `exec*`, never-hang / availability, localhost-only (aligned with `docs/raw/requirements.md` server section). Full runtime enforcement remains S02/S16.

## Risks / follow-ups

- Audit command examples that need a live TCP server still wait on S-track.
- Sprint 0 complete; next is **S01**.

## Next suggested ticket

- `S01` — CLI parse `-p -x -y -n -c -t`; usage on bad/missing args; default t=100
