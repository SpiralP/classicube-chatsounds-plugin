{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-23.05";
  };

  outputs = { nixpkgs, ... }:
    let
      inherit (nixpkgs) lib;
    in
    {
      packages = lib.genAttrs lib.systems.flakeExposed (system:
        let
          pkgs = import nixpkgs {
            inherit system;
          };

          inherit (pkgs) rustPlatform dockerTools buildNpmPackage;
        in
        rec {
          default = rustPlatform.buildRustPackage {
            name = "classicube-chatsounds-plugin";
            src = lib.cleanSourceWith rec {
              src = ./.;
              filter = path: type:
                lib.cleanSourceFilter path type
                && (
                  let
                    baseName = builtins.baseNameOf (builtins.toString path);
                    relPath = lib.removePrefix (builtins.toString ./.) (builtins.toString path);
                  in
                  lib.any (re: builtins.match re relPath != null) [
                    "/Cargo.toml"
                    "/Cargo.lock"
                    "/src"
                    "/src/.*"
                  ]
                );
            };

            cargoLock = {
              lockFile = ./Cargo.lock;
              outputHashes = {
                "async-dispatcher-0.1.0" = "sha256-rqpQ176/PnI9vvPrwQvK3GJbryjb3hHkb+o1RyCZ3Vg=";
                "chatsounds-0.2.0" = "sha256-HJq5MXkXnEKGOHX+DRzVhQjLTPmar0MWW7aItqrlpys=";
                "classicube-helpers-2.0.0+classicube.1.3.5" = "sha256-E9ORHAO8rGVCMXTq2TvsQwrSV5H5WF3bAj5+OJ2f7jA=";
                "classicube-sys-2.0.0+classicube.1.3.5" = "sha256-VXHyJwF8cdX3PlTn7xgbviGg3D5bsRRh375I+DRpE4g=";
                "color-backtrace-0.3.0" = "sha256-wVf6EEmD/PqHGJtVUXBg5y2kXPXxGtQTU52WurrFv+M=";
              };
            };

            nativeBuildInputs = with pkgs; [
              pkg-config
              rustPlatform.bindgenHook
            ];

            buildInputs = with pkgs; [
              alsa-lib
              at-spi2-atk
              cairo
              gdk-pixbuf
              glib
              gtk3
              openssl
              pango
            ];

            doCheck = false;
          };
        }
      );
    };
}
