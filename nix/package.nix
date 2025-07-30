{ pkgs, rustPlatform }:
rustPlatform.buildRustPackage {
  pname = "openxai-indexer";
  version = "1.0.0";
  src = ../rust-app;

  cargoLock = {
    lockFile = ../rust-app/Cargo.lock;
  };

  doDist = false;

  buildInputs = with pkgs; [
    openssl
  ];
  nativeBuildInputs = with pkgs; [
    pkg-config
  ];

  meta = {
    mainProgram = "openxai-indexer";
  };
}
