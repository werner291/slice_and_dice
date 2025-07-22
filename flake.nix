{
  description = "Foo Bar";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    pre-commit-hooks = { url = "github:cachix/pre-commit-hooks.nix"; };
  };
  outputs = { self, nixpkgs, pre-commit-hooks }:
    let
      supportedSystems = [ "x86_64-linux" ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
      pkgsFor = nixpkgs.legacyPackages;
    in
    {
      formatter.x86_64-linux = nixpkgs.legacyPackages.x86_64-linux.nixpkgs-fmt;
      devShells = forAllSystems (system: {
        default = pkgsFor.${system}.callPackage ./shell.nix { inherit pre-commit-hooks; inherit system; };
      });
      packages = forAllSystems (system: {
        default = pkgsFor.${system}.callPackage ./. { };
      });
    };
}
