{
  description = "Linux input method switcher";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs =
    { nixpkgs, ... }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];

      forAllSystems = nixpkgs.lib.genAttrs systems;

      manifest = with builtins; fromTOML (readFile ./Cargo.toml);
    in
    {
      packages = forAllSystems (
        system:
        let
          pkgs = import nixpkgs { inherit system; };
          lib = pkgs.lib;
        in
        {
          default = pkgs.pkgsStatic.rustPlatform.buildRustPackage {
            inherit (manifest.package) version;
            pname = manifest.package.name;
            src = lib.cleanSource ./.;
            cargoLock.lockFile = ./Cargo.lock;

            nativeInstallCheckInputs = [ pkgs.versionCheckHook ];

            meta = {
              description = "Linux input method switcher";
              license = lib.licenses.mit;
              mainProgram = "limes";
              platforms = lib.platforms.linux;
            };
          };
        }
      );

      devShells = forAllSystems (
        system:
        let
          pkgs = import nixpkgs { inherit system; };
        in
        {
          default = pkgs.mkShell {
            packages = with pkgs; [
              cargo
              clippy
              rust-analyzer
              rustc
              rustfmt
              systemd
            ];
          };
        }
      );
    };
}
