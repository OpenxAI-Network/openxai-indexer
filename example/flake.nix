{
  inputs = {
    xnode-manager.url = "github:Openmesh-Network/xnode-manager";
    openxai-indexer.url = "github:OpenxAI-Network/openxai-indexer";
    nixpkgs.follows = "openxai-indexer/nixpkgs";
  };

  outputs = inputs: {
    nixosConfigurations.container = inputs.nixpkgs.lib.nixosSystem {
      specialArgs = {
        inherit inputs;
      };
      modules = [
        inputs.xnode-manager.nixosModules.container
        {
          services.xnode-container.xnode-config = {
            host-platform = ./xnode-config/host-platform;
            state-version = ./xnode-config/state-version;
            hostname = ./xnode-config/hostname;
          };
        }
        inputs.openxai-indexer.nixosModules.default
        {
          services.openxai-indexer = {
            enable = true;
            verbosity = "info";
            claimerkey = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
            tokenownerkey = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
            tokenminterkey = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
            rpc = {
              http = "https://base-sepolia-rpc.publicnode.com";
              ws = "wss://base-sepolia-rpc.publicnode.com";
            };
            contracts = {
              usdc = "0x036CbD53842c5426634e7929541eC2318f3dCF7e";
            };
            chainId = 84532;
            hyperstackapikey = "";
          };
        }
      ];
    };
  };
}
