{ inputs, ... }:
{
  perSystem =
    {
      config,
      self',
      inputs',
      pkgs,
      system,
      ...
    }:
    {
      devShells.default = pkgs.mkShell {
        name = "void-srv-devshell";
        meta.description = "Void Server Development Environment";
        inputsFrom = [
          self'.devShells.rust
          config.pre-commit.devShell
        ];
        packages = with pkgs; [
          nixd
          nixfmt
          jq
          stdenv.cc
          just
          cargo-workspaces
          typst
          podman
          podman-compose
        ];
        nativeBuildInputs = with pkgs; [
          pkg-config
        ];
        buildInputs = with pkgs; [
          openssl
        ];
        shellHook = ''
          export PKG_CONFIG_PATH="${pkgs.openssl.dev}/lib/pkgconfig"
          echo "rustc version: `rustc --version`"
          echo "cargo version: `cargo --version`"
        '';
      };
    };
}
