{
  description = "A very basic flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };

  outputs = { self, nixpkgs }: let
    pkgs = nixpkgs.legacyPackages."x86_64-linux"; 
  in {
    devShells."x86_64-linux".default = pkgs.mkShell rec {
      buildInputs = with pkgs; [
        cargo
        rustc
        rustfmt
        clippy
        rust-analyzer
        lld
        binaryen
        dart-sass
        wayland
        libxkbcommon
        libGL
        cargo-leptos
        leptosfmt
        nodejs
        playwright-driver
        playwright-driver.browsers
      ];
      shellHook = ''
        export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${builtins.toString (pkgs.lib.makeLibraryPath buildInputs)}";
        export PLAYWRIGHT_BROWSERS_PATH="${pkgs.playwright-driver.browsers}"
        export PLAYWRIGHT_SKIP_VALIDATE_HOST_REQUIREMENTS=1
        export PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD=1
      '';
      env.RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
    };
  };
}
