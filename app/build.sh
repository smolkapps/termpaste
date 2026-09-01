#!/bin/bash
# Build TermPaste.app (menu-bar app) from TermPaste.swift, bundling the termpaste CLI.
# Requires: macOS with Xcode command-line tools (swiftc) and the termpaste binary
# either at ~/.cargo/bin/termpaste (cargo install --path ..) or on PATH.
set -euo pipefail
cd "$(dirname "$0")"

APP="TermPaste.app"
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS"

cat > "$APP/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>TermPaste</string>
  <key>CFBundleIdentifier</key><string>com.smolkapps.termpaste</string>
  <key>CFBundleVersion</key><string>1</string>
  <key>CFBundleShortVersionString</key><string>0.1.0</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleExecutable</key><string>TermPaste</string>
  <key>LSUIElement</key><true/>
  <key>LSMinimumSystemVersion</key><string>12.0</string>
</dict>
</plist>
PLIST

echo "Compiling TermPaste.swift ..."
swiftc -O -o "$APP/Contents/MacOS/TermPaste" TermPaste.swift -framework AppKit

# Bundle the termpaste CLI so the app is self-contained.
if [ -x "$HOME/.cargo/bin/termpaste" ]; then
  cp "$HOME/.cargo/bin/termpaste" "$APP/Contents/MacOS/termpaste"
  echo "Bundled termpaste from ~/.cargo/bin."
elif command -v termpaste >/dev/null 2>&1; then
  cp "$(command -v termpaste)" "$APP/Contents/MacOS/termpaste"
  echo "Bundled termpaste from PATH."
else
  echo "WARNING: termpaste binary not found — run 'cargo install --path ..' first." >&2
fi

echo "Built $(pwd)/$APP"
echo "Run it with:  open $APP"
