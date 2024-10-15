{
  description = "Flake for ESP32CamVisionAI with flake-utils for cross-platform support";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils"; # Add flake-utils for simplifying platform management
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachSystem
      [
        "x86_64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
        "aarch64-linux"
      ]
      (
        system:
        let
          pkgs = import nixpkgs { inherit system; };
        in
        {
          devShells = {
            # Development shell for ESP32
            esp = pkgs.mkShell {
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
            };

            # Additional shell could be defined here, like another devShell (e.g., "tools" or "build")
            # Example of a shell for building or testing purposes
            # tools = pkgs.mkShell {
            #   name = "Tools Dev Environment";
            #
            #   buildInputs = with pkgs; [
            #     rustc
            #     cargo
            #     clang
            #     cmake
            #     ninja
            #     git
            #   ];
            #
            #   shellHook = ''
            #     echo "Tools development environment ready!";
            #     # Here you can define any setup specific to this environment
            #   '';
            # };
          };
        }
      );
}
