#!/bin/sh
# Makes everything under apps/ios that is not committed: the XCFramework of the C
# bindings, the bundled story and its license, and the Xcode project.
# Run from anywhere; safe to run again.
set -eu

root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root"

missing=0
if ! command -v xcodegen >/dev/null 2>&1; then
    echo "xcodegen is not installed. Install it with: brew install xcodegen" >&2
    missing=1
fi
if ! command -v xcodebuild >/dev/null 2>&1; then
    echo "xcodebuild is not available. Install Xcode and select it with xcode-select." >&2
    missing=1
fi
for target in aarch64-apple-ios-sim aarch64-apple-ios; do
    if ! rustup target list --installed | grep -qx "$target"; then
        echo "The Rust target $target is not installed. Install it with: rustup target add $target" >&2
        missing=1
    fi
done
[ "$missing" -eq 0 ] || exit 1

for target in aarch64-apple-ios-sim aarch64-apple-ios; do
    cargo build --release -p zmachine-ffi --target "$target"
    echo "Built the bindings for $target"
done

framework=apps/ios/Frameworks/ZMX.xcframework
rm -rf "$framework"
mkdir -p apps/ios/Frameworks
xcodebuild -create-xcframework \
    -library target/aarch64-apple-ios-sim/release/libzmachine_ffi.a -headers crates/zmachine-ffi/include \
    -library target/aarch64-apple-ios/release/libzmachine_ffi.a -headers crates/zmachine-ffi/include \
    -output "$framework" >/dev/null
echo "Made $framework"

story=stories/zork1/zork1.z3
recorded=$(sed -n 's/^| SHA-256 | `\([0-9a-f]\{64\}\)` |$/\1/p' stories/zork1/SOURCE.md)
actual=$(shasum -a 256 "$story" | cut -d' ' -f1)
if [ -z "$recorded" ] || [ "$recorded" != "$actual" ]; then
    echo "$story does not match the digest recorded in stories/zork1/SOURCE.md" >&2
    echo "  recorded: ${recorded:-none found}" >&2
    echo "  actual:   $actual" >&2
    exit 1
fi
resources=apps/ios/Lamplight/Resources
mkdir -p "$resources"
cp "$story" "$resources/story.z3"
# The About sheet shows where the story came from, then its license.
source=$(sed -n 's/^| Source | <\(.*\)>, path `\(.*\)` |$/Source: \1 (\2)/p' stories/zork1/SOURCE.md)
{ echo "$source"; echo; cat stories/zork1/LICENSE; } > "$resources/story-license.txt"
echo "Copied the story (digest verified), its source and its license to $resources"
# The story's game pack (docs/introspection.md, section 12), if it has one. It is
# copied only if it names this story file; the layer checks the same again.
pack=packs/zork1.pack
rm -f "$resources/story.pack"
if [ -f "$pack" ]; then
    if ! grep -qx "sha256: $actual" "$pack"; then
        echo "$pack does not name $story" >&2
        exit 1
    fi
    cp "$pack" "$resources/story.pack"
    echo "Copied the story's game pack to $resources"
fi

(cd apps/ios && xcodegen generate --quiet)
echo "Generated apps/ios/Lamplight.xcodeproj"
