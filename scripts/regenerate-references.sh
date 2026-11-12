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

# Reference saves and the transcripts of their continuations. For each save point n,
# make_n.txt plays the walkthrough up to that point and saves as save_n.qzl;
# cont_n.txt, in a fresh run, restores save_n.qzl and plays on, writing cont_n.scr.
for make in "$fid"/saves/make_*.txt; do
    n=$(basename "$make" .txt | sed 's/^make_//')
    (cd "$work" && dfrotz -m -q -w 80 -S 0 -s "$seed" "$story" < "$make" >/dev/null 2>&1) || true
    cp "$work/save_$n.qzl" "$fid/saves/frotz_$n.qzl"
    (cd "$work" && dfrotz -m -q -w 80 -S 0 -s "$seed" "$story" < "$fid/saves/cont_$n.txt" >/dev/null 2>&1) || true
    cp "$work/cont_$n.scr" "$fid/saves/cont_$n.scr"
    echo "wrote save point $n"
done

# ---- Advent, versions 5 and 8 (feature 002) ----
adv="$root/stories/advent"
afid="$adv/fidelity"
aseed=$(cat "$afid/seed.txt")
for ver in 5 8; do
    astory="$adv/advent.z$ver"
    w=$(mktemp -d)
    name=$(sed -n '2p' "$afid/walkthrough.txt")
    (cd "$w" && dfrotz -m -q -w 80 -S 0 -s "$aseed" "$astory" < "$afid/walkthrough.txt" >/dev/null 2>&1) || true
    cp "$w/$name.scr" "$afid/reference_z$ver.scr"
    (cd "$w" && dfrotz -m -q -w 80 -S 0 -s "$aseed" "$astory" < "$afid/undo.txt" >/dev/null 2>&1) || true
    cp "$w/undo.scr" "$afid/undo_z$ver.scr"
    for make in "$afid"/saves/make_*.txt; do
        n=$(basename "$make" .txt | sed 's/^make_//')
        rm -f "$w/save_$n.qzl"
        (cd "$w" && dfrotz -m -q -w 80 -S 0 -s "$aseed" "$astory" < "$make" >/dev/null 2>&1) || true
        cp "$w/save_$n.qzl" "$afid/saves/frotz_z${ver}_$n.qzl"
        (cd "$w" && dfrotz -m -q -w 80 -S 0 -s "$aseed" "$astory" < "$afid/saves/cont_$n.txt" >/dev/null 2>&1) || true
        cp "$w/cont_$n.scr" "$afid/saves/cont_z${ver}_$n.scr"
    done
    rm -rf "$w"
    echo "wrote Advent version $ver references"
done
