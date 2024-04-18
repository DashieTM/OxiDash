{ pkgs ? import <nixpkgs> {} }:

with pkgs;
mkShell {
  nativeBuildInputs = [
    pkg-config
  ];

  buildInputs = [
    dbus
    gtk3
    gtk-layer-shell
  ];

}
