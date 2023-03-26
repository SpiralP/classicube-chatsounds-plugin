{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-22.11";
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
            cargoSha256 = "sha256-BsKoyHPRVgrkzoQL5ls82UNLyHYPqjJ6pXIH1HKJt5s=";
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
