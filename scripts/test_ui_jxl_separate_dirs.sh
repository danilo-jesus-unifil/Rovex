#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

root=/tmp/rovex-jxl-separate-dirs
bin_dir="$root/bin"
image_dir="$root/images"
rm -rf "$root"
mkdir -p "$bin_dir" "$image_dir"
cp target/release/rovex "$bin_dir/rovex"
ffmpeg -hide_banner -loglevel error -nostdin \
    -f lavfi -i color=c=0x2563eb:s=16x16 -frames:v 1 \
    "$image_dir/entrada.png"

xvfb_display=103
Xvfb ":$xvfb_display" -screen 0 1200x800x24 -nolisten tcp \
    >"$root/xvfb.log" 2>&1 &
xvfb_pid=$!
app_pid=''
cleanup() {
    if [ -n "$app_pid" ]; then kill "$app_pid" 2>/dev/null || true; fi
    kill "$xvfb_pid" 2>/dev/null || true
    wait "$app_pid" 2>/dev/null || true
    wait "$xvfb_pid" 2>/dev/null || true
    rm -rf "$root"
}
trap cleanup EXIT

sleep 1
DISPLAY=":$xvfb_display" "$bin_dir/rovex" "$image_dir" \
    >"$root/app.log" 2>&1 &
app_pid=$!
sleep 3
window_id=$(DISPLAY=":$xvfb_display" xdotool search --name '^Rovex$' | head -n 1)
if [ -z "$window_id" ]; then
    cat "$root/app.log" >&2
    exit 1
fi
DISPLAY=":$xvfb_display" xdotool windowfocus "$window_id"
DISPLAY=":$xvfb_display" xdotool mousemove --window "$window_id" 520 234 click 3
sleep 0.4
DISPLAY=":$xvfb_display" xdotool mousemove --window "$window_id" 180 320 click 1
sleep 0.5
DISPLAY=":$xvfb_display" xdotool mousemove --window "$window_id" 220 464 click 1

for _ in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20; do
    if [ -s "$image_dir/entrada.jxl" ]; then break; fi
    sleep 0.5
done
if [ ! -s "$image_dir/entrada.jxl" ]; then
    cat "$root/app.log" >&2
    printf '%s\n' 'saída JXL não foi publicada pela UI' >&2
    exit 1
fi
codec=$(ffprobe -hide_banner -v error -select_streams v:0 \
    -show_entries stream=codec_name -of default=nw=1:nk=1 \
    "$image_dir/entrada.jxl")
test "$codec" = jpegxl
printf 'Conversão real aprovada: binário=%s imagem=%s saída=%s codec=%s\n' \
    "$bin_dir/rovex" "$image_dir/entrada.png" "$image_dir/entrada.jxl" "$codec"
