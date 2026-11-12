#!/bin/sh
# Fetches stories of the corpus that are not committed, by name, into the folder
# the repository ignores (specs/005-corpus-testing/contracts/commands.md).
#
#   scripts/fetch-corpus.sh ID...
#   scripts/fetch-corpus.sh --propose ADDRESS
#
# Every story fetched is named by the person running this: there is no "all".
# It downloads from the address the manifest records and nowhere else, and keeps
# a file only if its SHA-256 is the manifest's. No test and no CI job runs it.
set -eu

root=${CORPUS_ROOT:-$(cd "$(dirname "$0")/.." && pwd)}
manifest="$root/stories/corpus/manifest.txt"

usage() {
    echo "usage: scripts/fetch-corpus.sh ID..." >&2
    echo "       scripts/fetch-corpus.sh --propose ADDRESS" >&2
    exit 2
}

for tool in curl shasum awk; do
    command -v "$tool" >/dev/null 2>&1 || { echo "fetch-corpus: $tool is needed" >&2; exit 1; }
done

digest() {
    shasum -a 256 "$1" | awk '{print $1}'
}

[ $# -ge 1 ] || usage

if [ "$1" = "--propose" ]; then
    [ $# -eq 2 ] || usage
    dir=$(mktemp -d)
    name=$(basename "$2")
    curl -fsSL -A curl -o "$dir/$name" "$2"
    echo "file:    $dir/$name"
    echo "name:    $name"
    echo "size:    $(wc -c < "$dir/$name" | tr -d ' ')"
    echo "sha256:  $(digest "$dir/$name")"
    echo "version: $(od -An -tu1 -N1 "$dir/$name" | tr -d ' ')"
    exit 0
fi

case "$1" in -*) usage ;; esac

# One value of one entry of the manifest.
value() {
    awk -v id="$1" -v key="$2" '
        /^#/ { next }
        /^[ \t]*$/ { if (found) exit; next }
        $0 == "id: " id { found = 1 }
        found && index($0, key ": ") == 1 { print substr($0, length(key) + 3); exit }
    ' "$manifest"
}

status=0
for id in "$@"; do
    tier=$(value "$id" tier)
    if [ -z "$tier" ]; then
        echo "$id: not in the manifest" >&2
        status=1
        continue
    fi
    if [ "$tier" != "fetched" ]; then
        echo "$id: a committed story is not fetched" >&2
        status=1
        continue
    fi
    address=$(value "$id" source | awk '{print $1}')
    path=$(value "$id" path)
    want=$(value "$id" sha256)
    case "$path" in
        stories/local/corpus/*) ;;
        *) echo "$id: its path is not under stories/local/corpus/" >&2; status=1; continue ;;
    esac
    file="$root/$path"
    if [ -f "$file" ] && [ "$(digest "$file")" = "$want" ]; then
        echo "$id: already present"
        continue
    fi
    mkdir -p "$(dirname "$file")"
    if ! curl -fsSL -A curl -o "$file.part" "$address"; then
        rm -f "$file.part"
        echo "$id: could not be downloaded from $address" >&2
        status=1
        continue
    fi
    got=$(digest "$file.part")
    if [ "$got" != "$want" ]; then
        rm -f "$file.part"
        echo "$id: refused: the file's SHA-256 is $got, the manifest's is $want" >&2
        status=1
        continue
    fi
    mv "$file.part" "$file"
    echo "$id: fetched to $path"
done
exit $status
