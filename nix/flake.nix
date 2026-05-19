{
  description = "dn-kernel dev environment";

  inputs = {
nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
flake-utils.lib.eachDefaultSystem (system:
let
pkgs = import nixpkgs { inherit system; };
in {
devShells.default = pkgs.mkShell {
packages = [
pkgs.rustc
pkgs.cargo
pkgs.python311
pkgs.git
];
};
});
}
