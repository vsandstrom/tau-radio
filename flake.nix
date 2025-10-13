{
  description = "Tau webradio client - Nix flake";

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
        linuxDeps = if pkgs.stdenv.isLinux then with pkgs; [
          jack2
          alsa-lib
        ] else [];
      in 
    {
      packages.default = craneLib.buildPackage {
        src = craneLib.cleanCargoSource ./.;

        nativeBuildInputs = with pkgs; [
          pkg-config
          rustPlatform.bindgenHook
        ];

        buildInputs = with pkgs; [
          libshout
          # libopusenc
          libopus
          libogg
        ] ++ linuxDeps;

        doCheck = false;

        env.NIX_CFLAGS_COMPILE = "-I${pkgs.libopus.dev}/include/opus";
        # Prevent bindgens from constantly rebuilding:
        # https://crane.dev/faq/rebuilds-bindgen.html?highlight=bindgen#i-see-the-bindgen-crate-constantly-rebuilding
        env.NIX_OUTPATH_USED_AS_RANDOM_SEED = "aaaaaaaaaa";
      };
    });
}
