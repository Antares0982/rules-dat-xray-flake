{
  stdenv,
  fetchurl,
  ...
}:
stdenv.mkDerivation rec {
  pname = "xray_geoip";
  version = "202607122240";

  src = fetchurl {
    url = "https://github.com/Loyalsoldier/v2ray-rules-dat/releases/download/${version}/geoip.dat";
    sha256 = "sha256-g3l3GfrMCS4hD4+ODl4LC9/gaskKOko9amqy14GpF64=";
  };

  unpackPhase = ":";

  installPhase = ''
    mkdir -p $out/bin
    cp $src $out/bin/geoip.dat
    runHook postInstall
  '';
}
