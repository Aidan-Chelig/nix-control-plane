{
  config,
  lib,
  package,
  pkgs,
  ...
}:
let
  cfg = config.services.nixos-control;
in
{
  options.services.nixos-control = {
    enable = lib.mkEnableOption "the NixOS control plane daemon";

    package = lib.mkOption {
      type = lib.types.package;
      default = package;
      defaultText = lib.literalExpression "inputs.nixos-control.packages.${pkgs.system}.default";
      description = "Package containing the nixos-control-daemon executable.";
    };

    listenAddress = lib.mkOption {
      type = lib.types.str;
      default = "127.0.0.1";
      description = "Address on which the HTTP API listens. Keep this on loopback until authentication is configured.";
    };

    port = lib.mkOption {
      type = lib.types.port;
      default = 9834;
      description = "TCP port on which the HTTP API listens.";
    };

    stateDirectory = lib.mkOption {
      type = lib.types.str;
      default = "nixos-control";
      description = "Directory below /var/lib used for persistent daemon state.";
    };

    commandTimeoutSeconds = lib.mkOption {
      type = lib.types.ints.positive;
      default = 30;
      description = "Timeout for commands used to inspect the local system.";
    };

    configurationPath = lib.mkOption {
      type = lib.types.path;
      default = "/etc/nixos";
      description = "Local NixOS configuration directory or file to inspect.";
    };

    flakeUri = lib.mkOption {
      type = lib.types.nullOr lib.types.str;
      default = null;
      description = "Explicit flake URI to inspect instead of configurationPath.";
    };

    host = lib.mkOption {
      type = lib.types.nullOr lib.types.str;
      default = null;
      description = "NixOS configuration host name within the selected flake.";
    };
  };

  config = lib.mkIf cfg.enable {
    assertions = [
      {
        assertion = lib.elem cfg.listenAddress [
          "127.0.0.1"
          "::1"
          "localhost"
        ];
        message = "nixos-control has no authentication yet and must listen on loopback";
      }
    ];

    systemd.services.nixos-control = {
      description = "NixOS Control Plane";
      wantedBy = [ "multi-user.target" ];
      after = [ "network.target" ];
      path = [
        pkgs.nix
        pkgs.systemd
      ];
      serviceConfig = {
        ExecStart = lib.getExe' cfg.package "nixos-control-daemon";
        Restart = "on-failure";
        DynamicUser = true;
        StateDirectory = cfg.stateDirectory;
        ProtectSystem = "strict";
        ProtectHome = true;
        PrivateTmp = true;
        NoNewPrivileges = true;
        RestrictAddressFamilies = [
          "AF_INET"
          "AF_INET6"
          "AF_UNIX"
        ];
      };
      environment = {
        NIXOS_CONTROL_LISTEN = "${cfg.listenAddress}:${toString cfg.port}";
        NIXOS_CONTROL_COMMAND_TIMEOUT = toString cfg.commandTimeoutSeconds;
        NIXOS_CONTROL_STATE_DIR = "/var/lib/${cfg.stateDirectory}";
      }
      // lib.optionalAttrs (cfg.host != null) {
        NIXOS_CONTROL_HOST = cfg.host;
      }
      // lib.optionalAttrs (cfg.flakeUri != null) {
        NIXOS_CONTROL_FLAKE_URI = cfg.flakeUri;
      }
      // lib.optionalAttrs (cfg.flakeUri == null) {
        NIXOS_CONTROL_CONFIGURATION_PATH = toString cfg.configurationPath;
      };
    };
  };
}
