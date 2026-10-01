{
  description = "Orthal hybrid-node — Nix build and dev shell";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";

  outputs = { self, nixpkgs }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      forAll = f: nixpkgs.lib.genAttrs systems (system:
        f (import nixpkgs { inherit system; }));
    in {
      packages = forAll (pkgs: rec {
        hybrid-node = pkgs.rustPlatform.buildRustPackage {
          pname = "hybrid-node";
          version = "0.1.0";
          src = ./hybrid-chain;
          cargoLock.lockFile = ./hybrid-chain/Cargo.lock;
          doCheck = true;
          meta = {
            description = "Orthal hybrid PoW/PoS node";
            mainProgram = "hybrid-node";
          };
        };
        default = hybrid-node;
      });

      devShells = forAll (pkgs: {
        default = pkgs.mkShell {
          packages = with pkgs; [
            rustc
            cargo
            rustfmt
            clippy
            pkg-config
          ];
          shellHook = ''
            echo "orthal dev shell — crate is hybrid-chain/"
            echo "  cargo test --all-targets"
            echo "  nix build"
          '';
        };
      });

      nixosModules.orthal = import ./nix/module.nix;
      nixosModules.default = self.nixosModules.orthal;
    };
}
