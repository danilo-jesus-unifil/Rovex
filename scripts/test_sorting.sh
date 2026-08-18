#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
export PATH="/home/ubuntu/.cargo/bin:$PATH"

mkdir -p artifacts
xvfb_display=105
Xvfb ":$xvfb_display" -screen 0 1200x800x24 -nolisten tcp >/tmp/rovex-sort-xvfb.log 2>&1 &
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
DISPLAY=":$xvfb_display" target/release/rovex /tmp >/tmp/rovex-sort-app.log 2>&1 &
app_pid=$!
sleep 3
window_id=$(DISPLAY=":$xvfb_display" xdotool search --name '^Rovex$' | head -n 1)
if [ -z "$window_id" ]; then
    cat /tmp/rovex-sort-app.log >&2
    printf '%s\n' 'janela Rovex não encontrada' >&2
    exit 1
fi

DISPLAY=":$xvfb_display" xdotool windowfocus "$window_id"
# O cabeçalho fica logo acima da lista; cada clique deve alternar sem derrubar a janela.
for coordinate in '430 210' '530 210' '650 210' '650 210'; do
    read -r x y <<<"$coordinate"
    DISPLAY=":$xvfb_display" xdotool mousemove --window "$window_id" "$x" "$y" click 1
    sleep 0.4
done
if ! kill -0 "$app_pid" 2>/dev/null; then
    cat /tmp/rovex-sort-app.log >&2
    printf '%s\n' 'o processo encerrou durante o teste de ordenação' >&2
    exit 1
fi
DISPLAY=":$xvfb_display" import -display ":$xvfb_display" -window root artifacts/rovex-sorting.png
printf '%s\n' 'Ordenação por nome, tamanho e modificação exercitada sem encerramento do processo.'
cat /tmp/rovex-sort-app.log
