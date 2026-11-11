#!/bin/sh
# Rebuilds the reference artefacts under stories/zork1/fidelity/ with dfrotz.
# The references are only meaningful against one known version of Frotz, so this
# refuses to run with any other.
set -eu

REQUIRED="FROTZ V2.55"
root=$(cd "$(dirname "$0")/.." && pwd)
story="$root/stories/zork1/zork1.z3"
fid="$root/stories/zork1/fidelity"

if ! command -v dfrotz >/dev/null 2>&1; then
    echo "dfrotz not found; install Frotz 2.55 (brew install frotz)" >&2
    exit 1
fi
version=$(dfrotz -v 2>&1 | head -1)
case "$version" in
    "$REQUIRED"*) ;;
    *) echo "need $REQUIRED, found: $version" >&2; exit 1 ;;
esac

if [ ! -f "$fid/walkthrough.txt" ] || [ ! -f "$fid/seed.txt" ]; then
    echo "no walkthrough yet under $fid" >&2
    exit 1
fi
seed=$(cat "$fid/seed.txt")

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

# The walkthrough's second line names the transcript file; Frotz appends .scr.
name=$(sed -n '2p' "$fid/walkthrough.txt")
(cd "$work" && dfrotz -m -q -w 80 -S 0 -s "$seed" "$story" < "$fid/walkthrough.txt" >/dev/null 2>&1) || true
cp "$work/$name.scr" "$fid/reference.scr"
echo "wrote $fid/reference.scr"
