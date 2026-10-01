# Nix

Build and shell for `hybrid-chain`. Pin is `nixos-25.05`.

`Cargo.lock` is committed. Nix will not build without it. Do not gitignore it again.

## Shell

```bash
nix develop
cd hybrid-chain
cargo test --all-targets
```

## Package

```bash
nix build
./result/bin/hybrid-node --data /tmp/orthal/chain.bin
./result/bin/hybrid-node --data /tmp/orthal/chain.bin --replay
```

`nix build` runs `cargo test` (`doCheck = true`). Header v5 snapshots only.

## NixOS

```nix
{
  inputs.orthal.url = "github:5mil/orthal";
  outputs = { orthal, nixpkgs, ... }: {
    nixosConfigurations.host = nixpkgs.lib.nixosSystem {
      modules = [
        orthal.nixosModules.orthal
        ({ pkgs, ... }: {
          nixpkgs.overlays = [
            (_: _: { hybrid-node = orthal.packages.${pkgs.system}.default; })
          ];
          services.orthal.enable = true;
        })
      ];
    };
  };
}
```

The unit is `Type = oneshot`. The binary mines one block or replays and exits. A resident peer loop is not in the crate yet.

## What this does not do

- No flake lock until the first `nix flake lock` on a machine with Nix.
- No cross-compiled miner.
- No secret management. Wallet seeds stay out of the Nix store.
