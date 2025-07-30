{
  config,
  pkgs,
  lib,
  ...
}:
let
  cfg = config.services.openxai-indexer;
  openxai-indexer = pkgs.callPackage ./package.nix { };
in
{
  options = {
    services.openxai-indexer = {
      enable = lib.mkEnableOption "Enable the rust app";

      hostname = lib.mkOption {
        type = lib.types.str;
        default = "0.0.0.0";
        example = "127.0.0.1";
        description = ''
          The hostname under which the app should be accessible.
        '';
      };

      port = lib.mkOption {
        type = lib.types.port;
        default = 36092;
        example = 36092;
        description = ''
          The port under which the app should be accessible.
        '';
      };

      verbosity = lib.mkOption {
        type = lib.types.str;
        default = "warn";
        example = "info";
        description = ''
          The logging verbosity that the app should use.
        '';
      };

      claimerkey = lib.mkOption {
        type = lib.types.str;
        example = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
        description = ''
          The private key of the claim signer.
        '';
      };

      database = lib.mkOption {
        type = lib.types.str;
        default = "postgres:openxai-indexer?host=/run/postgresql";
        example = "postgres:openxai-indexer?host=/run/postgresql";
        description = ''
          Connection string to access the postgres database.
        '';
      };

      rpc = lib.mkOption {
        type = lib.types.str;
        default = "wss://base-rpc.publicnode.com";
        example = "wss://base-sepolia-rpc.publicnode.com";
        description = ''
          Blockchain RPC to subscribe to smart contract events.
        '';
      };

      chainId = lib.mkOption {
        type = lib.types.ints.unsigned;
        default = 8453;
        example = 84532;
        description = ''
          The port under which the app should be accessible.
        '';
      };

      postgres = {
        enable = lib.mkOption {
          type = lib.types.bool;
          default = true;
          example = false;
          description = ''
            Enable the default postgres config.
          '';
        };
      };

      contracts = {
        claimer = lib.mkOption {
          type = lib.types.str;
          default = "0xc749169dB9C231E1797Aa9cD7f5B7a88AeD25b08";
          example = "0xc749169dB9C231E1797Aa9cD7f5B7a88AeD25b08";
          description = ''
            OpenxAIClaimer contract address. 
          '';
        };

        genesis = lib.mkOption {
          type = lib.types.str;
          default = "0xC950A7c6D892ba39df564f263c830eB2A6E600e1";
          example = "0xC950A7c6D892ba39df564f263c830eB2A6E600e1";
          description = ''
            OpenxAIGenesis contract address. 
          '';
        };
      };

      openFirewall = lib.mkOption {
        type = lib.types.bool;
        default = true;
        description = ''
          Whether to open ports in the firewall for this application.
        '';
      };
    };
  };

  config = lib.mkIf cfg.enable {
    users.groups.openxai-indexer = { };
    users.users.openxai-indexer = {
      isSystemUser = true;
      group = "openxai-indexer";
    };

    systemd.services.openxai-indexer = {
      wantedBy = [ "multi-user.target" ];
      description = "Indexes OpenxAI smart contract events for efficient retrieval.";
      after = [
        "network.target"
        "postgresql.target"
      ];
      environment = {
        HOSTNAME = cfg.hostname;
        PORT = toString cfg.port;
        RUST_LOG = cfg.verbosity;
        CLAIMERKEY = cfg.claimerkey;
        DATABASE = cfg.database;
        RPC = cfg.rpc;
        CHAINID = toString cfg.chainId;
        CLAIMER = cfg.contracts.claimer;
        GENESIS = cfg.contracts.genesis;
      };
      serviceConfig = {
        ExecStart = "${lib.getExe openxai-indexer}";
        User = "openxai-indexer";
        Group = "openxai-indexer";
        StateDirectory = "openxai-indexer";
        Restart = "on-failure";
      };
    };

    services.postgresql = lib.mkIf cfg.postgres.enable {
      enable = true;
      ensureDatabases = [ "openxai-indexer" ];
      ensureUsers = [
        {
          name = "openxai-indexer";
          ensureDBOwnership = true;
        }
      ];
      authentication = pkgs.lib.mkOverride 10 ''
        #type database  DBuser  auth-method
        local sameuser  all     peer
      '';
    };

    networking.firewall = lib.mkIf cfg.openFirewall {
      allowedTCPPorts = [ cfg.port ];
    };
  };
}
