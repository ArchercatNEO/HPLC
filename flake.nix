{
  description = "A very basic flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };

  outputs = { self, nixpkgs }: let
    pkgs = import nixpkgs {
      system = "x86_64-linux";
    };
  in {
    devShells.x86_64-linux.default = pkgs.mkShell {
      packages = [
        pkgs.pkg-config

        pkgs.fontconfig
        pkgs.freetype
        pkgs.libxkbcommon
        pkgs.vulkan-loader
        pkgs.wayland

        pkgs.cargo
        pkgs.rustc
        pkgs.rust-analyzer
      ];

      env = {
        LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
          pkgs.fontconfig
          pkgs.freetype
          pkgs.libxkbcommon
          pkgs.vulkan-loader
          pkgs.wayland
        ];

        RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
      };
    };
  };
}
