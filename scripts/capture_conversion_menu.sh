#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts
fixture=/tmp/rovex-ui-conversion-fixture
rm -rf "$fixture"
mkdir -p "$fixture"
ffmpeg -hide_banner -loglevel error -nostdin -f lavfi -i color=c=0x2563eb:s=16x16 -frames:v 1 "$fixture/entrada.png"
ffmpeg -hide_banner -loglevel error -nostdin -f lavfi -i sine=frequency=440:duration=0.2 -c:a pcm_s16le "$fixture/entrada.wav"
xvfb_display=101
Xvfb ":$xvfb_display" -screen 0 1200x800x24 -nolisten tcp >/tmp/rovex-conversion-menu-xvfb.log 2>&1 &
xvfb_pid=$!
app_pid=''
cleanup() {
    if [ -n "$app_pid" ]; then kill "$app_pid" 2>/dev/null || true; fi
    kill "$xvfb_pid" 2>/dev/null || true
    wait "$app_pid" 2>/dev/null || true
    wait "$xvfb_pid" 2>/dev/null || true
    rm -rf "$fixture"
}
trap cleanup EXIT
sleep 1
DISPLAY=":$xvfb_display" target/release/rovex "$fixture" >/tmp/rovex-conversion-menu-app.log 2>&1 &
app_pid=$!
sleep 3
window_id=$(DISPLAY=":$xvfb_display" xdotool search --name '^Rovex$' | head -n 1)
if [ -z "$window_id" ]; then
    cat /tmp/rovex-conversion-menu-app.log >&2
    exit 1
fi
DISPLAY=":$xvfb_display" xdotool windowfocus "$window_id"
DISPLAY=":$xvfb_display" xdotool mousemove --window "$window_id" 520 246 click 3
sleep 1
DISPLAY=":$xvfb_display" import -display ":$xvfb_display" -window root artifacts/rovex-conversion-menu.png
kill "$app_pid" 2>/dev/null || true
wait "$app_pid" 2>/dev/null || true
cat /tmp/rovex-conversion-menu-app.log
