{ lib, pkgs, config, ... }:
with lib;
let
  # Shorter name to access final settings a 
  # user of module HAS ACTUALLY SET.
  # cfg is a typical convention.
  cfg = config.services.lysand.ap;

  # unused when the entrypoint is flake
  flake = import ../flake-compat.nix;
  overlay = flake.defaultNix.overlays.default;

  hasLocalPostgresDB =
    let
      url = cfg.database.url or "";
      localStrings = [ "localhost" "127.0.0.1" "/run/postgresql" ];
      hasLocalStrings = lib.any (lib.flip lib.hasInfix url) localStrings;
    in
    config.services.postgresql.enable && lib.hasPrefix "postgresql://" url && hasLocalStrings;

  # Settings necessary for running with an automatically managed local database
  localDatabaseConfig = lib.mkIf cfg.database.createLocally {
    assertions = [
      {
        assertion = cfg.database.user == cfg.database.dbname;
        message = ''
          For local automatic database provisioning (services.lysand.ap.database.createLocally == true)
          to  work, the username used to connect to PostgreSQL must match the database name, that is
          services.lysand.ap.database.user must match services.lysand.ap.database.dbname.
          This is the default since NixOS 24.05. For older systems, it is normally safe to manually set
          the user to "lysandap" as the new user will be created with permissions
          for the existing database. `REASSIGN OWNED BY kemal TO lysandap;` may also be needed, it can be
          run as `sudo -u postgres env psql --user=postgres --dbname=lysandap -c 'reassign OWNED BY kemal to lysandap;'`.
        '';
      }
    ];
    # Default to using the local database if we create it
    services.lysand.ap.database.host = lib.mkDefault null;

    services.postgresql = {
      enable = true;
      ensureUsers = lib.singleton { name = cfg.database.user; ensureDBOwnership = true; };
      ensureDatabases = lib.singleton cfg.database.dbname;
    };
  };
  dbconfone = lib.mkIf cfg.database.createLocally {
    systemd.services.lysandap.Environment = {
      DATABASE_URL = "postgresql:///${cfg.database.user}@localhost/${cfg.database.dbname}";
      "PORT" = "${toString cfg.port}";
      "ADDRESS" = "${cfg.address}:${toString cfg.port}";
      "FEDERATED_DOMAIN" = cfg.domain;
      "SERVICE_SCALE" = toString cfg.serviceScale;
      "LOCAL_USER_NAME" = "example";
    };
  };
  dbconftwo = lib.mkIf (cfg.database.createLocally == false) {
    systemd.services.lysandap.Environment = {
      DATABASE_URL = "postgresql://${cfg.database.user}:${cfg.database.passwordFile}@${cfg.database.host}:${toString cfg.database.port}/${cfg.database.dbname}";
      "PORT" = "${toString cfg.port}";
      "ADDRESS" = "${cfg.address}:${toString cfg.port}";
      "FEDERATED_DOMAIN" = cfg.domain;
      "SERVICE_SCALE" = toString cfg.serviceScale;
      "LOCAL_USER_NAME" = "example";
    };
  };
  nginxConfig = lib.mkIf cfg.nginx.enable {
    services.nginx =
      let
        ip = if cfg.address == "0.0.0.0" then "127.0.0.1" else cfg.address;
      in
      {
        enable = true;
        virtualHosts.${cfg.domain} = {
          locations."/".proxyPass =
            if cfg.serviceScale == 1 then
              "http://${ip}:${toString cfg.port}"
            else "http://upstream-invidious";

          enableACME = lib.mkDefault true;
          forceSSL = lib.mkDefault true;
        };
        upstreams = lib.mkIf (cfg.serviceScale > 1) {
          "upstream-invidious".servers = builtins.listToAttrs (builtins.genList
            (scaleIndex: {
              name = "${ip}:${toString (cfg.port + scaleIndex)}";
              value = { };
            })
            cfg.serviceScale);
        };
      };

    assertions = [{
      assertion = cfg.domain != null;
      message = "To use services.lysand.ap.nginx, you need to set services.lysand.ap.domain";
    }];
  };
