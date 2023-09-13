{ pkgs ? import (import ./nix/sources.nix { }).nixpkgs { } }:

pkgs.mkShell {
  buildInputs = with pkgs; [ rustc cargo niv ];
  nativeBuildInputs = with pkgs; [ pkg-config gtk4 gtk4-layer-shell ];
}
