{
  pkgs,
}:

pkgs.stdenv.mkDerivation {
  pname = "esp32cam-rs";
  version = "0.1.0";

  # The same build inputs from your dev shell
  nativeBuildInputs =
    with pkgs;
    [
      rustup
      clang
      libiconv
      espup
      ldproxy
      espflash
      just
      git
      openssl
      cacert
      python3
    ]
    ++ (if pkgs.stdenv.isDarwin then [ pkgs.darwin.apple_sdk.frameworks.SystemConfiguration ] else [ ]);

  # Source code directory
  src = ../../embedded;

  buildPhase = ''
    echo "Building the ESP32 app..."
    export RUSTUP_HOME=$TMPDIR/.rustup
    export CARGO_HOME=$TMPDIR/.cargo
    export ESPUP_HOME=$TMPDIR/.espup
    export HOME=$TMPDIR/home
    export CARGO_TARGET_DIR=$TMPDIR/target
    mkdir -p $ESPUP_HOME $RUSTUP_OME $CARGO_HOME $HOME $CARGO_TARGET_DIR

    espup install --export-file $ESPUP_HOME/export-esp.sh
  '';

  installPhase = ''
    mkdir -p $out/bin
    . $ESPUP_HOME/export-esp.sh
  '';

  meta = with pkgs.lib; {
    description = "Package to build the ESP32CamVisionAI app with configuration prompt.";
    license = licenses.mit;
    platforms = platforms.all;
  };
}
