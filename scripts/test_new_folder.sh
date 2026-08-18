#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
export PATH="/home/ubuntu/.cargo/bin:$PATH"

mkdir -p artifacts
root="$(mktemp -d /tmp/rovex-new-folder-smoke.XXXXXX)"
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

xvfb_display=107
Xvfb ":$xvfb_display" -screen 0 1200x800x24 -nolisten tcp >/tmp/rovex-new-folder-xvfb.log 2>&1 &
xvfb_pid=$!
sleep 1
DISPLAY=":$xvfb_display" cargo build --release --quiet
DISPLAY=":$xvfb_display" target/release/rovex "$root" >/tmp/rovex-new-folder-app.log 2>&1 &
app_pid=$!
sleep 3
window_id=$(DISPLAY=":$xvfb_display" xdotool search --name '^Rovex$' | head -n 1)
if [ -z "$window_id" ]; then
    cat /tmp/rovex-new-folder-app.log >&2
    exit 1
fi
DISPLAY=":$xvfb_display" xdotool windowfocus "$window_id"
# Nova pasta é o primeiro botão da barra de operações.
DISPLAY=":$xvfb_display" xdotool mousemove --window "$window_id" 55 158 click 1
sleep 0.5
DISPLAY=":$xvfb_display" import -display ":$xvfb_display" -window root artifacts/rovex-new-folder-dialog.png
# O input e o botão Confirmar ficam no diálogo central padrão do Rovex.
DISPLAY=":$xvfb_display" xdotool mousemove --window "$window_id" 360 345 click 1
DISPLAY=":$xvfb_display" xdotool type --delay 15 -- "Nova pasta"
DISPLAY=":$xvfb_display" xdotool mousemove --window "$window_id" 220 465 click 1
sleep 2
if [ ! -d "$root/Nova pasta" ]; then
    cat /tmp/rovex-new-folder-app.log >&2
    printf '%s\n' 'Nova pasta não foi criada no filesystem' >&2
    exit 1
fi
if ! kill -0 "$app_pid" 2>/dev/null; then
    cat /tmp/rovex-new-folder-app.log >&2
    exit 1
fi
DISPLAY=":$xvfb_display" import -display ":$xvfb_display" -window root artifacts/rovex-new-folder.png
printf '%s\n' 'Nova pasta criada no filesystem e processo permaneceu ativo.'
cat /tmp/rovex-new-folder-app.log
