# Architecture

The daemon is an independently deployable service. Frontends are clients and do
not evaluate Nix themselves.

The system distinguishes declared configuration, evaluated configuration,
built closures/generations, and active runtime state. Not every layer exists in
the initial read-only API; endpoints and UI labels must identify the layer they
represent rather than collapsing them into a generic state. In particular,
configured service enablement and live systemd status are independent.

- `api-types` owns stable, serializable wire models.
- `nix` contains narrow process and filesystem inspection adapters.
- `core` contains frontend-independent application services.
- `daemon` exposes those services through Axum.
- `api-client` is the reusable typed Rust client.
- `cli` is the first client and exercises the public API.
- `apps/ui` owns the shared Dioxus component tree. The daemon currently renders
  it server-side, keeping deployment to one executable while the interactive
  Web/WASM client is developed.
- `privileged` reserves a separate executable boundary for future mutations. It
  deliberately exposes no operations yet.

## Trust boundary

The current daemon is read-only and unauthenticated. It binds to loopback, and
the NixOS module rejects a non-loopback address. No privileged helper protocol
or configuration mutation is implemented. Remote access, authentication, and
write operations must be designed together rather than widening this initial
surface accidentally.

All external commands must be invoked with fixed executable names and arguments
constructed by typed adapters, and must have timeouts. API input must never be
passed to a shell.

See [engineering-decisions.md](engineering-decisions.md) for dependency and
integration decisions derived from the project research checklist.
