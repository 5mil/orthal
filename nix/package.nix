{ lib, rustPlatform }:
rustPlatform.buildRustPackage {
  pname = "hybrid-node";
  version = "0.1.0";
  src = ../hybrid-chain;
  cargoLock.lockFile = ../hybrid-chain/Cargo.lock;
  doCheck = true;
  meta = {
    description = "Orthal hybrid PoW/PoS node (header v5, ASERT target)";
    homepage = "https://github.com/5mil/orthal";
    mainProgram = "hybrid-node";
    platforms = lib.platforms.unix;
  };
}
