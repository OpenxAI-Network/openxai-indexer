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
      enable = lib.mkEnableOption "Enable the node.js app";

      port = lib.mkOption {
        type = lib.types.port;
        default = 3001;
        example = 3001;
        description = ''
          The port under which the app should be accessible.
        '';
      };

      basePath = lib.mkOption {
        type = lib.types.str;
        default = "/";
        example = "/indexer/";
        description = ''
          Path prefix to use for API routes.
        '';
      };

      dataDir = lib.mkOption {
        type = lib.types.path;
        default = "/var/lib/openxai-indexer";
        example = "/var/lib/openxai-indexer";
        description = ''
          The main directory to store data.
        '';
      };

      infuraApiKey = lib.mkOption {
        type = lib.types.str;
        default = "";
        example = "<YOUR-API-KEY>";
        description = ''
          Infura API key to use for RPC calls.
        '';
      };
    };
  };

  config = lib.mkIf cfg.enable {
    systemd.services.openxai-indexer = {
      wantedBy = [ "multi-user.target" ];
      description = "Node.js App.";
      after = [ "network.target" ];
      environment = {
        PORT = toString cfg.port;
        BASEPATH = cfg.basePath;
        DATADIR = cfg.dataDir;
        INFURA_API_KEY = cfg.infuraApiKey;
      };
      serviceConfig = {
        ExecStart = "${lib.getExe openxai-indexer}";
        StateDirectory = "openxai-indexer";
        DynamicUser = true;
        CacheDirectory = "nodejs-app";
        Restart = "on-failure";
      };
    };
  };
}
