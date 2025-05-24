{config, lib, pkgs, ... }:

let
  cfg = config.services.omen-fan;
  omen-fan = pkgs.rustPlatform.buildRustPackage rec {
    pname = "omen-fan";
    version = "0.1.4";

    src = pkgs.fetchFromGitHub {
      owner = "koevoet1221";
      repo = "omen-fan";
      rev = "main";
      sha256 = "sha256-aVrSgZ5yAJStcYyJhYw0xNkpS0B1utrcyIJnfPmyBec=";
    };

    sourceRoot = "source/omen-fan";

    cargoHash = "sha256-4O7sTHw7cREESd1kd3UmeTZpJ1IfTxgDz4fMLqKZctI=";

    nativeBuildInputs = [ pkgs.pkg-config ];
    buildInputs = [ pkgs.systemd ];

    meta = with lib; {
      description = "Utility to control fans in HP Omen laptops";
      homepage = "https://github.com/koevoet1221/omen-fan";
      license = licenses.mit;
    };
  };
in {
  options.services.omen-fan = {
    enable = lib.mkEnableOption "HP Omen fan control service";
  };

  config = lib.mkIf cfg.enable {
    systemd.services.omen-fan = {
      description = "Omen Fan Control Service";
      wantedBy = [ "multi-user.target" ];
      serviceConfig = {
        ExecStart = "${omen-fan}/bin/omen-fan";
        Environment = "PATH=/run/current-system/sw/bin";
        Restart = "always";
      };
    };
  };
}
