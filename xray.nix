{ pkgs, ... }:
pkgs.xray.overrideAttrs (oldAttrs: {
  postFixup = ''
    wrapProgram $out/bin/xray \
    --suffix XRAY_LOCATION_ASSET : $out/share
  '';

  postInstall = (
    let
      xrayGeoip = pkgs.callPackage ./geoip.nix { };
      xrayGeosite = pkgs.callPackage ./geosite.nix { };
    in
    ''
      mkdir -p $out/share
      ln -s ${xrayGeoip}/bin/geoip.dat $out/share/geoip.dat
      ln -s ${xrayGeosite}/bin/geosite.dat $out/share/geosite.dat
    ''
  );
})
