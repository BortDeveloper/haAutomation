# Node-RED Integration (work in progress)

> **Branch-isoliert**: This work-in-progress branch (`feat/nodered-integration`)
> does NOT affect the `main` branch. The Heimnetz PoC walkthrough
> (`docs/getting-started.md` on `main`) continues to work unchanged.

## Architectural decision

This integration follows a **two-actor separation**:

- **Analysis & change proposals**: hosted in the stack-master cockpit
  (a separate orchestration repo). Multi-expert review of Node-RED
  flow analytics happens there. Proposals are emitted as structured
  data (schema defined in stack-master ADR-0011).
- **Application of flow changes**: only this repo's developer agent
  (`ha-automation-dev`) applies changes on this branch, after
  receiving a proposal from the cockpit.

The rationale: production-affecting flow mutations must not be
triggered directly by the cockpit. The cockpit observes, suggests,
and reviews; this branch enacts.

## Scope (this branch)

- **In scope**:
  - Sync module `inventory/src/sync/nodered.rs` — pull flow snapshots
    (read-only) for analysis ingestion
  - Schema additions in `inventory/src/types.rs` for Flow objects
    (separate from Device — no Multi-Source dedup conflict)
  - DB-Migration for `flows` table
  - Stub for proposal-application logic (deferred until ADR-0011
    finalizes proposal schema)
- **NOT in scope**:
  - Direct Node-RED API write operations (no flow mutation from
    this code without cockpit-proposal review)
  - Anything that touches `docs/getting-started.md` PoC steps
  - Sync-Pipeline impact (NR sync is opt-in via separate
    `sync nodered` subcommand)

## How this stays isolated from main

- New module + subcommand only (no edits to existing `sync/{ha,ccu,hue,shelly}`)
- New DB-Table `flows` (no schema change to existing `devices`)
- New CLI subcommand `inventory sync nodered` (gated, opt-in)
- New types in dedicated submodule (`types::flow::Flow`)

## Status

This branch is under active development. Do not merge to `main` until:
1. ADR-0011 (cockpit flow-analysis service) is `ACCEPTED`
2. Proposal-application interface is implemented and tested
3. User has run an end-to-end roundtrip (cockpit suggest → dev apply)
   on a non-production Node-RED instance
