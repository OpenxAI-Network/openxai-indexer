{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    openxai-indexer = {
      url = "path:.."; # "github:OpenxAI-Network/openxai-indexer";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      openxai-indexer,
      ...
    }:
    let
      system = "x86_64-linux";
    in
    {
      nixosConfigurations.container = nixpkgs.lib.nixosSystem {
        inherit system;
        specialArgs = {
          inherit openxai-indexer;
        };
        modules = [
          (
            { openxai-indexer, ... }:
            {
              imports = [
                openxai-indexer.nixosModules.default
              ];

              boot.isContainer = true;

              services.openxai-indexer = {
                enable = true;
              };

              networking = {
                firewall.allowedTCPPorts = [ 3001 ];
              };

              system.stateVersion = "25.05";
            }
          )
        ];
      };
    };
}
