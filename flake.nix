{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs =
    { nixpkgs, ... }:
    let
      forAllSystems = nixpkgs.lib.genAttrs [
        "aarch64-linux"
        "x86_64-linux"
        "aarch64-darwin"
      ];
    in
    {
      devShells = forAllSystems (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
        in
        with pkgs;
        {
          default = mkShell {
            nativeBuildInputs = [ pkg-config ];
            buildInputs =
              [
                rustup
              ]
              ++ (
                if stdenv.isLinux then
                  [
                    xorg.libX11
                    xorg.libXcursor
                    xorg.libXrandr
                    xorg.libXi
                    xorg.libXtst
                    libevdev
                    libnotify
                    libinput
                    libxkbcommon
                    udev
                  ]
                else
                  [ ]
              );
          };

        }
      );
    };
}
