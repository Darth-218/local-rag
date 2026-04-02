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
    pkgs.glib
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

    # OCR support
    pkgs.tesseract  # Includes English language data
    pkgs.poppler-utils  # For pdftoppm (PDF to image conversion)
  ];

  LIBCLANG_PATH = "${pkgs.lib.makeLibraryPath [ pkgs.llvmPackages_latest.libclang.out ]}";

  GSETTINGS_SCHEMA_DIR = "${pkgs.gtk3}/share/gsettings-schemas/${pkgs.gtk3.name}/glib-2.0/schemas";

  shellHook = ''
    # Compile GTK schemas for Tauri file dialogs
    export GSETTINGS_SCHEMA_DIR="${pkgs.gtk3}/share/gsettings-schemas/${pkgs.gtk3.name}/glib-2.0/schemas"
    mkdir -p "$HOME/.local/share/glib-2.0/schemas"
    cp -r "$GSETTINGS_SCHEMA_DIR/"* "$HOME/.local/share/glib-2.0/schemas/" 2>/dev/null || true
    glib-compile-schemas "$HOME/.local/share/glib-2.0/schemas" 2>/dev/null || true

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
