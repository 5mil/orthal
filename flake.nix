{
  description = "Orthal hybrid-node";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";

  outputs = { self, nixpkgs }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      forAll = f: nixpkgs.lib.genAttrs systems (system:
        f (import nixpkgs { inherit system; overlays = [ self.overlays.default ]; }));
    in {
      overlays.default = import ./nix/overlay.nix;

      packages = forAll (pkgs: {
        hybrid-node = pkgs.hybrid-node;
        default = pkgs.hybrid-node;
      });

      apps = forAll (pkgs: {
        hybrid-node = {
          type = "app";
          program = "${pkgs.hybrid-node}/bin/hybrid-node";
        };
        default = self.apps.${pkgs.stdenv.hostPlatform.system}.hybrid-node;
      });

      checks = forAll (pkgs: {
        hybrid-node = pkgs.hybrid-node;
      } // nixpkgs.lib.optionalAttrs pkgs.stdenv.isLinux {
        nixos-oneshot = import ./nix/test.nix { inherit self pkgs; };
      });

      devShells = forAll (pkgs: {
        default = pkgs.mkShell {
          inputsFrom = [ pkgs.hybrid-node ];
          packages = with pkgs; [
            rustc
            cargo
            rustfmt
            clippy
            rust-analyzer
            pkg-config
            alejandra
          ];
          shellHook = ''
            echo "orthal: cd hybrid-chain && cargo test --all-targets"
            echo "        nix build   |   nix flake check   |   nix run"
          '';
        };
      });

      formatter = forAll (pkgs: pkgs.alejandra);

      nixosModules.orthal = ./nix/module.nix;
      nixosModules.default = self.nixosModules.orthal;
    };
}
