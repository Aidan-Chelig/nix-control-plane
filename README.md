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
- `GET /api/v1/configuration`
- `GET /api/v1/options`
- `GET /api/v1/packages`
- `GET /api/v1/services`

The daemon discovers `/etc/nixos/flake.nix` or `configuration.nix` by default.
Use `--configuration-path`, or select a flake with `--flake-uri` and `--host`.
The dashboard evaluates option names, descriptions, types, and declaration
locations through a timeout-bound `nix eval` process. Evaluation errors are
shown without taking down the system overview.

The packages view reports the evaluated `environment.systemPackages` list. It
does not mislabel transitive Nix store closure entries as explicitly installed
packages.

## NixOS module

Import `nixosModules.default`, set `services.nixos-control.enable = true`, and
set `services.nixos-control.package` to the flake package when it is not already
available from your package set. Until authentication lands, the module rejects
non-loopback listen addresses.

See [docs/architecture.md](docs/architecture.md) for component boundaries and
current security constraints.
