# Linux-only NixOS VM test. Imported from the flake when pkgs.stdenv.isLinux.
{ self, pkgs }:
pkgs.nixosTest {
  name = "orthal-oneshot";
  nodes.machine = { ... }: {
    imports = [ self.nixosModules.orthal ];
    services.orthal.enable = true;
    services.orthal.package = self.packages.${pkgs.stdenv.hostPlatform.system}.hybrid-node;
  };
  testScript = ''
    machine.wait_for_unit("orthal.service")
    machine.succeed("test -s /var/lib/orthal/chain.bin")
  '';
}
