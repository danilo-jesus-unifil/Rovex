#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
export PATH="/home/ubuntu/.cargo/bin:$PATH"

root="$(mktemp -d /tmp/rovex-settings-smoke.XXXXXX)"
config_dir="$(mktemp -d /tmp/rovex-settings-config.XXXXXX)"
xvfb_display=107
app_pid=''
xvfb_pid=''
cleanup() {
    if [ -n "$app_pid" ]; then kill "$app_pid" 2>/dev/null || true; fi
    if [ -n "$xvfb_pid" ]; then kill "$xvfb_pid" 2>/dev/null || true; fi
    wait "$app_pid" 2>/dev/null || true
    wait "$xvfb_pid" 2>/dev/null || true
    rm -rf "$root" "$config_dir"
}
trap cleanup EXIT

printf '%s\n' 'arquivo' >"$root/arquivo.txt"
mkdir -p "$root/subpasta"
Xvfb ":$xvfb_display" -screen 0 1200x800x24 -nolisten tcp >/tmp/rovex-settings-xvfb.log 2>&1 &
xvfb_pid=$!
sleep 1
DISPLAY=":$xvfb_display" cargo build --release --quiet

ROVEX_CONFIG_DIR="$config_dir" DISPLAY=":$xvfb_display" target/release/rovex "$root" >/tmp/rovex-settings-app.log 2>&1 &
app_pid=$!
sleep 3
window_id=$(DISPLAY=":$xvfb_display" xdotool search --name '^Rovex$' | head -n 1)
if [ -z "$window_id" ]; then
    cat /tmp/rovex-settings-app.log >&2
    exit 1
fi
DISPLAY=":$xvfb_display" xdotool windowfocus "$window_id"
# Ocultos e o cabeçalho de modificação são preferências reais da UI.
DISPLAY=":$xvfb_display" xdotool mousemove --window "$window_id" 171 158 click 1
DISPLAY=":$xvfb_display" xdotool mousemove --window "$window_id" 650 210 click 1
sleep 1
config_file="$config_dir/Rovex/settings.v1.conf"
if [ ! -s "$config_file" ]; then
    printf '%s\n' 'arquivo de configuração não foi criado' >&2
    find "$config_dir" -maxdepth 3 -type f -print >&2 || true
    cat /tmp/rovex-settings-app.log >&2
    exit 1
fi
printf '%s\n' '--- settings.v1.conf ---'
cat "$config_file"
grep -q '^version=1$' "$config_file" || { cat /tmp/rovex-settings-app.log >&2; exit 1; }
grep -q '^show_hidden_files=1$' "$config_file" || { cat /tmp/rovex-settings-app.log >&2; exit 1; }
grep -q '^sort_ascending=1$' "$config_file" || { cat /tmp/rovex-settings-app.log >&2; exit 1; }
kill "$app_pid" 2>/dev/null || true
wait "$app_pid" 2>/dev/null || true
app_pid=''

ROVEX_CONFIG_DIR="$config_dir" DISPLAY=":$xvfb_display" target/release/rovex >/tmp/rovex-settings-reload.log 2>&1 &
app_pid=$!
sleep 3
if ! kill -0 "$app_pid" 2>/dev/null; then
    cat /tmp/rovex-settings-reload.log >&2
    exit 1
fi
DISPLAY=":$xvfb_display" import -display ":$xvfb_display" -window root artifacts/rovex-settings-reloaded.png
printf '%s\n' 'Configuração persistida e restaurada; UI permaneceu ativa na segunda execução.'
cat /tmp/rovex-settings-app.log
cat /tmp/rovex-settings-reload.log
