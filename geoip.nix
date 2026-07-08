{
  stdenv,
  fetchurl,
  ...
}:
stdenv.mkDerivation rec {
  pname = "xray_geoip";
  version = "202607072255";

  src = fetchurl {
    url = "https://github.com/Loyalsoldier/v2ray-rules-dat/releases/download/${version}/geoip.dat";
    sha256 = "sha256-k7FF+QAXMYwtIo7n6vU5Ip18Ru1ZhJnyLroWhh6sWa0=";
  };

  unpackPhase = ":";

  installPhase = ''
    mkdir -p $out/bin
    cp $src $out/bin/geoip.dat
    runHook postInstall
  '';
}
