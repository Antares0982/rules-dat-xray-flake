{
  stdenv,
  fetchurl,
  ...
}:
stdenv.mkDerivation rec {
  pname = "xray_geoip";
  version = "202606072259";

  src = fetchurl {
    url = "https://github.com/Loyalsoldier/v2ray-rules-dat/releases/download/${version}/geoip.dat";
    sha256 = "sha256-fA4ONx3erKpBA4lV1Q6oac9PsfPc3/D3HC8NbVn8zOg=";
  };

  unpackPhase = ":";

  installPhase = ''
    mkdir -p $out/bin
    cp $src $out/bin/geoip.dat
    runHook postInstall
  '';
}
