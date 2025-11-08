{
  description = "any-subtraits";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.05";
    nixpkgs-unstable.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url  = "github:numtide/flake-utils";
  };

  outputs = flakes@{ self, nixpkgs, nixpkgs-unstable, rust-overlay, flake-utils, ... }:
  flake-utils.lib.eachDefaultSystem (system:
    let
      overlays = [ (import rust-overlay) ];
      pkgs = import nixpkgs {
        inherit system overlays;
      };
      pkgs-unstable = import nixpkgs-unstable {
        inherit system overlays;
      };
      #RUST_VERSION="1.85.0";
      RUST_VERSION="2025-10-31";
    in
    {
      devShells.default = pkgs.mkShell {
        name = "any-trait";
        buildInputs = with pkgs; [
          # system deps
           git
           gnupg
           openssh
           openssl
           fd
          # rust deps
           #(rust-bin.stable.latest.default.override {
           # go with nightly to have async fn in traits
           #(rust-bin.nightly."2023-02-01".default.override {
           #  #extensions = [ "rust-src" ];
           #  #targets = [ "arm-unknown-linux-gnueabihf" ];
           #})
           #clippy
           cargo-watch
           cargo-flamegraph
           cargo-license
           cargo-expand
           lld
           #rust-bin.stable.${RUST_VERSION}.default
           #rust-bin.beta.${RUST_VERSION}.default
           (rust-bin.nightly.${RUST_VERSION}.default.override {
              extensions = [ "rust-src" "rustfmt" "rust-analyzer" "clippy" "cargo" "rustc" "rust-src" "miri" ];
           })
           #rust-bin.nightly.${RUST_VERSION}.rustfmt
           #rust-bin.nightly.${RUST_VERSION}.rust-analyzer
           #rust-bin.nightly.${RUST_VERSION}.clippy
           #rust-bin.nightly.${RUST_VERSION}.cargo
           #rust-bin.nightly.${RUST_VERSION}.rustc
           #rust-bin.nightly.${RUST_VERSION}.rust-src
           #rustfmt
           #rust-analyzer
           #clang_16
           #mold
        ];
        # if you want to try the mold linker, add 'clang_16', 'mold', and append this to ~/.cargo/config.toml:
        #  [target.x86_64-unknown-linux-gnu]
        #  linker = "clang"
        #  rustflags = ["-C", "link-arg=--ld-path=mold"]

        shellHook = ''
          # use zsh or other custom shell
          USER_SHELL="$(grep $USER /etc/passwd | cut -d ':' -f 7)"
          if [ -n "$USER_SHELL" ]; then
            export SHELL=$USER_SHELL
            exec $USER_SHELL
          fi
        '';
      };
    }
  );
}
