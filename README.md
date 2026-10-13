# NixOS Control Plane

An early-stage, self-hosted control plane for inspecting and eventually managing
NixOS machines. Nix configuration remains the source of truth. The daemon is
headless and clients communicate through a versioned HTTP/JSON API.

## Development

Enter the repository's development environment before invoking Rust tools:

```console
nix develop
cargo test --workspace
cargo run -p nixos-control-daemon
```

In another terminal:

```console
nix develop --command cargo run -p nixos-control -- health
nix develop --command cargo run -p nixos-control -- system
nix develop --command cargo run -p nixos-control -- configuration
```

The web dashboard and API listen on `127.0.0.1:9834` by default. Open
`http://127.0.0.1:9834/` after starting the daemon. The initial API surface is:

- `GET /health`
- `GET /api` (version and capability discovery)
- `GET /api/v1/system/summary`
- `GET /api/v1/generations`
- `GET /api/v1/generations/{generation}`
- `GET /api/v1/generations/{from}/compare/{to}`
- `GET /api/v1/configuration`
- `GET /api/v1/configuration/managed`
- `GET /api/v1/flake/inputs`
- `GET /api/v1/repository/status`
- `GET /api/v1/options`
- `GET /api/v1/options/{option}`
- `GET /api/v1/packages`
- `GET /api/v1/services`
- `GET /api/v1/services/{service}`
- `GET /api/v1/runtime/services`

The overview reports the active NixOS generation, kernel, uptime, logical CPU
count and load averages, memory availability, and root filesystem capacity.
The Generations page lists installed system profile generations newest-first and
distinguishes the running system, current profile, and bootable closures. Each
generation has a lazy detail resource reporting its recursive closure size and
store-path count. Generation comparisons report exact added and removed store
paths and byte totals without mislabeling closure entries as installed packages.
Immutable detail and comparison results are cached in memory.

The daemon discovers `/etc/nixos/flake.nix` or `configuration.nix` by default.
Use `--configuration-path`, or select a flake with `--flake-uri` and `--host`.
The dashboard evaluates option names, descriptions, types, and declaration
locations through a timeout-bound `nix eval` process. Evaluation errors are
shown without taking down the system overview.

Opening an option detail evaluates that option lazily and displays its effective
value, type, default, example, read-only status, and declaration sources. Values
that cannot be represented safely as JSON remain unavailable rather than being
forced or approximated from source syntax.

The Inputs page reads the selected local `flake.lock` and distinguishes direct
inputs from transitive nodes while displaying pinned origins, revisions, content
hashes, and timestamps. It never updates or rewrites the lock file.
The same page reports the selected repository's Git branch, commit, upstream
divergence, conflicts, and dirty-file counts without fetching or changing Git
state.

Expensive option, package, configured-service, and service-detail evaluations
are cached in memory. Local flake targets passed with either
`--configuration-path` or `--flake-uri` are fingerprinted recursively, excluding
build and VCS cache directories, so navigation reuses results until the selected
configuration changes. Runtime systemd and host-health data remain live.

The packages view reports the evaluated `environment.systemPackages` list. It
does not mislabel transitive Nix store closure entries as explicitly installed
packages. Package source inputs are derived from evaluated Nix metadata and can
be selected with the dashboard's input tags or the API's `source` query
parameter (for example, `/api/v1/packages?source=nixpkgs-unstable`).

The services view lists only services whose evaluated enable option is true.
Runtime systemd state is queried independently so declarative and active state
remain distinct. Each configured service links to a detail page backed by its
own API resource, exposing the option namespace and declaration metadata.
Current values are evaluated only for the selected service subtree and are
returned as JSON text; values that cannot be safely evaluated remain unavailable.
Associated systemd units are derived by matching Nix option declaration files
to `systemd.services` definition provenance rather than guessing from names.

## Managed configuration ownership

Control-plane-owned configuration is confined to `ncp/` at the selected
repository root. Per-host modules use `ncp/hosts/<host>.nix`. Existing
repositories keep manual control of their imports: wire the selected host with
an import such as `./ncp/hosts/portim.nix`; the control plane will not rewrite
unmanaged files.

An `ncp/metadata.json` file with `"owner":"nixos-control-plane"` establishes
the ownership boundary. Set `"repository_managed":false` when imports are wired
manually, or `true` only when the repository and its wiring were created by the
control plane. The ownership API reports this mode and the expected host
module. Definition ownership is granted only to existing files whose canonical
paths remain inside `ncp/`; all other definitions remain read-only.

## NixOS module

Import `nixosModules.default`, set `services.nixos-control.enable = true`, and
set `services.nixos-control.package` to the flake package when it is not already
available from your package set. Until authentication lands, the module rejects
non-loopback listen addresses.

See [docs/architecture.md](docs/architecture.md) for component boundaries and
current security constraints, and
[docs/engineering-decisions.md](docs/engineering-decisions.md) for decisions
adopted from the engineering research inventory.
