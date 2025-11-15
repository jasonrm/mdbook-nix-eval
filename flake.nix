{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    # various, usually obscure, programs that are missing from nixpkgs
    nixpkgs-staging.url = "github:jasonrm/nixpkgs-staging";

    chips = {
      url = "github:jasonrm/nix-chips";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.nixpkgs-staging.follows = "nixpkgs-staging";
    };
  };

  outputs = {
    chips,
    # rust-overlay,
    ...
  }:
    chips.lib.use {
      devShellsDir = ./nix/devShells;
    };
}
