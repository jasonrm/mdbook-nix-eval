{ pkgs ? import <nixpkgs> {} }:
let
  inherit (pkgs.lib) optionals;
  inherit (pkgs.stdenv.hostPlatform) isDarwin;

  rust-toolchain = pkgs.symlinkJoin {
    name = "rust-toolchain";
    paths = [
        pkgs.rustc
        pkgs.cargo
        pkgs.rustPlatform.rustcSrc
    ];
  };
in
{
  devEnv = pkgs.stdenv.mkDerivation rec {
    name = "dev";
    buildInputs = with pkgs; [
      stdenv
      cargo
      rust-toolchain
      cargo-watch
      mdbook
      libiconv
      # pkg-config
    ] ++ optionals isDarwin [
      darwin.apple_sdk.frameworks.Security
    ];

    shellHook = ''
      echo "If using the Rust plugin for JetBrains, the following paths are helpful"
      echo "RUST_TOOLCHAIN_LOCATION:  ${rust-toolchain}"
      echo "RUST_STDLIB_LOCATION:     ${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}"
    '';
  };
}
