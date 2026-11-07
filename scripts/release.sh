#!/bin/bash
# Build, sign, notarize and package this checkout; never launches the app.
set -euo pipefail
cd "$(dirname "$0")/.."
: "${SIGNING_IDENTITY:?Set SIGNING_IDENTITY to your Developer ID Application identity}"
: "${NOTARY_PROFILE:?Set NOTARY_PROFILE to a notarytool keychain profile}"
export MACOSX_DEPLOYMENT_TARGET=13.0
cargo build --release --locked
version=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
arch=$(uname -m)
out=$(mktemp -d "$PWD/dist-release.XXXXXX")
app="$out/quiet.md.app"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
cp target/release/quiet_md "$app/Contents/MacOS/"
cp LICENSE THIRD_PARTY.md "$app/Contents/Resources/"
icons="$out/AppIcon.iconset"
mkdir -p "$icons"
for size in 16 32 128 256 512; do
  sips -z "$size" "$size" assets/branding/app-icon.png --out "$icons/icon_${size}x${size}.png" >/dev/null
  double=$((size * 2))
  sips -z "$double" "$double" assets/branding/app-icon.png --out "$icons/icon_${size}x${size}@2x.png" >/dev/null
done
iconutil -c icns "$icons" -o "$app/Contents/Resources/AppIcon.icns"
cat > "$app/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleName</key><string>Quiet.md</string>
<key>CFBundleDisplayName</key><string>Quiet.md</string>
<key>CFBundleIdentifier</key><string>com.s1dashu.quietmd</string>
<key>CFBundleExecutable</key><string>quiet_md</string>
<key>CFBundleIconFile</key><string>AppIcon</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>CFBundleShortVersionString</key><string>$version</string>
<key>CFBundleVersion</key><string>$version</string>
<key>LSMinimumSystemVersion</key><string>13.0</string>
<key>NSHighResolutionCapable</key><true/>
<key>NSDocumentsFolderUsageDescription</key><string>Quiet.md 将笔记保存在文稿中的 Quiet.md 文件夹。</string>
</dict></plist>
PLIST
/usr/libexec/PlistBuddy -c 'Merge packaging/macos-document-types.plist' "$app/Contents/Info.plist"
plutil -lint "$app/Contents/Info.plist"
codesign --force --options runtime --timestamp --sign "$SIGNING_IDENTITY" "$app"
codesign --verify --deep --strict --verbose=2 "$app"
ditto -c -k --keepParent "$app" "$out/notarization.zip"
xcrun notarytool submit "$out/notarization.zip" --keychain-profile "$NOTARY_PROFILE" --wait --output-format json > "$out/notarization.json"
python3 - "$out/notarization.json" <<'PYTHON'
import json, sys
result = json.load(open(sys.argv[1]))
if result.get("status") != "Accepted":
    raise SystemExit("Notarization was not accepted; see " + sys.argv[1])
print("Notarization accepted:", result["id"])
PYTHON
xcrun stapler staple "$app"
xcrun stapler validate "$app"
spctl --assess --type execute --verbose=2 "$app"
archive="$out/quiet-md-$version-macos-$arch.zip"
ditto -c -k --keepParent "$app" "$archive"
(cd "$out" && shasum -a 256 "$(basename "$archive")" > "$(basename "$archive").sha256")
printf 'Release artifact: %s\n' "$archive"
