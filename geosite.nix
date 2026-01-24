{
  stdenv,
  fetchurl,
  ...
}:
stdenv.mkDerivation rec {
  pname = "xray_geosite";
  version = "202601202217";

  src = fetchurl {
    url = "https://github.com/Loyalsoldier/v2ray-rules-dat/releases/download/${version}/geosite.dat";
    # expected: sha256 = "sha256-VwV0jJsr+jMk5NFzct3AFP/IHO2x1ZomTVHSaWIeg7s=";
    # not: sha256 = "sha256-0CzZ0FxdRivoIbwGfWwPsXPyCDzvBmQq0hrHxyeZ1N0=";
    sha256 = "sha256-gGOPZn+hOsUSOSSFxTLbrPyUQO9m7j5La2mF7RtXoAg=";
  };

  unpackPhase = ":";

  installPhase = ''
    mkdir -p $out/bin
    cp $src $out/bin/geosite.dat
    runHook postInstall
  '';
}
