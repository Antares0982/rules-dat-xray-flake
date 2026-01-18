{
  description = "A very basic flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
  };

  outputs =
    inputs@{ self, nixpkgs }:
    let
      forAllSystems =
        function:
        nixpkgs.lib.genAttrs
          [
            "x86_64-linux"
            "aarch64-linux"
            "x86_64-darwin"
            "aarch64-darwin"
          ]
          (
            system:
            function (
              import nixpkgs {
                inherit system;
              }
            )
          );
    in
    {
      packages = forAllSystems (pkgs: rec {
        xray = pkgs.callPackage ./xray.nix { };
        default = xray;
        geoip = pkgs.callPackage ./geoip.nix { };
        geosite = pkgs.callPackage ./geosite.nix { };
        xray_sub = pkgs.callPackage ./xray_sub.nix { };
      });
    };
}
