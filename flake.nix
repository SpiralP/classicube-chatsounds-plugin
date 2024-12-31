{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
  };

  outputs = { self, nixpkgs }:
    let
      inherit (nixpkgs) lib;

      rustManifest = lib.importTOML ./Cargo.toml;

      revSuffix = lib.optionalString (self ? shortRev || self ? dirtyShortRev)
        "-${self.shortRev or self.dirtyShortRev}";

      makePackages = (system: dev:
        let
          pkgs = import nixpkgs {
            inherit system;
          };

          defaultAttrs = {
            pname = rustManifest.package.name;
            version = rustManifest.package.version + revSuffix;

            src = lib.sourceByRegex ./. [
              "^\.cargo(/.*)?$"
              "^build\.rs$"
              "^Cargo\.(lock|toml)$"
              "^src(/.*)?$"
            ];

            cargoLock = {
              lockFile = ./Cargo.lock;
              allowBuiltinFetchGit = true;
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
