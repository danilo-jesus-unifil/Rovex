#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
export PATH="/home/ubuntu/.cargo/bin:$PATH"

mkdir -p artifacts
xvfb_display=104
Xvfb ":$xvfb_display" -screen 0 1200x800x24 -nolisten tcp >/tmp/rovex-keyboard-xvfb.log 2>&1 &
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
DISPLAY=":$xvfb_display" cargo build --release --quiet
DISPLAY=":$xvfb_display" target/release/rovex /tmp >/tmp/rovex-keyboard-app.log 2>&1 &
app_pid=$!
sleep 3
window_id=$(DISPLAY=":$xvfb_display" xdotool search --name '^Rovex$' | head -n 1)
if [ -z "$window_id" ]; then
    cat /tmp/rovex-keyboard-app.log >&2
    printf '%s\n' 'janela Rovex não encontrada' >&2
    exit 1
fi

DISPLAY=":$xvfb_display" xdotool windowfocus "$window_id"
# Foco na listagem: dispara os atalhos de operação sem executar uma operação destrutiva.
DISPLAY=":$xvfb_display" xdotool mousemove --window "$window_id" 520 246 click 1
DISPLAY=":$xvfb_display" xdotool key F2
sleep 0.4
DISPLAY=":$xvfb_display" xdotool key Escape
DISPLAY=":$xvfb_display" xdotool key Delete
sleep 0.4
DISPLAY=":$xvfb_display" xdotool key Escape
# Ctrl+F e Ctrl+L devem mover o foco para os campos reais; Ctrl+L restaura /tmp ao final.
DISPLAY=":$xvfb_display" xdotool key ctrl+f
DISPLAY=":$xvfb_display" xdotool type --delay 10 rovex
DISPLAY=":$xvfb_display" xdotool key ctrl+l
DISPLAY=":$xvfb_display" xdotool key ctrl+a
DISPLAY=":$xvfb_display" xdotool type --delay 10 /tmp
DISPLAY=":$xvfb_display" xdotool key Return
# Alt+Left/Right, Backspace e setas devem permanecer seguros mesmo sem histórico suficiente.
DISPLAY=":$xvfb_display" xdotool key alt+Left
DISPLAY=":$xvfb_display" xdotool key alt+Right
DISPLAY=":$xvfb_display" xdotool key Down
DISPLAY=":$xvfb_display" xdotool key Up
sleep 1
if ! kill -0 "$app_pid" 2>/dev/null; then
    cat /tmp/rovex-keyboard-app.log >&2
    printf '%s\n' 'o processo encerrou durante o teste de atalhos' >&2
    exit 1
fi
DISPLAY=":$xvfb_display" import -display ":$xvfb_display" -window root artifacts/rovex-keyboard-shortcuts.png
printf '%s\n' 'Atalhos de teclado exercitados sem encerramento ou erro fatal do processo.'
cat /tmp/rovex-keyboard-app.log
