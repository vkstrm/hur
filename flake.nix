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
      in 
      with pkgs;
      {
        # formatter.${system} = nixpkgs-fmt;

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
