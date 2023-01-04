{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell {
  packages = [ 
      pkgs.rustup 
      pkgs.gcc 
      pkgs.fontconfig 
      pkgs.xorg.libXcursor 
      pkgs.xorg.libXi 
      pkgs.vulkan-validation-layers 
      pkgs.vulkan-loader 
      pkgs.vulkan-headers 
      pkgs.vulkan-tools
      pkgs.vulkan-tools-lunarg
      pkgs.renderdoc
    ];

  inputsFrom = [ pkgs.vulkan-validation-layers  pkgs.vulkan-loader ];

  shellHook = ''
    export RUST_LOG=debug
    export ENABLE_VALIDATION=1
  '';

    LD_LIBRARY_PATH="${pkgs.vulkan-loader}/lib";
}