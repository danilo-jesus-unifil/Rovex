#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
fixture_dir="$(mktemp -d /tmp/rovex-open-with-fixture.XXXXXX)"
fixture_file="$fixture_dir/nota com espaço — ação.txt"
printf '%s\n' 'arquivo de teste' >"$fixture_file"
xvfb_display=101
Xvfb ":$xvfb_display" -screen 0 1200x800x24 -nolisten tcp >/tmp/rovex-open-with-xvfb.log 2>&1 &
xvfb_pid=$!
app_pid=''
cleanup() {
    if [ -n "$app_pid" ]; then kill "$app_pid" 2>/dev/null || true; fi
    kill "$xvfb_pid" 2>/dev/null || true
    rm -rf "$fixture_dir"
    wait "$app_pid" 2>/dev/null || true
    wait "$xvfb_pid" 2>/dev/null || true
}
trap cleanup EXIT
sleep 1
DISPLAY=":$xvfb_display" target/release/rovex "$fixture_dir" >/tmp/rovex-open-with-app.log 2>&1 &
app_pid=$!
sleep 3
window_id=$(DISPLAY=":$xvfb_display" xdotool search --name '^Rovex$' | head -n 1)
if [ -z "$window_id" ]; then
    cat /tmp/rovex-open-with-app.log >&2
    printf '%s\n' 'janela Rovex não encontrada' >&2
    exit 1
fi
DISPLAY=":$xvfb_display" xdotool windowfocus "$window_id"
DISPLAY=":$xvfb_display" xdotool mousemove --window "$window_id" 520 246 click 3
sleep 1
DISPLAY=":$xvfb_display" import -display ":$xvfb_display" -window root artifacts/rovex-open-with-file-menu.png
printf '%s\n' 'Open With smoke concluído: menu contextual de arquivo regular capturado.'
