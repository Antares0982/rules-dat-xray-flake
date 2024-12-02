{
  stdenv,
  fetchurl,
  ...
}:
stdenv.mkDerivation rec {
  pname = "xray_geoip";
  version = "202412012212";

  src = fetchurl {
    url = "https://github.com/Loyalsoldier/v2ray-rules-dat/releases/download/${version}/geoip.dat";
    sha256 = "sha256-UTSnW3/xfB7JfJGhWiE2OxXeb5HFVLLNhCWlWuq9yaU=";
  };

  unpackPhase = ":";

  installPhase = ''
    mkdir -p $out/bin
    cp $src $out/bin/geoip.dat
    runHook postInstall
  '';
}
