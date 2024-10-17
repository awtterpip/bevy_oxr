{ lib
, rustPlatform
, fetchFromGitHub
, pkg-config
, bzip2
, xz
, zstd
, stdenv
, darwin
,
}:

rustPlatform.buildRustPackage rec {
  pname = "xbuild";
  version = "0.2.0";

  src = fetchFromGitHub {
    owner = "rust-mobile";
    repo = "xbuild";
    rev = "ffe3e34af6a34decd98222057b140b0fafb1b07a";
    hash = "sha256-60XHUkwGyTlold5/gNlbF3GIoe6nQMot9Twlt1+VSGA=";
  };

  cargoLock = {
    lockFile = ./Cargo.lock;
  };

  postPatch = ''
    ln -s ${./Cargo.lock} Cargo.lock
  '';

  nativeBuildInputs = [
    pkg-config
  ];

  buildInputs = [
    bzip2
    xz
    zstd
  ] ++ lib.optionals stdenv.isDarwin [
    darwin.apple_sdk.frameworks.Security
    darwin.apple_sdk.frameworks.SystemConfiguration
  ];

  env = {
    ZSTD_SYS_USE_PKG_CONFIG = true;
  };

  meta = {
    description = "Cross compile rust to any platform";
    homepage = "https://github.com/rust-mobile/xbuild.git";
    license = [ lib.licenses.mit lib.licenses.asl20 ];
    maintainers = with lib.maintainers; [ ];
    mainProgram = "xbuild";
  };
}
