#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
export PATH="/home/ubuntu/.cargo/bin:$PATH"

mkdir -p artifacts
root="$(mktemp -d /tmp/rovex-properties-smoke.XXXXXX)"
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
printf '%s\n' 'propriedades' >"$root/relatório.txt"

xvfb_display=109
Xvfb ":$xvfb_display" -screen 0 1200x800x24 -nolisten tcp >/tmp/rovex-properties-xvfb.log 2>&1 &
xvfb_pid=$!
sleep 1
DISPLAY=":$xvfb_display" cargo build --release --quiet
DISPLAY=":$xvfb_display" target/release/rovex "$root" >/tmp/rovex-properties-app.log 2>&1 &
app_pid=$!
sleep 3
window_id=$(DISPLAY=":$xvfb_display" xdotool search --name '^Rovex$' | head -n 1)
if [ -z "$window_id" ]; then
    cat /tmp/rovex-properties-app.log >&2
    exit 1
fi
DISPLAY=":$xvfb_display" xdotool windowfocus "$window_id"
DISPLAY=":$xvfb_display" xdotool mousemove --window "$window_id" 420 246 click 3
sleep 0.5
DISPLAY=":$xvfb_display" import -display ":$xvfb_display" -window root artifacts/rovex-properties-menu.png
# O item Propriedades fica depois de Copiar, Mover e Renomear/Excluir.
DISPLAY=":$xvfb_display" xdotool mousemove --window "$window_id" 180 314 click 1
sleep 1
DISPLAY=":$xvfb_display" import -display ":$xvfb_display" -window root artifacts/rovex-properties.png
DISPLAY=":$xvfb_display" xdotool mousemove --window "$window_id" 500 450 click --repeat 6 5
sleep 0.5
DISPLAY=":$xvfb_display" import -display ":$xvfb_display" -window root artifacts/rovex-properties-scrolled.png
if [ ! -f "$root/relatório.txt" ]; then
    cat /tmp/rovex-properties-app.log >&2
    printf '%s\n' 'A leitura de propriedades alterou o arquivo' >&2
    exit 1
fi
if ! kill -0 "$app_pid" 2>/dev/null; then
    cat /tmp/rovex-properties-app.log >&2
    exit 1
fi
printf '%s\n' 'Propriedades abertas sem mutação do arquivo e processo permaneceu ativo.'
cat /tmp/rovex-properties-app.log
