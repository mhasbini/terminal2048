{ nixpkgs ? import <nixpkgs> { } }:
 
let
  install_packages = [
    nixpkgs.rustup
  ];


in
  nixpkgs.stdenv.mkDerivation {
    name = "terminal2048";
    buildInputs = install_packages;
  }
