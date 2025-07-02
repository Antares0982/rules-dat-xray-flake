{
  stdenv,
  fetchurl,
  ...
}:
stdenv.mkDerivation rec {
  pname = "xray_geosite";
  version = "202507012214";

  src = fetchurl {
    url = "https://github.com/Loyalsoldier/v2ray-rules-dat/releases/download/${version}/geosite.dat";
    sha256 = "sha256-5VwmXhiz+MlrDK+o4/ciRTxOe4tQEM6oXkb5LmsniP0=";
  };

  unpackPhase = ":";

  installPhase = ''
    mkdir -p $out/bin
    cp $src $out/bin/geosite.dat
    runHook postInstall
  '';
}
