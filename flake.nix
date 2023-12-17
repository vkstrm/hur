{
  description = "hur, a HTTP request CLI tool";

  inputs = {
    nixpkgs.url = "github:NixOs/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system: 
      let 
        pkgs = nixpkgs.legacyPackages.${system};
        manifest = (pkgs.lib.importTOML ./Cargo.toml).package;
        # formatter.${system} = pkgs.nixpkgs-fmt;
        buildInputs = with pkgs; [ cargo rustc openssl ];
        nativeBuildInputs = with pkgs; [ pkg-config ];
      in 
      with pkgs;
      {
        devShells.default = mkShell {
          nativeBuildInputs = [ cargo rustc openssl pkg-config ];
        };

        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = manifest.name;
          version = manifest.version;
          src = self;
          buildInputs = [ openssl ];
          nativeBuildInputs = [ pkg-config ];
          cargoLock.lockFile = ./Cargo.lock;
        };
      }
    ); 
}
