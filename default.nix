{
  holonixPath ?  builtins.fetchTarball { url = "https://github.com/holochain/holonix/archive/a0dcdfac2c8783c58805175dd5bc5528ccbb35fd.tar.gz"; }
}:

let
  holonix = import (holonixPath) {
    include = {
        # making this explicit even though it's the default
        holochainBinaries = true;
    };

    holochainVersionId = "custom";

    holochainVersion = {
      rev = "holochain-0.0.107";
      sha256 = "1m5clhh0xpr4ajdbybxjqc5vblkd30lsfb1sac4zbzxjrnpp5iki";
      cargoSha256 = "175b76j31sls0gj08imchwnk7n4ylsxlc1bm58zrhfmq62hcchb1";
      bins = {
        holochain = "holochain";
        hc = "hc";
      };

      lairKeystoreHashes = {
        sha256 = "0khg5w5fgdp1sg22vqyzsb2ri7znbxiwl7vr2zx6bwn744wy2cyv";
        cargoSha256 = "1lm8vrxh7fw7gcir9lq85frfd0rdcca9p7883nikjfbn21ac4sn4";
      };
    };
  };
  nixpkgs = holonix.pkgs;
in nixpkgs.mkShell {
  inputsFrom = [ holonix.main ];
  buildInputs = with nixpkgs; [
    binaryen
    nodejs-16_x
  ];
}
