{
  description = "rustlog project";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };

  outputs = {
    self,
    nixpkgs,
  }: let
    supportedSystems = [
      "x86_64-linux"
      "aarch64-linux"
      "x86_64-darwin"
      "aarch64-darwin"
    ];

    eachSystem = nixpkgs.lib.genAttrs supportedSystems (
      system: let
        pkgs = nixpkgs.legacyPackages.${system};
        nativeBuildInputs = with pkgs; [
          cargo
          rustc
          rustfmt
          rust-analyzer
          clippy
        ];
        buildInputs = [];
      in {
        devShell = pkgs.mkShell {
          inherit nativeBuildInputs buildInputs;
          packages = with pkgs; [
            llvmPackages_latest.llvm
            cargo-llvm-cov
            gnuplot
          ];
          shellHook = ''
            export LLVM_COV=$(which llvm-cov)
            export LLVM_PROFDATA=$(which llvm-profdata)

            echo "Rust toolchain: $(rustc --version)"
            echo "Rust analyzer: $(rust-analyzer --version)"
            echo "Clippy: $(clippy-driver --version)"
          '';
        };
      }
    );
  in {
    devShells =
      nixpkgs.lib.mapAttrs (system: systemAttrs: {
        default = systemAttrs.devShell;
      })
      eachSystem;
  };
}
