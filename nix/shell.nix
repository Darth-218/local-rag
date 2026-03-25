{ pkgs ? import <nixpkgs> { } }:

pkgs.mkShell {
  name = "local-rag";

  packages = [
    # Rust toolchain
    pkgs.rustc
    pkgs.cargo
    pkgs.clippy
    pkgs.rustfmt
    pkgs.rust-analyzer

    # Node.js ecosystem (npm is included with nodejs)
    pkgs.nodejs_22
    pkgs.pnpm

    # Tauri system dependencies (Linux)
    pkgs.pkg-config
    pkgs.glib.dev
    pkgs.gtk3
    pkgs.webkitgtk_4_1
    pkgs.libsoup_3
    pkgs.dbus
    pkgs.libsecret
    pkgs.openssl
    pkgs.cairo
    pkgs.pango
    pkgs.gdk-pixbuf
    pkgs.libsass
    pkgs.sassc
    pkgs.gcc
    pkgs.gnumake
    pkgs.patchelf

    # Version control
    pkgs.git

    # Utilities
    pkgs.curl
    pkgs.wget
  ];

  LIBCLANG_PATH = "${pkgs.lib.makeLibraryPath [ pkgs.llvmPackages_latest.libclang.out ]}";

  shellHook = ''
    echo "═══════════════════════════════════════════════════════"
    echo "  local-rag development environment"
    echo "═══════════════════════════════════════════════════════"
    echo "  Node: $(node --version)"
    echo "  npm:  $(npm --version)"
    echo "  Rust: $(rustc --version)"
    echo "  Cargo: $(cargo --version)"
    echo "═══════════════════════════════════════════════════════"
    echo ""
    echo "Next steps:"
    echo "  1. npm install"
    echo "  2. cd src-tauri && cargo build"
    echo "  3. npm run tauri dev"
    echo ""
  '';
}
