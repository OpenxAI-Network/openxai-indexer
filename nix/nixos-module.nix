{
  config,
  pkgs,
  lib,
  ...
}:
let
  cfg = config.services.xnode-nodejs-template;
  xnode-nodejs-template = pkgs.callPackage ./package.nix { };
in
{
  options = {
    services.xnode-nodejs-template = {
      enable = lib.mkEnableOption "Enable the node.js app";
    };
  };

  config = lib.mkIf cfg.enable {
    systemd.services.xnode-nodejs-template = {
      wantedBy = [ "multi-user.target" ];
      description = "Node.js App.";
      after = [ "network.target" ];
      serviceConfig = {
        ExecStart = "${lib.getExe xnode-nodejs-template}";
        DynamicUser = true;
        CacheDirectory = "nodejs-app";
      };
    };
  };
}
