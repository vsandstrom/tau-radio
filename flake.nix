{
  description = "Tau webradio Nix flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    crate.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, crane, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system: 
      let 
        pkgs = import nixpkgs {inherit system;};
        craneLib = crane.mkLib pkgs;
      in 
    {
      packages.default = craneLib.buildPackage {
        src = craneLib.cleanCargoSource ./.;
        # pname = "tau-radio";
        # version = "0.1.0"; 
        # src = ./.;
        # cargoVendorDir = ./vendor;

        # cargoSha256 = "sha256-aaaaaaaaaaaaaaaaaaaaaaaa";
        # cargoLock = ./Cargo.lock;

        nativeBuildInputs = [pkgs.pkg-config];
        buildInputs = [
          pkgs.libshout
          pkgs.libopusenc
          pkgs.libopus
          pkgs.libogg
        ];

        doCheck = false;
      };

      # devShell = pkgs.mkShell {
      #   nativeBuildInputs = [pkgs.pkg-config];
      #   buildInputs = [
      #     pkgs.rustup
      #     pkgs.cargo
      #     pkgs.libshout
      #     pkgs.libopusenc
      #     pkgs.libopus
      #     pkgs.libogg
      #   ];
      #   shellHook = ''
      #     rustup override set stable
      #   '';
      #
      # };
    });
}
