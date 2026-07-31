{ inputs, lib, ... }:
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
      rust-project = {
        crateNixFile = "crate.nix";
        src =
          let
            filterCargoSources =
              path: type:
              config.rust-project.crane-lib.filterCargoSources path type
              && !(lib.hasSuffix ".toml" path && !lib.hasSuffix "Cargo.toml" path);
          in
          lib.cleanSourceWith {
            src = inputs.self;
            filter =
              path: type:
              filterCargoSources path type
              || lib.hasSuffix "crate.nix" path
              || "${inputs.self}/flake.nix" == path
              || "${inputs.self}/flake.lock" == path
              || "${inputs.self}/rust-toolchain.toml" == path
              || "${inputs.self}/clippy.toml" == path
              || "${inputs.self}/rustfmt.toml" == path
              || lib.hasSuffix "rust.nix" path
              || lib.hasSuffix ".typ" path
              || lib.hasPrefix "${inputs.self}/assets" path;
          };
        defaults.perCrate.crane.args = {
          nativeBuildInputs = with pkgs; [ pkg-config ];
          buildInputs = with pkgs; [ openssl ];
        };
      };
      packages =
        let
          inherit (config.rust-project) crates;
        in
        rec {
          default = server;
          server = crates.server.crane.outputs.drv.crate;
        };
    };
}
