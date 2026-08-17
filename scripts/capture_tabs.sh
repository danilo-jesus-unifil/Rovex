#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts
xvfb_display=103
Xvfb ":$xvfb_display" -screen 0 1200x800x24 -nolisten tcp >/tmp/rovex-tabs-xvfb.log 2>&1 &
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
DISPLAY=":$xvfb_display" target/release/rovex /tmp >/tmp/rovex-tabs-app.log 2>&1 &
app_pid=$!
sleep 3
window_id=$(DISPLAY=":$xvfb_display" xdotool search --name '^Rovex$' | head -n 1)
if [ -z "$window_id" ]; then
    cat /tmp/rovex-tabs-app.log >&2
    printf '%s\n' 'janela Rovex não encontrada' >&2
    exit 1
fi
DISPLAY=":$xvfb_display" xdotool windowfocus "$window_id"
# O botão + fica na faixa de abas, logo acima da toolbar de navegação.
DISPLAY=":$xvfb_display" xdotool mousemove --window "$window_id" 230 35 click 1
sleep 0.8
DISPLAY=":$xvfb_display" import -display ":$xvfb_display" -window root artifacts/rovex-tabs-two.png
# Selecionar a aba inicial e fechar a segunda aba pelo botão x.
DISPLAY=":$xvfb_display" xdotool mousemove --window "$window_id" 90 35 click 1
sleep 0.3
DISPLAY=":$xvfb_display" xdotool mousemove --window "$window_id" 190 35 click 1
sleep 0.8
if ! kill -0 "$app_pid" 2>/dev/null; then
    cat /tmp/rovex-tabs-app.log >&2
    printf '%s\n' 'o processo encerrou durante o teste de abas' >&2
    exit 1
fi
DISPLAY=":$xvfb_display" import -display ":$xvfb_display" -window root artifacts/rovex-tabs-one.png
printf '%s\n' 'Fluxo de abas aprovado: abriu segunda aba, alternou para a primeira e fechou a segunda.'
kill "$app_pid" 2>/dev/null || true
wait "$app_pid" 2>/dev/null || true
cat /tmp/rovex-tabs-app.log
