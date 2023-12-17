{ buildRustPackage }:
  
{  
  hur = buildRustPackage {
    pname = "hur";
    src = "./.";

    cargoLock = {
      lockFile = ./Cargo.lock;
    };
  };
}