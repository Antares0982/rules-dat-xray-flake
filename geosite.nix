{
  stdenv,
  fetchurl,
  ...
}:
stdenv.mkDerivation rec {
  pname = "xray_geosite";
  version = "202606082308";

  src = fetchurl {
    url = "https://github.com/Loyalsoldier/v2ray-rules-dat/releases/download/${version}/geosite.dat";
    sha256 = "sha256-fGmOU9OiCPfbwQ9YyUtjhclKwrgN3h2HitalMc77Rpk=";
  };

  unpackPhase = ":";

  installPhase = ''
    mkdir -p $out/bin
    cp $src $out/bin/geosite.dat
    runHook postInstall
  '';
}
