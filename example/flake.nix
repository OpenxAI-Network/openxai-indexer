{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    xnode-rust-template.url = "github:Openmesh-Network/xnode-rust-template";
  };

  outputs =
    {
      self,
      nixpkgs,
      xnode-rust-template,
      ...
    }:
    let
      system = "x86_64-linux";
    in
    {
      nixosConfigurations.container = nixpkgs.lib.nixosSystem {
        inherit system;
        specialArgs = {
          inherit xnode-rust-template;
        };
        modules = [
          (
            { xnode-rust-template, ... }:
            {
              imports = [
                xnode-rust-template.nixosModules.default
              ];

              boot.isContainer = true;

              services.xnode-rust-template = {
                enable = true;
              };

              networking = {
                useHostResolvConf = nixpkgs.lib.mkForce false;
              };

              services.resolved.enable = true;

              system.stateVersion = "25.05";
            }
          )
        ];
      };
    };
}
