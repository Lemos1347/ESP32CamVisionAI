{ pkgs }:
pkgs.mkShell {
  name = "ESP Dev Environment";

  # Common build inputs for all platforms
  buildInputs =
    with pkgs;
    [
      rustup
      neovim
      clang
      libiconv
      espup
      ldproxy
      espflash
      just
    ]
    ++ (
      if pkgs.stdenv.isDarwin then
        [
          pkgs.darwin.apple_sdk.frameworks.SystemConfiguration # macOS-specific package
        ]
      else
        [ ]
    );

  # Commands to run when the shell is activated
  shellHook = ''
    # Getting rustup home dir
    export RUSTUP_HOME=$(rustup show | grep 'rustup home' | awk '{print $3}')

    # ESP setup
    test -f $RUSTUP_HOME/export-esp.sh || (espup install && mv $HOME/export-esp.sh $RUSTUP_HOME/)
      . $RUSTUP_HOME/export-esp.sh

      # Generate config file if it not exists
      test ./embedded/cfg.toml || cp ./embedded/cfg.toml.example ./embedded/cfg.toml

      echo "Development environment ready!"
      echo "Please update file ./embedded/cfg.toml with your wifi credentials"
      echo "After that, just run:"
      echo "just esp32cam"
  '';
}
