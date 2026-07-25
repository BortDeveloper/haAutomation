# Documentation Index

> **Status**: active
> **Owner**: ha-automation project
> **Language**: mixed (top-level docs English, deep-dive runbooks German)
> **Convention**: Cockpit doku-standard (three-audience navigation)

This is the map of all documentation in this repository. New readers should
start at the [top-level README](../README.md) and its "Where to start"
navigation. This index groups every document by audience and task.

## By audience

| Audience | Start here |
|---|---|
| Technically interested | [README](../README.md), [Requirements](requirements.md) |
| Operators | [Runbooks](#operating-runbooks) below |
| Developers | [Architecture](architecture.md), [ADRs](decisions/README.md) |

## Setup and first run

- [Getting started](getting-started.md) — first-run PoC on a home network (security-audited walkthrough).
- [Raspberry Pi install](runbooks/raspberry-pi-install.md) — install the HA-automation suite on a Pi and run the HA sync.
- [Inventory build host](runbooks/inventory-build-host.md) — set up the Linux build host for the inventory backend.
- [VPS setup](vps-setup.md) — public VPS bootstrap: SSH/keys, ops, disaster recovery.

## Operating (runbooks)

- [Edge-secret backup and restore](runbooks/edge-secret-backup.md) — sops/age escrow plus restore drill (Cockpit ADR-0004).
- [Operator checklist ADR-0004](runbooks/operator-checklist-adr-0004.md) — generate and register age recipients.
- [Coordinator replacement](runbooks/coordinator-replacement.md) — replace or re-provision the Zigbee coordinator.
- [Node-RED flow update](runbooks/nodered-flow-update.md) — change a Node-RED flow in a controlled way.
- [Node-RED flow analysis with Claude](runbooks/nodered-flow-analysis-with-claude.md) — analyse flows with Claude.

## Architecture and decisions

- [Architecture](architecture.md) — components, data flow, trust boundaries, ownership rules, secrets architecture.
- [Requirements](requirements.md) — what is being built and why.
- [Roadmap](roadmap.md) — step plan S1–S14.
- [ADR index](decisions/README.md) — architecture decision records with rationale.

## Testing and hardware

- [Real-hardware test environment](test-umgebung-real-hardware.md) — test setup against physical devices.

## Cockpit integration

- [Cockpit interlock](cockpit/README.md) — how this project ties into the stack-master cockpit (phases, KPIs, audits).
