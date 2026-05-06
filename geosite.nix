{
  stdenv,
  fetchurl,
  ...
}:
stdenv.mkDerivation rec {
  pname = "xray_geosite";
  version = "202605052250";

  src = fetchurl {
    url = "https://github.com/Loyalsoldier/v2ray-rules-dat/releases/download/${version}/geosite.dat";
    sha256 = "sha256-VDHCN5I5zISqdZtGd7OmnXK863Tu/iOADnCOoDIuTCE=";
  };

  unpackPhase = ":";

  installPhase = ''
    mkdir -p $out/bin
    cp $src $out/bin/geosite.dat
    runHook postInstall
  '';
}
