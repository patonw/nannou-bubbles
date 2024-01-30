{
  description = "A very basic flake";
  inputs = {
    flake-utils.url = github:numtide/flake-utils;
    rust-overlay.url = github:oxalica/rust-overlay;
    naersk.url = github:nix-community/naersk;
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, naersk }: flake-utils.lib.eachDefaultSystem (system:
  let
    pkgs = import nixpkgs {
      inherit system;
      overlays = [
        (import rust-overlay)
      ];
    };
    toolchain = pkgs.rust-bin.stable.latest.default;
    naersk' = pkgs.callPackage naersk {
      rustc = toolchain;
      cargo = toolchain;
    };
    build-tools = with pkgs; [
      cmake
      pkg-config
      rust-analyzer
      toolchain
    ];
    build-deps = with pkgs; [
      alsa-lib.dev
      vulkan-headers
    ];
    runtime-libs = with pkgs; [
      xorg.libX11
      xorg.libXi
      xorg.libXrandr
      xorg.libXcursor

      vulkan-loader
      #vulkan-extension-layer
      #vulkan-utility-libraries
    ];
    dev-tools = with pkgs; [
      neovim

      #vulkan-helper
      #vulkan-tools
      #vk-bootstrap
    ];
  in
  {
    packages.hello = pkgs.hello;
    #packages.default = self.packages.${system}.hello;
    packages.default = naersk'.buildPackage {
      src = ./.;
    };

    defaultApp = {
      type = "app";
      program = "${self.packages.${system}.hello}/bin/hello";
    };

    devShell = with pkgs; mkShell {
      shellHook = ''
        export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath runtime-libs}:$LD_LIBRARY_PATH
      '';
      nativeBuildInputs = build-tools;
      buildInputs = build-deps;
      packages = dev-tools;
    };
  });
}
