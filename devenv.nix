{
  pkgs,
  lib,
  config,
  ...
}:
{
  cachix.enable = false;

  packages = [
    pkgs.openssl
    pkgs.pkg-config
  ];

  languages.rust.enable = true;

  services.postgres = {
    enable = true;
    initialDatabases = [
      {
        name = "ledgers";
        # initialSQL = "CREATE EXTENSION IF NOT EXISTS pg_uuidv7;";
      }
    ];
    extensions = extensions: [
      extensions.pg_uuidv7
    ];
  };

  dotenv.disableHint = true;

}
