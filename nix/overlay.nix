{
  crane,
  cranix,
  fenix,
}: final: prev: let
  zoomer = prev.callPackage ./. {inherit crane cranix fenix;};
in {
  zoomer = zoomer.packages.default;
}
