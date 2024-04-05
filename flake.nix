{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-23.11";
  };

  outputs = { nixpkgs, ... }:
    let
      inherit (nixpkgs) lib;

      makePackages = (system: dev:
        let
          pkgs = import nixpkgs {
            inherit system;
          };
        in
        {
          default = pkgs.rustPlatform.buildRustPackage {
            name = "classicube-chatsounds-plugin";
            src = lib.cleanSourceWith {
              src = ./.;
              filter = path: type:
                lib.cleanSourceFilter path type
                && (
                  lib.any (re: builtins.match re (lib.removePrefix (builtins.toString ./.) (builtins.toString path)) != null) [
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
                "chatsounds-0.2.0" = "sha256-PnggDT0oWtRRowrGoD8Bi8+Fpss6SKzQ1PDk3n1tCBM=";
                "classicube-helpers-2.0.0+classicube.1.3.6" = "sha256-V5PBZR0rj42crA1fGUjMk4rDh0ZpjjNcbMCe6bgotW8=";
                "color-backtrace-0.3.0" = "sha256-wVf6EEmD/PqHGJtVUXBg5y2kXPXxGtQTU52WurrFv+M=";
              };
            };

            nativeBuildInputs = with pkgs; [
              pkg-config
              rustPlatform.bindgenHook
            ] ++ (if dev then
              with pkgs; [
                clippy
                rustfmt
                rust-analyzer
              ] else [ ]);

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
    in
    builtins.foldl' lib.recursiveUpdate { } (builtins.map
      (system: {
        devShells.${system} = makePackages system true;
        packages.${system} = makePackages system false;
      })
      lib.systems.flakeExposed);
}
