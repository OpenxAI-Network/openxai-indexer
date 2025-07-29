{
  config,
  pkgs,
  lib,
  ...
}:
let
  cfg = config.services.xnode-rust-template;
  xnode-rust-template = pkgs.callPackage ./package.nix { };
in
{
  options = {
    services.xnode-rust-template = {
      enable = lib.mkEnableOption "Enable the rust app";
    };
  };

  config = lib.mkIf cfg.enable {
    users.groups.xnode-rust-template = { };
    users.users.xnode-rust-template = {
      isSystemUser = true;
      group = "xnode-rust-template";
    };

    systemd.services.xnode-rust-template = {
      wantedBy = [ "multi-user.target" ];
      description = "Rust App.";
      after = [ "network.target" ];
      serviceConfig = {
        ExecStart = "${lib.getExe xnode-rust-template}";
        User = "xnode-rust-template";
        Group = "xnode-rust-template";
      };
    };
  };
}
