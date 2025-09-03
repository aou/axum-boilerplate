# clanky: to build static, uncomment glibc.static, then
# RUSTFLAGS="-C target-feature=+crt-static" cargo build --release --target x86_64-unknown-linux-gnu
{
  description = "axum-boilerplate";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            # gcc
            # openssl
            # postgresql
            # sqlite
          ];
          nativeBuildInputs = with pkgs; [
            duckdb
            postgresql
            # gcc
            # glibc.static
          ];
        };
      });
}
