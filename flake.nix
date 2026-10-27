{
  description = "wavepool";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        stdenv = pkgs.stdenv;
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rust-bin.stable.latest.default
            rust-analyzer
            cargo-watch
            cargo-edit
            pkgs.nixfmt
            cmake
            libvterm
            glib
          ];
        };

        # cargo build
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "wavepool";
          version = "0.1.0";

          src = ./.;

          cargoBuildFlags = [
            "--examples"
          ];

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

	  postPatch = ''
	    echo "rustc $(rustc --version)' >2&"
	  '';

          postInstall = ''
            mkdir -p $out/bin
            cp target/${stdenv.hostPlatform.rust.rustcTarget}/release/examples/airshow $out/bin/
          '';
        };
      }
    );
}
