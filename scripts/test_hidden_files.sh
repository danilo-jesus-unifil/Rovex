#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
export PATH="/home/ubuntu/.cargo/bin:$PATH"

mkdir -p artifacts
root="$(mktemp -d /tmp/rovex-hidden-smoke.XXXXXX)"
app_pid=''
xvfb_pid=''
cleanup() {
    if [ -n "$app_pid" ]; then kill "$app_pid" 2>/dev/null || true; fi
    if [ -n "$xvfb_pid" ]; then kill "$xvfb_pid" 2>/dev/null || true; fi
    wait "$app_pid" 2>/dev/null || true
    wait "$xvfb_pid" 2>/dev/null || true
    rm -rf "$root"
}
trap cleanup EXIT
printf '%s\n' 'visível' >"$root/visivel.txt"
printf '%s\n' 'oculto' >"$root/.segredo.txt"

xvfb_display=106
Xvfb ":$xvfb_display" -screen 0 1200x800x24 -nolisten tcp >/tmp/rovex-hidden-xvfb.log 2>&1 &
xvfb_pid=$!
sleep 1
DISPLAY=":$xvfb_display" cargo build --release --quiet
DISPLAY=":$xvfb_display" target/release/rovex "$root" >/tmp/rovex-hidden-app.log 2>&1 &
app_pid=$!
sleep 3
window_id=$(DISPLAY=":$xvfb_display" xdotool search --name '^Rovex$' | head -n 1)
if [ -z "$window_id" ]; then
    cat /tmp/rovex-hidden-app.log >&2
    exit 1
fi
DISPLAY=":$xvfb_display" xdotool windowfocus "$window_id"
DISPLAY=":$xvfb_display" import -display ":$xvfb_display" -window root artifacts/rovex-hidden-off.png
# Operações ocupa a terceira barra; Ocultos é o primeiro botão dessa barra.
DISPLAY=":$xvfb_display" xdotool mousemove --window "$window_id" 55 158 click 1
sleep 1
DISPLAY=":$xvfb_display" import -display ":$xvfb_display" -window root artifacts/rovex-hidden-on.png
if ! kill -0 "$app_pid" 2>/dev/null; then
    cat /tmp/rovex-hidden-app.log >&2
    exit 1
fi
printf '%s\n' 'Toggle Ocultos exercitado sem encerramento do processo.'
cat /tmp/rovex-hidden-app.log
