{
  system ? builtins.currentSystem,
  pkgs ? import <nixpkgs> { },
  cargo ? pkgs.cargo,
  collectd ? pkgs.collectd,
  ...
}:

with pkgs;
let
  rustPlatform = makeRustPlatform {
    rustc = cargo;
    cargo = cargo;
  };
  plugin = rustPlatform.buildRustPackage rec {
    name = "collectd-veconnect-${version}";
    version = "0.1.0";
    src = ./.;
    cargoSha256 = "0jacm96l1gw9nxwavqi1x4669cg6lzy9hr18zjpwlcyb3qkw9z7f";
    buildInputs = [ collectd ];
    CARGO_HOME = "$(mktemp -d cargo-home.XXX)";
    installPhase = ''
      mkdir -p $out/lib;
      cp target/release/libcollectd-veconnect.so $out/lib/veconnect.so
    '';

    meta = with lib; {
      homepage = https://github.com/kloenk/collectd-veconnect;
      description = "collectd plugin for ve Connect data bus";
      license = licenses.mit;
      # maintainers = with maintainers; [ kloenk ];
      platforms = with stdenv.lib.platforms; all;
    };
  };
  # doc
in {
  inherit rustPlatform plugin;
}
