{ pkgs, lib }:
let
  pname = "xnode-nodejs-template";
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
    mkdir -p $out/{share,bin}
    cp -r build $out/share/build
    cp -r node_modules $out/share/node_modules
    cp -r package.json $out/share/package.json

    cat > $out/bin/${pname} << EOF
    #!/bin/sh
    ${pkgs.nodejs}/bin/npm run start --prefix $out/share
    EOF
    chmod +x $out/bin/${pname}
  '';

  doDist = false;

  meta = {
    mainProgram = "xnode-nodejs-template";
  };
}
