#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
export PATH="/home/ubuntu/.cargo/bin:$PATH"

mkdir -p artifacts
root=$(mktemp -d)
xvfb_display=107
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

# PNG 1x1 válido; a imagem é criada sem chamar nenhum executável externo.
printf '%s' 'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+A8AAQUBAScY42YAAAAASUVORK5CYII=' | base64 -d >"$root/foto.png"
printf '%s\n' 'conteúdo inválido' >"$root/falso.jpg"
Xvfb ":$xvfb_display" -screen 0 1200x800x24 -nolisten tcp >/tmp/rovex-preview-xvfb.log 2>&1 &
xvfb_pid=$!
sleep 1
DISPLAY=":$xvfb_display" cargo build --release --quiet
DISPLAY=":$xvfb_display" target/release/rovex "$root" >/tmp/rovex-preview-app.log 2>&1 &
app_pid=$!
sleep 3
window_id=$(DISPLAY=":$xvfb_display" xdotool search --name '^Rovex$' | head -n 1)
if [ -z "$window_id" ]; then
    cat /tmp/rovex-preview-app.log >&2
    printf '%s\n' 'janela Rovex não encontrada' >&2
    exit 1
fi
DISPLAY=":$xvfb_display" xdotool windowfocus "$window_id"
# O primeiro arquivo aparece na primeira linha da listagem, em y≈246.
DISPLAY=":$xvfb_display" xdotool mousemove --window "$window_id" 420 246 click 1
sleep 1
DISPLAY=":$xvfb_display" import -display ":$xvfb_display" -window root artifacts/rovex-preview-fallback.png
# O segundo item é a imagem válida e deve substituir o fallback no mesmo painel.
DISPLAY=":$xvfb_display" xdotool mousemove --window "$window_id" 420 286 click 1
sleep 1
DISPLAY=":$xvfb_display" import -display ":$xvfb_display" -window root artifacts/rovex-preview-valid.png
# O X do cabeçalho fecha o painel sem alterar o arquivo selecionado.
DISPLAY=":$xvfb_display" xdotool mousemove --window "$window_id" 896 220 click 1
sleep 0.3
DISPLAY=":$xvfb_display" import -display ":$xvfb_display" -window root artifacts/rovex-preview-closed.png
if ! kill -0 "$app_pid" 2>/dev/null; then
    cat /tmp/rovex-preview-app.log >&2
    printf '%s\n' 'o processo encerrou durante o preview' >&2
    exit 1
fi
printf '%s\n' 'Preview de imagem e fallback de conteúdo inválido exercitados sem encerramento do processo.'
cat /tmp/rovex-preview-app.log
