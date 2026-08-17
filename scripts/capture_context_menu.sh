#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts
xvfb_display=100
Xvfb ":$xvfb_display" -screen 0 1200x800x24 -nolisten tcp >/tmp/rovex-context-xvfb.log 2>&1 &
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
DISPLAY=":$xvfb_display" target/release/rovex /tmp >/tmp/rovex-context-app.log 2>&1 &
app_pid=$!
sleep 3
window_id=$(DISPLAY=":$xvfb_display" xdotool search --name '^Rovex$' | head -n 1)
if [ -z "$window_id" ]; then
    cat /tmp/rovex-context-app.log >&2
    printf '%s\n' 'janela Rovex não encontrada' >&2
    exit 1
fi
DISPLAY=":$xvfb_display" xdotool windowfocus "$window_id"
DISPLAY=":$xvfb_display" xdotool mousemove --window "$window_id" 520 190 click 3
sleep 1
DISPLAY=":$xvfb_display" import -display ":$xvfb_display" -window root artifacts/rovex-context-menu.png
kill "$app_pid" 2>/dev/null || true
wait "$app_pid" 2>/dev/null || true
cat /tmp/rovex-context-app.log
