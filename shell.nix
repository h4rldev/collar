{pkgs ? import <nixpkgs> {}}:
with pkgs;
  mkShell {
    name = "collar";
    description = "A discord bot for managing the petring webring, and petads";
    buildInputs = [
      just
      rustup
      pkg-config
      openssl

      # linters and formatters
      alejandra
      markdownlint-cli
      prettierd
      biome
      nodePackages_latest.alex
      doctoc
      cbfmt
      actionlint
      taplo
      dockerfmt
      hadolint
      fixjson

      # lsp
      nixd
      docker-language-server
    ];
  }
