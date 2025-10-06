{
  description = "Cleanup old nix generations";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    crane.url = "github:ipetkov/crane";
    cf.url = "github:jzbor/cornflakes";
  };

  outputs = { self, nixpkgs, cf, crane, ... }: ((cf.mkLib nixpkgs).flakeForDefaultSystems (system:
  let
    pkgs = nixpkgs.legacyPackages.${system};
    craneLib = crane.mkLib pkgs;
  in {
    packages.default = craneLib.buildPackage rec {
      # srcFilter = path: type: (builtins.match ".*pest$" path != null) || (craneLib.filterCargoSources path type);
      src = nixpkgs.lib.cleanSourceWith {
        src = ./.;
        # filter = srcFilter;
        name = "source";
      };
      strictDeps = true;

      cargoArtifacts = craneLib.buildDepsOnly {
        inherit src strictDeps;
      };

      nativeBuildInputs = with pkgs; [
        makeWrapper
        installShellFiles
      ];
      postFixup = ''
        wrapProgram $out/bin/toro \
          --prefix PATH ${pkgs.lib.makeBinPath [ pkgs.git pkgs.openssh]}
      '';
      postInstall = ''
        echo "Generating man pages"
        mkdir ./manpages
        $out/bin/toro man ./manpages
        installManPage ./manpages/*

        echo "Generating shell completions"
        mkdir ./completions
        $out/bin/toro completions ./completions
        installShellCompletion completions/toro.{bash,fish,zsh}
      '';
    };

    devShells.default = craneLib.devShell {
      inherit (self.packages.${system}.default) name;

      # Additional tools
      nativeBuildInputs = [];
    };
  }));
}
