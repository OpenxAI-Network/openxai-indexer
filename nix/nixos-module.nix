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

      tokenownerkey = lib.mkOption {
        type = lib.types.str;
        example = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
        description = ''
          The private key of the tokenized server owner.
        '';
      };

      tokenminterkey = lib.mkOption {
        type = lib.types.str;
        example = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
        description = ''
          The private key of the tokenized server minter.
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

      subdomaindistributor = lib.mkOption {
        type = lib.types.str;
        default = "http://subdomain-distributor.local:42923";
        example = "http://subdomain-distributor.local:42923";
        description = ''
          Subdomain distributor to acquire tokenized server subdomains.
        '';
      };

      rpc = {
        http = lib.mkOption {
          type = lib.types.str;
          default = "https://base-rpc.publicnode.com";
          example = "https://base-sepolia-rpc.publicnode.com";
          description = ''
            Blockchain HTTP RPC to query to smart contract calls.
          '';
        };

        ws = lib.mkOption {
          type = lib.types.str;
          default = "wss://base-rpc.publicnode.com";
          example = "wss://base-sepolia-rpc.publicnode.com";
          description = ''
            Blockchain WebSocket RPC to subscribe to smart contract events.
          '';
        };
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
          default = "0x84599c907B42e9bc21F9FE26D9e5A5D3747109D3";
          example = "0x84599c907B42e9bc21F9FE26D9e5A5D3747109D3";
          description = ''
            OpenxAIGenesis contract address. 
          '';
        };

        ownaiv1 = lib.mkOption {
          type = lib.types.str;
          default = "0x1962d34E472E205Bf504Aa305A375c8895Eaf9b4";
          example = "0x1962d34E472E205Bf504Aa305A375c8895Eaf9b4";
          description = ''
            OpenxAITokenizedServerV1 contract address. 
          '';
        };

        deposit = lib.mkOption {
          type = lib.types.str;
          default = "0x1EdE9dE47e5E3B8941884e7f5DDa43D82570180D";
          example = "0x1EdE9dE47e5E3B8941884e7f5DDa43D82570180D";
          description = ''
            OpenxAICreditsDeposit contract address. 
          '';
        };

        usdc = lib.mkOption {
          type = lib.types.str;
          default = "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913";
          example = "0x036CbD53842c5426634e7929541eC2318f3dCF7e";
          description = ''
            USDC contract address. 
          '';
        };
      };

      ownaiv1price = lib.mkOption {
        type = lib.types.ints.unsigned;
        default = 100000000;
        example = 150000000;
        description = ''
          The cost in 6 decimals of renting a OwnAIV1 server for 1 month
        '';
      };

      hyperstackapikey = lib.mkOption {
        type = lib.types.str;
        example = "7a12411b-0074-4d01-a375-ca91376f0bb8";
        description = ''
          The api key to use for hyperstack deployments.
        '';
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
        TOKENOWNERKEY = cfg.tokenownerkey;
        TOKENMINTERKEY = cfg.tokenminterkey;
        DATABASE = cfg.database;
        SUBDOMAINDISTRIBUTOR = cfg.subdomaindistributor;
        HTTPRPC = cfg.rpc.http;
        WSRPC = cfg.rpc.ws;
        CHAINID = toString cfg.chainId;
        CLAIMER = cfg.contracts.claimer;
        GENESIS = cfg.contracts.genesis;
        OWNAIV1 = cfg.contracts.ownaiv1;
        DEPOSIT = cfg.contracts.deposit;
        USDC = cfg.contracts.usdc;
        OWNAIV1PRICE = toString cfg.ownaiv1price;
        HYPERSTACKAPIKEY = cfg.hyperstackapikey;
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
