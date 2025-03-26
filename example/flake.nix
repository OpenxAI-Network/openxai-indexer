{
  # Define the inputs for the Nix flake
  inputs = {
    # Use the unstable branch of Nixpkgs as a dependency
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    # Reference the OpenxAI Indexer repository
    openxai-indexer = {
      url = "github:OpenxAI-Network/openxai-indexer";
      inputs.nixpkgs.follows = "nixpkgs"; # Ensure the same nixpkgs version is used
    };
  };

  # Define the flake outputs
  outputs =
    {
      self,  # The current flake itself
      nixpkgs,  # The Nixpkgs input
      openxai-indexer,  # The OpenxAI Indexer input
      ...
    }:
    let
      system = "x86_64-linux";  # Define the system architecture
    in
    {
      # Define a NixOS container configuration
      nixosConfigurations.container = nixpkgs.lib.nixosSystem {
        inherit system;  # Pass the system architecture

        specialArgs = {
          inherit openxai-indexer;  # Pass the OpenxAI Indexer input
        };

        modules = [
          (
            { openxai-indexer, ... }:
            {
              # Import the default module from the OpenxAI Indexer repository
              imports = [
                openxai-indexer.nixosModules.default
              ];

              # Indicate that this configuration is for a container
              boot.isContainer = true;

              # Enable the OpenxAI Indexer service
              services.openxai-indexer = {
                enable = true;
              };

              # Configure firewall settings to allow traffic on port 3001
              networking = {
                firewall.allowedTCPPorts = [ 3001 ];
              };

              # Define the system state version for compatibility
              system.stateVersion = "25.05";
            }
          )
        ];
      };
    };
}
