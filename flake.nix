{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
  };

  outputs = { nixpkgs, ... }:
    let
      inherit (nixpkgs) lib;

      makePackages = (system: dev:
        let
          pkgs = import nixpkgs {
            inherit system;
          };
          rustManifest = lib.importTOML ./Cargo.toml;

          defaultAttrs = {
            pname = rustManifest.package.name;
            version = rustManifest.package.version;

            src = lib.sourceByRegex ./. [
              "^\.cargo(/.*)?$"
              "^build\.rs$"
              "^Cargo\.(lock|toml)$"
              "^src(/.*)?$"
            ];

            cargoLock = {
              lockFile = ./Cargo.lock;
              outputHashes = {
                "async-dispatcher-0.1.0" = "sha256-rqpQ176/PnI9vvPrwQvK3GJbryjb3hHkb+o1RyCZ3Vg=";
                "chatsounds-0.2.0" = "sha256-l9Fk/qRdhxhFneXoLEszG5QTWwS+LwFCu6essLzbT5c=";
                "classicube-helpers-3.0.0+classicube.1.3.7" = "sha256-3hWKS6NmAH0x+SOi/nBKJLIQi/3ilG7WSRrPvF++wGE=";
                "color-backtrace-0.3.0" = "sha256-wVf6EEmD/PqHGJtVUXBg5y2kXPXxGtQTU52WurrFv+M=";
              };
            };

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

            nativeBuildInputs = with pkgs; [
              pkg-config
              rustPlatform.bindgenHook
            ] ++ (if dev then
              with pkgs; ([
                cargo-release
                clippy
                (rustfmt.override { asNightly = true; })
                rust-analyzer
              ]) else [ ]);
          };
        in
        {
          default = pkgs.rustPlatform.buildRustPackage defaultAttrs;

          debug = (pkgs.enableDebugging {
            inherit (pkgs) stdenv;
            override = (attrs: pkgs.makeRustPlatform ({
              inherit (pkgs) rustc cargo;
            } // attrs));
          }).buildRustPackage (
            (defaultAttrs // {
              pname = "${defaultAttrs.pname}-debug";

              buildType = "debug";

              hardeningDisable = [ "all" ];
            })
          );
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
