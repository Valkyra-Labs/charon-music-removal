#!/bin/sh
# Build "Charon Music Removal.app" and a DMG, ad-hoc signed (not notarized).
# Usage: tools/bundle.sh PATH/TO/tiger_music.onnx
set -eu
MODEL=${1:?path to tiger_music.onnx}
ROOT=$(cd "$(dirname "$0")/.." && pwd)
VERSION=$(grep '^version' "$ROOT/Cargo.toml" | head -1 | cut -d'"' -f2)
OUT="$ROOT/target/bundle"
APP="$OUT/Charon Music Removal.app"

cargo build --release --manifest-path "$ROOT/Cargo.toml"
rm -rf "$OUT"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources/licenses"
cp "$ROOT/target/release/charon-music-removal" "$APP/Contents/MacOS/"
cp "$MODEL" "$APP/Contents/Resources/tiger_music.onnx"
cp "$ROOT"/licenses/* "$APP/Contents/Resources/licenses/"
cp "$ROOT/THIRD_PARTY.md" "$APP/Contents/Resources/licenses/"
cp "$ROOT/LICENSE.md" "$APP/Contents/Resources/licenses/charon-music-removal-LICENSE.md"
cat > "$APP/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>Charon Music Removal</string>
  <key>CFBundleDisplayName</key><string>Charon Music Removal</string>
  <key>CFBundleIdentifier</key><string>com.valkyra-labs.charon-music-removal</string>
  <key>CFBundleExecutable</key><string>charon-music-removal</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>$VERSION</string>
  <key>CFBundleVersion</key><string>$VERSION</string>
  <key>LSMinimumSystemVersion</key><string>13.0</string>
  <key>LSArchitecturePriority</key><array><string>arm64</string></array>
  <key>NSHighResolutionCapable</key><true/>
  <key>CFBundleDocumentTypes</key>
  <array><dict>
    <key>CFBundleTypeRole</key><string>Viewer</string>
    <key>LSItemContentTypes</key><array><string>public.movie</string></array>
  </dict></array>
</dict>
</plist>
PLIST
codesign --force --deep --sign - "$APP"
DMG="$OUT/Charon-Music-Removal-$VERSION-arm64.dmg"
hdiutil create -quiet -volname "Charon Music Removal" -srcfolder "$APP" -ov -format UDZO "$DMG"
shasum -a 256 "$DMG"
du -sh "$APP" "$DMG"
