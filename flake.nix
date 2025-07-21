{
  description = "";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      utils,
    }:
    utils.lib.eachDefaultSystem (
      system:
      let
        lib = pkgs.lib;
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        devShells.default = pkgs.mkShell {

          buildInputs =
            with pkgs;
            lib.flatten [
              rustup
              pkgsCross.mingwW64.buildPackages.gcc

              u-config
              wayland
              wayland-protocols
            ];

          packages = with pkgs; [
            cargo-cross
            pkgsCross.mingwW64.buildPackages.gcc
            cargo-dist
          ];

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (
            with pkgs;
            [
              wayland
              wayland-protocols
              libGL
              libxkbcommon
            ]
          );
        };
      }
    );
}
