# Engineering decisions

This document records decisions prompted by `nixos-control-considerations.txt`.
The considerations file is a research inventory, not a dependency list. New
dependencies are added only when a concrete feature needs them and after their
license, maintenance status, and supported Nix versions have been checked.

## Current state model

The API and UI must name the state they report instead of presenting one value
as generic "system state":

| State | Meaning | Current source |
| --- | --- | --- |
| Declared | Definitions and their source locations | NixOS option metadata |
| Evaluated | Values after module merging | `nix eval` of the selected host |
| Built | Candidate or generation closure | Not implemented yet |
| Active | State of the running machine | `/run/current-system`, `/proc`, and systemd |

Declarative service enablement and active systemd unit state therefore remain
separate API resources. A successful future build must not imply successful
activation or healthy runtime state.

## Adopted now

- Use the NixOS module evaluator through timeout-bound `nix eval` calls and
  project deliberately limited JSON. Syntax parsing is not used to infer final
  values.
- Keep external processes in the `nix` adapter crate. Arguments are passed
  directly to executables without a shell, failures are structured, and child
  processes are killed when timed-out futures are dropped.
- Cache expensive evaluated configuration by the selected host and source-tree
  fingerprint. Continue querying runtime health and systemd state live.
- Keep the Axum daemon, shared wire types, Rust client, and Dioxus UI separate.
- Reserve `ncp/` for owned configuration and treat all other source as
  read-only. Provenance and ownership are prerequisites for future mutation.
- Prefer structured Nix and systemd output where available. Parsing unstable
  internal build-event formats will require compatibility fixtures before use.

## Adopt when its feature starts

- `rnix` and `nixfmt` for opt-in, comment-preserving source edits and generated
  managed modules. `rnix` will never substitute for module evaluation.
- `nix search --json` for a catalog tied to the selected nixpkgs revision.
- `nvd` or `dix` behind an adapter for generation and closure comparisons.
- systemd D-Bus through `zbus` for operations and event-driven status; use
  cursor-based journal access for logs.
- `utoipa` for an OpenAPI document when mutation/job endpoints settle enough to
  define the durable public contract.
- `notify` as a latency optimization for invalidation, with fingerprint
  revalidation retained for correctness.
- SQLite for sessions, jobs, audit records, and cache metadata only. Nix files
  remain authoritative configuration.
- OIDC, server-side sessions, and role enforcement as one security slice before
  any remote or mutating endpoint is exposed.

## Deferred intentionally

- Direct `libnixstore` FFI: the compatibility and unsafe-code burden conflicts
  with the workspace's `unsafe_code = "forbid"` policy. CLI adapters remain the
  initial boundary.
- A custom scheduler, logger, service manager, or dependency-graph engine:
  systemd and Nix already own those concerns.
- Automatic edits or fixes from `statix` or `deadnix`: diagnostics may be
  exposed later, but unmanaged files will not be rewritten.
- Remote administration before authentication, TLS deployment guidance,
  authorization, and audit logging exist together.
