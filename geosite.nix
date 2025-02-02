{
  stdenv,
  fetchurl,
  ...
}:
stdenv.mkDerivation rec {
  pname = "xray_geosite";
  version = "202502012211";

  src = fetchurl {
    url = "https://github.com/Loyalsoldier/v2ray-rules-dat/releases/download/${version}/geosite.dat";
    sha256 = "sha256-ZkAoVM/o4GG8tWgg8DgESU8DnjJ+ep4lfZWFFYbDEY0=";
  };

  unpackPhase = ":";

  installPhase = ''
    mkdir -p $out/bin
    cp $src $out/bin/geosite.dat
    runHook postInstall
  '';
}
