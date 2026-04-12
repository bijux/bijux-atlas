---
title: Release Packets
audience: operators
type: guide
status: canonical
owner: atlas-docs
last_reviewed: 2026-04-13
---

# Release Packets

Release packets gather the governed inputs required to move Atlas through a
real release path.

## Purpose

Use this page to understand what a release packet contains, how it differs from
the full evidence bundle, and who consumes it.

## Source of Truth

- `ops/release/packet/packet.json`
- `ops/release/evidence/`
- `ops/release/notes/`

## Packet Structure

`ops/release/packet/packet.json` currently defines:

- the evidence root
- a list of packet items with paths and checksums
- the minimum required packet members
- the contracts the packet satisfies

The packet includes portable release artifacts such as manifests, ingest
artifacts, schemas, package tarballs, SBOMs, provenance, and signing outputs.

## Packet Versus Evidence Bundle

- the evidence bundle is the broader release evidence surface
- the packet is the selected transport set for downstream release consumers
- packet minimums ensure consumers always receive identity, manifest, bundle,
  checksums, signing outputs, and provenance

## Related Contracts and Assets

- `ops/release/packet/`
- `ops/release/notes/`
- `ops/release/packet/packet.json`
