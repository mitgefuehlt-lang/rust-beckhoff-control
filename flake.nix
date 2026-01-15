{
  description = "QiTech Rust Control Production";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
    flake-utils.url = "github:numtide/flake-utils";
    crane = {
      url = "github:ipetkov/crane";
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, crane, rust-overlay, ... }:
    flake-utils.lib.eachSystem [ "x86_64-linux" "aarch64-linux" ] (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        craneLib = crane.mkLib pkgs;

        # Server Package
        server = pkgs.callPackage ./nixos/packages/server.nix {
          inherit craneLib;
          libudev-zero = pkgs.libudev-zero; # Ensure this exists or use systemd
          libpcap = pkgs.libpcap;
        };

        # Electron Package
        electron = pkgs.callPackage ./nixos/packages/electron.nix { };

      in
      {
        packages = {
          inherit server electron;
          default = server;
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            cargo
            rustc
            rust-analyzer
            clippy
            rustfmt
            pkg-config
            udev
            openssl
            nodejs
            python3
          ];
          
          RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
        };
      }
    ) // {
      # System Configuration
      nixosConfigurations.nixos = nixpkgs.lib.nixosSystem {
        system = "x86_64-linux";
        modules = [
          ./nixos/os/configuration.nix
          ./nixos/modules/qitech.nix
          ({ pkgs, ... }: {
            nixpkgs.overlays = [
              (final: prev: {
                qitechPackages = self.packages.x86_64-linux;
              })
            ];
          })
        ];
      };
    };
}
