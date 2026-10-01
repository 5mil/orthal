# Nix

Reproducible build, dev shell, overlay, and NixOS unit for `hybrid-chain`.
Pin: `github:NixOS/nixpkgs/nixos-25.05`.

`hybrid-chain/Cargo.lock` is part of the pin. Do not gitignore it.

First machine with Nix should run `nix flake lock` and commit `flake.lock`
so every later build uses the same nixpkgs revision.

## Layout

| Path | Role |
| --- | --- |
| `flake.nix` | packages, apps, checks, devShell, formatter, module |
| `nix/package.nix` | `buildRustPackage`, `doCheck = true` |
| `nix/overlay.nix` | `pkgs.hybrid-node` |
| `nix/module.nix` | hardened oneshot systemd unit |
| `nix/test.nix` | NixOS VM check (Linux only) |
| `.github/workflows/nix.yml` | `nix build` and `nix flake check` |

## Commands

```bash
nix develop
nix build                  # ./result/bin/hybrid-node
nix run -- --data /tmp/orthal/chain.bin
nix flake check            # package tests + NixOS oneshot on Linux
nix fmt
```

Inside the shell:

```bash
cd hybrid-chain
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
```

Header v5 snapshots only. A pre-ASERT `chain.bin` will not load.

## NixOS

```nix
{
  inputs.orthal.url = "github:5mil/orthal";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";

  outputs = { self, orthal, nixpkgs }: {
    nixosConfigurations.host = nixpkgs.lib.nixosSystem {
      modules = [
        orthal.nixosModules.orthal
        {
          services.orthal.enable = true;
          # optional: services.orthal.replay = true;
        }
      ];
    };
  };
}
```

The unit is `Type = oneshot` and `DynamicUser`. It mines one block or
replays, then exits. `dataFile` must stay under `/var/lib/orthal`.
Do not pass a wallet seed in `extraArgs`; unit files are world-readable
on many systems.

Hardening: no new privileges, strict system, no home, no network
(`AF_UNIX` only — the binary does not dial peers), syscall filter,
memory not executable. Drop the address-family restriction when a peer
loop exists.

## What is still out of scope

- No committed `flake.lock` until the first `nix flake lock`.
- No long-running peer process, so no socket activation and no open port.
- Wallet seeds are not a Nix secret. Keep them off the store.
- PoS mint is not a service path.
