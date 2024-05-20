self: { config
      , pkgs
      , lib
      , hm
      , ...
      }:
let
  cfg = config.programs.oxidash;
  defaultPackage = self.packages.${pkgs.stdenv.hostPlatform.system}.default;
in
{
  meta.maintainers = with lib.maintainers; [ DashieTM ];
  options.programs.oxidash = with lib; {
    enable = mkEnableOption "oxidash";

    package = mkOption {
      type = with types; nullOr package;
      default = defaultPackage;
      defaultText = lib.literalExpression ''
        oxidash.packages.''${pkgs.stdenv.hostPlatform.system}.default
      '';
      description = mdDoc ''
        Package to run
      '';
    };
  };
  config = lib.mkIf cfg.enable {
    home.packages = lib.optional (cfg.package != null) cfg.package;
  };
}
