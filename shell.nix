{ pkgs, pre-commit-hooks, system }:
let
  toolchain = pkgs.symlinkJoin {
    name = "rust-toolchain";
    paths = with pkgs; [ rustc cargo rustPlatform.rustcSrc rustfmt clippy ];
  };
  pre-commit-check = pre-commit-hooks.lib.${system}.run {
    src = ./.;
    hooks = {
      nixpkgs-fmt.enable = true;
      rustfmt = {
        enable = true;
        package = toolchain;
      };
    };
  };
in
pkgs.mkShell rec {
  buildInputs = with pkgs; [
    toolchain
    rustPlatform.bindgenHook
    bashInteractive
  ] ++ pre-commit-check.enabledPackages;

  shellHook = pre-commit-check.shellHook;

  LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath buildInputs}";
  RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
}
