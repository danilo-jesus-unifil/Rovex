#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts
fixture=/tmp/rovex-ui-jxl-fixture
rm -rf "$fixture"
mkdir -p "$fixture"
ffmpeg -hide_banner -loglevel error -nostdin -f lavfi -i color=c=0x2563eb:s=16x16 -frames:v 1 "$fixture/entrada.png"
xvfb_display=102
Xvfb ":$xvfb_display" -screen 0 1200x800x24 -nolisten tcp >/tmp/rovex-ui-jxl-xvfb.log 2>&1 &
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
DISPLAY=":$xvfb_display" target/release/rovex "$fixture" >/tmp/rovex-ui-jxl-app.log 2>&1 &
app_pid=$!
sleep 3
window_id=$(DISPLAY=":$xvfb_display" xdotool search --name '^Rovex$' | head -n 1)
if [ -z "$window_id" ]; then
    cat /tmp/rovex-ui-jxl-app.log >&2
    exit 1
fi
DISPLAY=":$xvfb_display" xdotool windowfocus "$window_id"
DISPLAY=":$xvfb_display" xdotool mousemove --window "$window_id" 520 190 click 3
sleep 0.4
DISPLAY=":$xvfb_display" xdotool mousemove --window "$window_id" 180 320 click 1
sleep 0.5
DISPLAY=":$xvfb_display" import -display ":$xvfb_display" -window root artifacts/rovex-jxl-confirm.png
DISPLAY=":$xvfb_display" xdotool mousemove --window "$window_id" 220 464 click 1
for _ in 1 2 3 4 5 6 7 8 9 10; do
    if [ -s "$fixture/entrada.jxl" ]; then break; fi
    sleep 0.5
done
if [ ! -s "$fixture/entrada.jxl" ]; then
    cat /tmp/rovex-ui-jxl-app.log >&2
    printf '%s\n' 'saída JXL não foi publicada pela UI' >&2
    exit 1
fi
DISPLAY=":$xvfb_display" import -display ":$xvfb_display" -window root artifacts/rovex-jxl-result.png
printf 'Saída JXL criada: %s bytes\n' "$(stat -c '%s' "$fixture/entrada.jxl")"
kill "$app_pid" 2>/dev/null || true
wait "$app_pid" 2>/dev/null || true
cat /tmp/rovex-ui-jxl-app.log
