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
            esp = import ./nix/shells/esp-shell.nix {
              inherit pkgs;
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
