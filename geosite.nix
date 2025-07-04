{
  stdenv,
  fetchurl,
  ...
}:
stdenv.mkDerivation rec {
  pname = "xray_geosite";
  version = "202507032213";

  src = fetchurl {
    url = "https://github.com/Loyalsoldier/v2ray-rules-dat/releases/download/${version}/geosite.dat";
    sha256 = "sha256-OPPKgJ+PxNKL8xs3IJJTiLHFe/KcLNGqrceO5yIXP9k=";
  };

  unpackPhase = ":";

  installPhase = ''
    mkdir -p $out/bin
    cp $src $out/bin/geosite.dat
    runHook postInstall
  '';
}
