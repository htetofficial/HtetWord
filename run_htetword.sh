#!/usr/bin/env bash
# HtetWord Launcher & Build Tool for macOS, Web, and Android
set -euo pipefail

echo "=========================================================="
echo "🌸 HtetWord - မြန်မာစာနှင့် စာပေအရေးအသား အက်ပ်"
echo "   (Floating & Liquid Glass UI | Pure Rust)"
echo "=========================================================="

MODE="${1:-run}"

case "$MODE" in
  run)
    echo "==> Starting HtetWord Native macOS Desktop App..."
    cargo run -p wordcraft -- --sample
    ;;
  build)
    echo "==> Compiling HtetWord Native Release Binary..."
    cargo build --release -p wordcraft
    echo "✔ Binary compiled successfully at: target/release/wordcraft"
    ;;
  web)
    echo "==> Launching HtetWord Web Development Server (WASM)..."
    cd apps/wordcraft-web
    if ! command -v trunk >/dev/null 2>&1; then
      echo "Trunk not found. Installing trunk via cargo..."
      cargo install trunk --locked
    fi
    trunk serve
    ;;
  web-build)
    echo "==> Building HtetWord Static Web & PWA Distribution..."
    cd apps/wordcraft-web
    trunk build --release
    echo "✔ Web files generated at: dist/web/"
    ;;
  android)
    echo "==> Android Package Instructions:"
    cat packaging/android/README.md
    ;;
  help|-h|--help)
    echo "Usage: ./run_htetword.sh [run | build | web | web-build | android]"
    echo "  run        - Run the native desktop app on macOS"
    echo "  build      - Compile the release desktop binary"
    echo "  web        - Serve the web/PWA version locally (for browser & mobile test)"
    echo "  web-build  - Build static web output (dist/web/)"
    echo "  android    - Display Android build and PWA installation guide"
    ;;
  *)
    echo "Unknown option: $MODE"
    echo "Run './run_htetword.sh --help' for available commands."
    exit 1
    ;;
esac
