{
    description = "wayland + windows";

    inputs = {
        nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
        rust-overlay.url = "github:oxalica/rust-overlay";
        flake-utils.url = "github:numtide/flake-utils";
    };

    outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
        flake-utils.lib.eachDefaultSystem (system:
            let
                overlays = [ (import rust-overlay) ];
                pkgs = import nixpkgs {
                    inherit system overlays;
                };
            in
            with pkgs;
            {
                devShells.default = mkShell {
                    buildInputs = [
                        (rust-bin.stable.latest.default.override {
                          extensions = ["rust-src"];
                          targets = [ "x86_64-pc-windows-msvc" "x86_64-unknown-linux-gnu" "x86_64-unknown-linux-musl" ];
                        })
                    ];
                };
            }
    );
}
