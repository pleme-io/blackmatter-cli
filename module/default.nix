# blackmatter-cli — home-manager module.
#
# Puts the typed fleet CLI on the user's PATH so `blackmatter-cli report`,
# `blackmatter-cli check`, etc. work directly from a terminal without a
# project-local `nix run`.
#
# The module is intentionally tiny — `mkPackageOption` with an enable flag
# is the canonical HM shape for a single binary. No further surface is
# warranted until there's a second use-case (e.g. per-user config).
{
  config,
  lib,
  pkgs,
  ...
}: let
  cfg = config.blackmatter.components.cli;
in {
  options.blackmatter.components.cli = {
    enable = lib.mkEnableOption "blackmatter-cli (typed fleet CLI)";
    package = lib.mkPackageOption pkgs "blackmatter-cli" { };
  };

  config = lib.mkIf cfg.enable {
    home.packages = [ cfg.package ];
  };
}
