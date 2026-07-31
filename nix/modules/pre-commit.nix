{ inputs, ... }:
{
  imports = [
    (inputs.git-hooks + /flake-module.nix)
  ];
  perSystem =
    {
      config,
      self',
      inputs',
      pkgs,
      system,
      lib,
      ...
    }:
    {
      pre-commit = {
        check.enable = true;
        settings = {
          hooks = {
            nixfmt.enable = true;
            rustfmt.enable = true;
          };
        };
      };
    };
}
