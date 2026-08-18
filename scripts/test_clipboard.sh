#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
export PATH="/home/ubuntu/.cargo/bin:$PATH"

mkdir -p artifacts
root="$(mktemp -d /tmp/rovex-clipboard-smoke.XXXXXX)"
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

mkdir "$root/destino"
printf '%s\n' 'conteudo clipboard' >"$root/arquivo.txt"

xvfb_display=108
Xvfb ":$xvfb_display" -screen 0 1200x800x24 -nolisten tcp >/tmp/rovex-clipboard-xvfb.log 2>&1 &
xvfb_pid=$!
sleep 1
DISPLAY=":$xvfb_display" cargo build --release --quiet
DISPLAY=":$xvfb_display" target/release/rovex "$root" >/tmp/rovex-clipboard-app.log 2>&1 &
app_pid=$!
sleep 3
window_id=$(DISPLAY=":$xvfb_display" xdotool search --name '^Rovex$' | head -n 1)
if [ -z "$window_id" ]; then
    cat /tmp/rovex-clipboard-app.log >&2
    exit 1
fi
DISPLAY=":$xvfb_display" xdotool windowfocus "$window_id"
# Diretórios ficam antes de arquivos; o arquivo ocupa a segunda linha da listagem.
DISPLAY=":$xvfb_display" xdotool mousemove --window "$window_id" 420 286 click 1
DISPLAY=":$xvfb_display" xdotool key --window "$window_id" ctrl+x
sleep 0.5
DISPLAY=":$xvfb_display" import -display ":$xvfb_display" -window root artifacts/rovex-clipboard-after-cut.png
# Abrir a pasta destino e colar dentro dela.
DISPLAY=":$xvfb_display" xdotool mousemove --window "$window_id" 420 246 click --repeat 2 --delay 100 1
sleep 1
DISPLAY=":$xvfb_display" xdotool key --window "$window_id" ctrl+v
sleep 2
DISPLAY=":$xvfb_display" import -display ":$xvfb_display" -window root artifacts/rovex-clipboard-after-paste.png
if [ ! -f "$root/destino/arquivo.txt" ] || [ -f "$root/arquivo.txt" ]; then
    cat /tmp/rovex-clipboard-app.log >&2
    printf '%s\n' 'Cut/Paste não moveu o arquivo no filesystem' >&2
    exit 1
fi
if ! kill -0 "$app_pid" 2>/dev/null; then
    cat /tmp/rovex-clipboard-app.log >&2
    exit 1
fi
DISPLAY=":$xvfb_display" import -display ":$xvfb_display" -window root artifacts/rovex-clipboard.png
printf '%s\n' 'Clipboard Cut/Paste moveu o arquivo real e processo permaneceu ativo.'
cat /tmp/rovex-clipboard-app.log
