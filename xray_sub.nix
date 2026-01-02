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
  cargoHash = "sha256-gF3i+OoI7vxaBe2jzTUOt+DAIuthGcoeYbYM5sanFr8=";
}
