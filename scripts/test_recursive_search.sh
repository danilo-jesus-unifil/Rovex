#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
export PATH="/home/ubuntu/.cargo/bin:$PATH"

mkdir -p artifacts
root=$(mktemp -d)
xvfb_display=106
xvfb_pid=''
app_pid=''
cleanup() {
    if [ -n "$app_pid" ]; then kill "$app_pid" 2>/dev/null || true; fi
    if [ -n "$xvfb_pid" ]; then kill "$xvfb_pid" 2>/dev/null || true; fi
    rm -rf "$root"
    wait "$app_pid" 2>/dev/null || true
    wait "$xvfb_pid" 2>/dev/null || true
}
trap cleanup EXIT

mkdir -p "$root/nivel-1/nivel-2"
printf '%s\n' 'alvo profundo' >"$root/nivel-1/nivel-2/alvo-profundo.txt"
printf '%s\n' 'outro' >"$root/nivel-1/outro.txt"
printf '%s\n' 'alvo raiz' >"$root/alvo-raiz.txt"
Xvfb ":$xvfb_display" -screen 0 1200x800x24 -nolisten tcp >/tmp/rovex-search-xvfb.log 2>&1 &
xvfb_pid=$!
sleep 1
DISPLAY=":$xvfb_display" cargo build --release --quiet
DISPLAY=":$xvfb_display" target/release/rovex "$root" >/tmp/rovex-search-app.log 2>&1 &
app_pid=$!
sleep 3
window_id=$(DISPLAY=":$xvfb_display" xdotool search --name '^Rovex$' | head -n 1)
if [ -z "$window_id" ]; then
    cat /tmp/rovex-search-app.log >&2
    printf '%s\n' 'janela Rovex não encontrada' >&2
    exit 1
fi
DISPLAY=":$xvfb_display" xdotool windowfocus "$window_id"
# A barra de navegação fica no centro y≈98; o filtro ocupa x≈548–796 e Buscar x≈805–882.
DISPLAY=":$xvfb_display" xdotool mousemove --window "$window_id" 680 98 click 1
DISPLAY=":$xvfb_display" xdotool type --delay 15 alvo
DISPLAY=":$xvfb_display" xdotool mousemove --window "$window_id" 843 98 click 1
sleep 1
DISPLAY=":$xvfb_display" import -display ":$xvfb_display" -window root artifacts/rovex-recursive-search.png
# O mesmo comando deve mudar para Cancelar enquanto uma árvore maior ainda estiver sendo processada.
DISPLAY=":$xvfb_display" xdotool mousemove --window "$window_id" 843 98 click 1
sleep 0.3
if ! kill -0 "$app_pid" 2>/dev/null; then
    cat /tmp/rovex-search-app.log >&2
    printf '%s\n' 'o processo encerrou durante a busca recursiva' >&2
    exit 1
fi
printf '%s\n' 'Busca recursiva acionada e cancelamento exercitado sem encerramento do processo.'
cat /tmp/rovex-search-app.log
