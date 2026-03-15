{
  description = "Comms Relay - Cross-posting relay for Mastodon and Bluesky";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      crane,
      rust-overlay,
      flake-utils,
      ...
    }:
    (flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [
            "rust-src"
            "clippy"
            "rustfmt"
          ];
        };

        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        # Filter source to include Rust files + .sqlx/ + migrations/
        src = pkgs.lib.cleanSourceWith {
          src = ./.;
          filter =
            path: type:
            (craneLib.filterCargoSources path type)
            || (builtins.match ".*/.sqlx/.*" path != null)
            || (builtins.match ".*/migrations/.*" path != null);
        };

        commonArgs = {
          inherit src;
          strictDeps = true;
          SQLX_OFFLINE = "true";

          nativeBuildInputs = with pkgs; [
            pkg-config
          ];

          buildInputs =
            with pkgs;
            [
              openssl
            ]
            ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
              pkgs.darwin.apple_sdk.frameworks.Security
              pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
            ];
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        uplink = craneLib.buildPackage (
          commonArgs
          // {
            inherit cargoArtifacts;
            cargoExtraArgs = "--bin uplink";
          }
        );

      in
      {
        checks = {
          inherit uplink;
        };

        packages = {
          inherit uplink;
          default = uplink;
        };

        apps.default = flake-utils.lib.mkApp {
          drv = uplink;
        };

        devShells.default = craneLib.devShell {
          checks = self.checks.${system};

          packages = with pkgs; [
            sqlx-cli
            bacon
            pgcli
          ];

          SQLX_OFFLINE = "true";
        };
      }
    ))
    // {
      # Overlay for NixOS/Home Manager
      overlays.default = final: prev: {
        uplink = self.packages.${final.system}.uplink;
      };
    };
}
