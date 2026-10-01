# Oneshot unit. hybrid-node mines one PoW block or replays, then exits.
# A resident peer loop is not in the crate; do not set this to simple/notify.
{ config, lib, pkgs, ... }:
let
  cfg = config.services.orthal;
in {
  options.services.orthal = {
    enable = lib.mkEnableOption "Orthal hybrid-node (oneshot mine or replay)";

    package = lib.mkOption {
      type = lib.types.package;
      default = pkgs.callPackage ./package.nix { };
      defaultText = "pkgs.hybrid-node from the orthal overlay";
      description = "hybrid-node derivation. Header v5 snapshots only.";
    };

    dataFile = lib.mkOption {
      type = lib.types.str;
      default = "/var/lib/orthal/chain.bin";
      description = "Chain snapshot. Must live under StateDirectory.";
    };

    replay = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "Revalidate the snapshot and exit. Otherwise mine one PoW block.";
    };

    extraArgs = lib.mkOption {
      type = lib.types.listOf lib.types.str;
      default = [ ];
      description = "Extra argv. Do not put wallet seeds here; they land in the store-visible unit file.";
    };
  };

  config = lib.mkIf cfg.enable {
    assertions = [{
      assertion = lib.hasPrefix "/var/lib/orthal/" cfg.dataFile;
      message = "services.orthal.dataFile must be under /var/lib/orthal (StateDirectory).";
    }];

    systemd.services.orthal = {
      description = "Orthal hybrid-node";
      wantedBy = [ "multi-user.target" ];
      after = [ "network.target" ];
      serviceConfig = {
        Type = "oneshot";
        DynamicUser = true;
        StateDirectory = "orthal";
        StateDirectoryMode = "0700";
        UMask = "0077";
        ExecStart = lib.concatStringsSep " " ([
          "${cfg.package}/bin/hybrid-node"
          "--data"
          cfg.dataFile
        ] ++ lib.optionals cfg.replay [ "--replay" ] ++ cfg.extraArgs);

        NoNewPrivileges = true;
        PrivateTmp = true;
        PrivateDevices = true;
        PrivateUsers = true;
        ProtectSystem = "strict";
        ProtectHome = true;
        ProtectClock = true;
        ProtectKernelTunables = true;
        ProtectKernelModules = true;
        ProtectKernelLogs = true;
        ProtectControlGroups = true;
        ProtectProc = "invisible";
        ProtectHostname = true;
        RestrictSUIDSGID = true;
        LockPersonality = true;
        MemoryDenyWriteExecute = true;
        RestrictNamespaces = true;
        RestrictRealtime = true;
        RestrictAddressFamilies = [ "AF_UNIX" ];
        SystemCallArchitectures = "native";
        SystemCallFilter = [ "@system-service" "~@privileged" "~@resources" ];
        CapabilityBoundingSet = "";
        AmbientCapabilities = "";
      };
    };
  };
}
