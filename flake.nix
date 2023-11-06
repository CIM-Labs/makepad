{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:  
    flake-utils.lib.eachDefaultSystem 
      (system:
        let
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs {
            inherit system overlays;
            config.allowUnfree = true;
          };
          rustToolchain = pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
       in
        with pkgs;
        {
          modules = [
            nix-ld.nixosModules.nix-ld
            { programs.nix-ld.dev.enable = true; }
          ];

          fonts.packages = with pkgs; [
            noto-fonts
            noto-fonts-cjk
            noto-fonts-emoji
            liberation_ttf
            fira-code
            fira-code-symbols
            mplus-outline-fonts.githubRelease
            dina-font
            proggyfonts
          ];

          vscodeExtensions = with pkgs.vscode-extensions; [
            bmewburn.vscode-intelephense-client
            swellaby.rust-pack
            rust-lang.rust-analyzer
            sdras.night-owl
            mkloubert.vscode-remote-workspace
            mjbvz.markdown-mermaid
            tomoki1207.pdf
            arrterian.nix-env-selector
          ];

          devShells.default = mkShell {
            buildInputs = [
              rustToolchain
              cargo-edit
              cargo-expand
              cargo-udeps
              cargo-whatfeatures
              lld
              clang
              gcc
              zsh
              git
              just
              starship
              openssl
              openssl.dev
              vscode-with-extensions
              pkg-config
              zlib.dev
              (with dotnetCorePackages; combinePackages [
                sdk_6_0
                sdk_7_0
              ])
              binutils
              alsaLib
            ];

          shellHook = ''
            if [ -f .env ]; then
              export $(grep -v '^#' .env | xargs)
            fi
            export GIT_CONFIG_NOSYSTEM=1
            ZSH_CUSTOM=$HOME/.config/zsh
          '';
        };
        }
      );
}
