{
  description = "NixOS control plane evaluation fixture";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs =
    { nixpkgs, ... }:
    {
      nixosConfigurations.fixture = nixpkgs.lib.nixosSystem {
        system = "x86_64-linux";
        modules = [
          (
            { pkgs, ... }:
            {
              boot.loader.grub.enable = false;
              fileSystems."/" = {
                device = "/dev/disk/by-label/nixos";
                fsType = "ext4";
              };
              networking.hostName = "fixture";
              environment.systemPackages = [
                pkgs.curl
                pkgs.ripgrep
              ];
              services.openssh.enable = true;
              system.stateVersion = "26.05";
            }
          )
        ];
      };
    };
}
