{ crane
, cranix
, fenix
,
}: { config
   , lib
   , pkgs
   , ...
   }:
with lib; let
  zoomer= import ./. {
    inherit crane cranix fenix pkgs lib;
    system = pkgs.system;
  };
  cfgZoomer = config.programs.zoomer;
  # Temp config
  zoomerPackage = lists.optional cfgZoomer.enable zoomer.packages.default;
in
{
  options.programs = {
    zoomer = {
      enable = mkEnableOption "enable zoomer";
    };
  };

  config = mkIf cfgZoomer.enable {
    home.packages = zoomerPackage;
  };
}
