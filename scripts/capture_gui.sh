#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts
xvfb_display=99
Xvfb ":$xvfb_display" -screen 0 1200x800x24 -nolisten tcp >/tmp/rovex-xvfb.log 2>&1 &
xvfb_pid=$!
app_pid=''
cleanup() {
    if [ -n "$app_pid" ]; then kill "$app_pid" 2>/dev/null || true; fi
    kill "$xvfb_pid" 2>/dev/null || true
    wait "$app_pid" 2>/dev/null || true
    wait "$xvfb_pid" 2>/dev/null || true
}
trap cleanup EXIT
sleep 1
DISPLAY=":$xvfb_display" target/release/rovex /tmp >/tmp/rovex-capture.log 2>&1 &
app_pid=$!
sleep 3
import -display ":$xvfb_display" -window root artifacts/rovex-dark-theme.png
kill "$app_pid" 2>/dev/null || true
wait "$app_pid" 2>/dev/null || true
cat /tmp/rovex-capture.log
