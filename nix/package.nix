{ pkgs, lib }:
let
  pname = "openxai-indexer";
in
pkgs.buildNpmPackage {
  inherit pname;
  version = "1.0.0";
  src = ../nodejs-app;

  npmDeps = pkgs.importNpmLock {
    npmRoot = ../nodejs-app;
  };
  npmConfigHook = pkgs.importNpmLock.npmConfigHook;

  postBuild = ''
    # Add a shebang to the server js file, then patch the shebang to use a
    # nixpkgs nodes binary
    sed -i '1s|^|#!/usr/bin/env node\n|' build/index.js
    patchShebangs build/index.js
  '';

  installPhase = ''
    mkdir -p $out
    cp package.json $out/package.json
    cp -r node_modules $out/node_modules
    cp -r build $out/build
    chmod +x $out/build/index.js
    makeWrapper $out/build/index.js $out/bin/openxai-indexer 
  '';

  doDist = false;

  meta = {
    mainProgram = "openxai-indexer";
  };
}
