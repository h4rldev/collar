{pkgs ? import <nixpkgs> {}}:
with pkgs;
  mkShell {
    name = "collar";
    description = "A discord bot for managing the petring webring, and petads";
    buildInputs = [
      rustup
      pkg-config
      openssl
    ];
  }
