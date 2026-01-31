{
  description = "Claude Project Manager — TUI for managing Claude Code projects";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        packages.default = pkgs.stdenv.mkDerivation {
          pname = "claude-project-manager";
          version = "2.0.0";
          src = ./.;

          nativeBuildInputs = [ pkgs.makeWrapper ];

          # Runtime dependencies — shipped with the package
          buildInputs = [
            pkgs.bash
            pkgs.jq
            pkgs.fzf
            pkgs.gum
          ];

          installPhase = ''
            mkdir -p $out/bin $out/lib/claude-pm $out/share/doc/claude-pm

            # Install main CLI
            install -m 755 bin/claude-pm $out/bin/claude-pm
            install -m 755 bin/cpm $out/bin/cpm

            # Install library files
            cp lib/*.sh $out/lib/claude-pm/

            # Install templates
            mkdir -p $out/lib/claude-pm/templates
            cp templates/* $out/lib/claude-pm/templates/

            # Install docs
            cp README.md $out/share/doc/claude-pm/
            cp -r docs/* $out/share/doc/claude-pm/ 2>/dev/null || true

            # Patch the main script to find lib at $out
            substituteInPlace $out/bin/claude-pm \
              --replace 'SCRIPT_DIR="$(cd "$(dirname "''${BASH_SOURCE[0]}")/.." && pwd)"' \
                        "SCRIPT_DIR=\"$out\""

            # Wrap executables to ensure deps are on PATH
            wrapProgram $out/bin/claude-pm \
              --prefix PATH : ${pkgs.lib.makeBinPath [
                pkgs.jq
                pkgs.fzf
                pkgs.gum
                pkgs.coreutils
                pkgs.findutils
              ]}

            wrapProgram $out/bin/cpm \
              --prefix PATH : ${pkgs.lib.makeBinPath [
                pkgs.jq
                pkgs.fzf
                pkgs.gum
                pkgs.coreutils
                pkgs.findutils
              ]}
          '';

          meta = with pkgs.lib; {
            description = "TUI project manager for Claude Code sessions";
            homepage = "https://github.com/titus/claude-project-manager";
            license = licenses.mit;
            platforms = platforms.unix;
            mainProgram = "claude-pm";
          };
        };

        # Dev shell for contributing
        devShells.default = pkgs.mkShell {
          buildInputs = [
            pkgs.bash
            pkgs.jq
            pkgs.fzf
            pkgs.gum
            pkgs.bats
            pkgs.shellcheck
          ];
        };
      }
    );
}
