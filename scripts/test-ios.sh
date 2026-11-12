#!/bin/sh
# Runs Lamplight's UI tests in a simulator and prints one line per test.
# Usage: scripts/test-ios.sh [26|27] [-only-testing:LamplightUITests/LamplightUITests/testName ...]
set -eu
root=$(cd "$(dirname "$0")/.." && pwd)
case "${1:-27}" in
    26) destination='platform=iOS Simulator,name=iPhone 17 Pro,OS=26.0' ;;
    27) destination='platform=iOS Simulator,name=iPhone 18 Pro,OS=27.2' ;;
    *) echo "usage: $0 [26|27] [xcodebuild test options]" >&2; exit 2 ;;
esac
[ $# -gt 0 ] && shift
cd "$root/apps/ios"
log=$(mktemp)
status=0
xcodebuild test -project Lamplight.xcodeproj -scheme Lamplight \
    -destination "$destination" CODE_SIGNING_ALLOWED=NO "$@" >"$log" 2>&1 || status=$?
grep -E "^Test Case .*(passed|failed)|error:|XCTAssert|\*\* TEST|Executed [0-9]+ tests" "$log" | cut -c1-400 | sort -u || true
echo "full log: $log"
exit "$status"
