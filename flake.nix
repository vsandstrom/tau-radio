{
  description = "Tau webradio Nix flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    crane.url = "github:ipetkov/crane";
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
