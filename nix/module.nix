# NixOS module. Current binary mines one block and exits, or replays.
# Oneshot until a long-running peer loop exists.
{ config, lib, pkgs, ... }:
let
  cfg = config.services.orthal;
in {
  options.services.orthal = {
    enable = lib.mkEnableOption "Orthal hybrid-node";
    package = lib.mkPackageOption pkgs "hybrid-node" { };
    dataFile = lib.mkOption {
      type = lib.types.str;
      default = "/var/lib/orthal/chain.bin";
      description = "Isolated chain snapshot. Header v5 files only.";
    };
    replay = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "Revalidate and exit instead of mining one PoW block.";
    };
  };

  config = lib.mkIf cfg.enable {
    systemd.services.orthal = {
      description = "Orthal hybrid-node";
      wantedBy = [ "multi-user.target" ];
      serviceConfig = {
        Type = "oneshot";
        DynamicUser = true;
        StateDirectory = "orthal";
        ExecStart = "${cfg.package}/bin/hybrid-node --data ${cfg.dataFile} ${lib.optionalString cfg.replay "--replay"}";
      };
    };
  };
}
