#!/bin/sh
# Checks scripts/fetch-corpus.sh without the network: it serves a temporary folder
# on this machine and fetches from that (specs/005-corpus-testing/tasks.md T035).
set -eu

here=$(cd "$(dirname "$0")" && pwd)
work=$(mktemp -d)
server=
trap '[ -z "$server" ] || { kill "$server" 2>/dev/null; wait "$server" 2>/dev/null; } || true; rm -rf "$work"' EXIT

mkdir -p "$work/served" "$work/root/stories/corpus"
printf '\003a story of sorts' > "$work/served/right.z3"
printf '\003some other file' > "$work/served/other.z3"
right=$(shasum -a 256 "$work/served/right.z3" | awk '{print $1}')

port=$(python3 -I -c 'import socket; s = socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1])')
(cd "$work/served" && exec python3 -I -m http.server "$port" --bind 127.0.0.1 >/dev/null 2>&1) &
server=$!
for _ in 1 2 3 4 5 6 7 8 9 10; do
    curl -fs -o /dev/null "http://127.0.0.1:$port/right.z3" && break
    sleep 0.3
done

entry() {
    printf 'id: %s\nsha256: %s\nsize: 17\nversion: 3\nfamily: none\nsource: http://127.0.0.1:%s/%s\nstanding: a test\ntier: %s\npath: %s\nseed: 1\n\n' \
        "$1" "$right" "$port" "$2" "$3" "$4"
}
{
    echo "# a test's manifest"
    entry good right.z3 fetched stories/local/corpus/good.z3
    entry bad other.z3 fetched stories/local/corpus/bad.z3
    entry kept right.z3 committed stories/kept/kept.z3
    entry gone nosuch.z3 fetched stories/local/corpus/gone.z3
} > "$work/root/stories/corpus/manifest.txt"

export CORPUS_ROOT="$work/root"
fail() { echo "FAILED: $1" >&2; exit 1; }
expect() { [ "$1" = "$2" ] || fail "$3: got '$1', expected '$2'"; }

out=$("$here/fetch-corpus.sh" good)
expect "$out" "good: fetched to stories/local/corpus/good.z3" "a right digest"
[ -f "$work/root/stories/local/corpus/good.z3" ] || fail "the file was not kept"

out=$("$here/fetch-corpus.sh" good)
expect "$out" "good: already present" "a present file"

if out=$("$here/fetch-corpus.sh" bad 2>&1); then fail "a wrong digest was accepted"; fi
case "$out" in "bad: refused: "*) ;; *) fail "a wrong digest: $out" ;; esac
[ ! -e "$work/root/stories/local/corpus/bad.z3" ] || fail "a refused file was kept"
[ ! -e "$work/root/stories/local/corpus/bad.z3.part" ] || fail "a refused download was left"

if out=$("$here/fetch-corpus.sh" kept 2>&1); then fail "a committed story was fetched"; fi
expect "$out" "kept: a committed story is not fetched" "a committed story"

if out=$("$here/fetch-corpus.sh" gone 2>&1); then fail "a missing file was fetched"; fi
case "$out" in *"gone: could not be downloaded from "*) ;; *) fail "a missing file: $out" ;; esac
[ ! -e "$work/root/stories/local/corpus/gone.z3" ] || fail "a failed download was kept"

if out=$("$here/fetch-corpus.sh" nosuch 2>&1); then fail "an unknown id was accepted"; fi
expect "$out" "nosuch: not in the manifest" "an unknown id"

if "$here/fetch-corpus.sh" >/dev/null 2>&1; then fail "no arguments were accepted"; fi
if "$here/fetch-corpus.sh" --all >/dev/null 2>&1; then fail "there is an 'all'"; fi

out=$("$here/fetch-corpus.sh" --propose "http://127.0.0.1:$port/right.z3" | sed 1d)
expect "$out" "name:    right.z3
size:    17
sha256:  $right
version: 3" "a proposal"

echo "fetch-corpus: 10 checks passed"