in
{
  # Declare what settings a user of this "hello.nix" module CAN SET.
  options.services.lysand.ap = {
    enable = mkEnableOption "Whenever to enable Lysands Activitypub layer";
    package = lib.mkOption {
      description = ''
        The package to use.
      '';
      type = types.package;
      default = pkgs.lysand-ap-layer;
    };
    mig-package = lib.mkOption {
      description = ''
        The migration package to use.
      '';
      type = types.package;
      default = pkgs.ls-ap-migration;
    };
    user = lib.mkOption {
      description = ''
        The group under which lysand AP layer runs.
      '';
      type = types.str;
      default = "lysandap";
    };
    group = lib.mkOption {
      description = ''
        The user under which lysand AP layer runs.
      '';
      type = types.str;
      default = "lysandap";
    };
    port = lib.mkOption {
      type = types.port;
      default = 3000;
      description = ''
        The port Lysand AP layer should listen on.

        To allow access from outside,
        you can use either {option}`services.lysand.ap.nginx`
        or add `config.services.lysand.ap.port` to {option}`networking.firewall.allowedTCPPorts`.
      '';
    };
    address = lib.mkOption {
      type = types.str;
      default = if cfg.nginx.enable then "127.0.0.1" else "0.0.0.0";
      defaultText = lib.literalExpression ''if config.services.lysand.ap.nginx.enable then "127.0.0.1" else "0.0.0.0"'';
      description = ''
        The IP address Lysand AP layer should bind to.
      '';
    };
    domain = lib.mkOption {
      type = types.nullOr types.str;
      default = null;
      description = ''
        The FQDN Lysand AP layer is reachable on.

        This is used to configure nginx and for federation.
      '';
    };
    nginx.enable = lib.mkOption {
      type = types.bool;
      default = false;
      description = ''
        Whether to configure nginx as a reverse proxy for Lysand AP layer.

        It serves it under the domain specified in {option}`services.lysand.ap.domain` with enabled TLS and ACME.
        Further configuration can be done through {option}`services.nginx.virtualHosts.''${config.services.lysand.ap.domain}.*`,
        which can also be used to disable AMCE and TLS (will break federation).
      '';
    };
    serviceScale = lib.mkOption {
      type = types.int;
      default = 1;
      description = ''
        How many lysand ap instances to run.

        See https://docs.invidious.io/improve-public-instance/#2-multiple-invidious-processes for more details
        on how this is intended to work. All instances beyond the first one have the options `channel_threads`
        and `feed_threads` set to 0 to avoid conflicts with multiple instances refreshing subscriptions. Instances
        will be configured to bind to consecutive ports starting with {option}`services.invidious.port` for the
        first instance.
      '';
    };
    database = {
      createLocally = lib.mkOption {
        type = types.bool;
        default = true;
        description = ''
          Whether to create a local database with PostgreSQL.
        '';
      };

      host = lib.mkOption {
        type = types.nullOr types.str;
        default = null;
        description = ''
          The database host Lysand AP layer should use.

          If `null`, the local unix socket is used. Otherwise
          TCP is used.
        '';
      };

      port = lib.mkOption {
        type = types.port;
        default = config.services.postgresql.settings.port;
        defaultText = lib.literalExpression "config.services.postgresql.settings.port";
        description = ''
          The port of the database Lysand AP layer should use.

          Defaults to the the default postgresql port.
        '';
      };

      passwordFile = lib.mkOption {
        type = types.nullOr types.str;
        apply = lib.mapNullable toString;
        default = null;
        description = ''
          Path to file containing the database password.
        '';
      };

      user = lib.mkOption {
        type = types.str;
        default = "lysandap";
        description = ''
          The database user Lysand AP layer should use.
        '';
      };

      dbname = lib.mkOption {
        type = types.str;
        default = "lysandap";
        description = ''
          The database name Lysand AP layer should use.
        '';
      };
    };
  };

  # Define what other settings, services and resources should be active IF
  # a user of this "hello.nix" module ENABLED this module 
  # by setting "services.hello.enable = true;".
  config = mkIf cfg.enable (lib.mkMerge [
    localDatabaseConfig
    nginxConfig
    dbconfone
    dbconftwo
    {
      systemd.services.lysandap = {
        wantedBy = [ "multi-user.target" ];
        #wants = [ "network-online.target" ];
        #after = [ "network-online.target" ] ++ lib.optional cfg.database.createLocally "postgresql.service";
        #requires = lib.optional cfg.database.createLocally "postgresql.service";
        script = "${cfg.package}/bin/lysandap";
        preStart = "${cfg.mig-package}/bin/ls-ap-migration up";
        Environment = lib.mkDefault {
            "PORT" = "${toString cfg.port}";
            "ADDRESS" = "${cfg.address}:${toString cfg.port}";
            "FEDERATED_DOMAIN" = cfg.domain;
            "SERVICE_SCALE" = toString cfg.serviceScale;
            "LOCAL_USER_NAME" = "example";
        };
        serviceConfig = {
          DynamicUser = true;
          User = lib.mkIf (cfg.database.createLocally || cfg.serviceScale > 1) "lysandap";
          StateDirectory = "lysandap";
          StateDirectoryMode = "0750";
          WorkingDirectory = cfg.package;
          Restart = lib.mkDefault "always";
        };
      };
    }
  ]);
}
