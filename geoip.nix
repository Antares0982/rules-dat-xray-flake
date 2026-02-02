{
  stdenv,
  fetchurl,
  ...
}:
stdenv.mkDerivation rec {
  pname = "xray_geoip";
  version = "202602012218";

  src = fetchurl {
    url = "https://github.com/Loyalsoldier/v2ray-rules-dat/releases/download/${version}/geoip.dat";
    sha256 = "sha256-hNm02ObgG4FeL5p9y5UNMK63OtwMc4MYj0jDS3QK0a4=";
  };

  unpackPhase = ":";

  installPhase = ''
    mkdir -p $out/bin
    cp $src $out/bin/geoip.dat
    runHook postInstall
  '';
}
