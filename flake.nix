{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, fenix }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; overlays = [ fenix.overlays.default ]; };

        rustfmt-nightly = (pkgs.rustfmt.override { asNightly = true; });
        fenixPkgs = fenix.packages.${system};
        rust-toolchain = with fenixPkgs; combine [
          targets.wasm32-unknown-unknown.latest.rust-std
          latest.toolchain
          rustfmt-nightly
        ];
      in
      {
        packages = {
          inherit rust-toolchain;
        };

        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            rust-toolchain
            pkg-config
          ];

          buildInputs = with pkgs; [
            openssl
            clang
          ] ++ (if pkgs.stdenv.isDarwin then [ libiconv ] else [ ]);

          packages = with pkgs; [
            cargo-criterion
            cargo-nextest
            wabt
            rustfmt
            rust-analyzer

            wasm-bindgen-cli
            symbolicator # for wasm-split
            binaryen
            wabt
            entr
            nodePackages_latest.live-server
            concurrently
            http-server
          ];

          RUST_BACKTRACE = "1";
          RUST_LOG = "debug,wasm_bindgen=info,walrus=info";
        };
      }
    );
}
