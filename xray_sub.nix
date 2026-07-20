{
  pkgs,
  rustPlatform,
  lib,
  ...
}:
rustPlatform.buildRustPackage rec {
  pname = "xray-sub";
  version = "0.1.0";
  src = builtins.path {
    name = pname;
    path = ./xray_sub;
  };
  cargoHash = "sha256-2uH1nzG/tHeceioMFSS9eWs3kGlA68ZjxqatxCK/a54=";
}
