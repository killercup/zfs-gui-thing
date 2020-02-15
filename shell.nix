{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    gtk3
    lldb_9
    xvfb_run # for testing on a console
  ];
}
