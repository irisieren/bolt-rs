{ pkgs ? import <nixpkgs> {} }:

pkgs.llvmPackages.stdenv.mkDerivation {
  name = "env";

  buildInputs = with pkgs; [
    llvmPackages.libclang
    llvmPackages.lld
    llvmPackages.bintools
    clang
    cmake
  ];

  LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
}
