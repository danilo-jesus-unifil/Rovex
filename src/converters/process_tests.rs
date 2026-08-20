#![cfg(unix)]

use super::process::{run_ffprobe_with_timeout, spawn_ffmpeg_with_timeout};
use super::{ConversionError, ConversionKind};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn fake_backend(name: &str, body: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "rovex-process-{name}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    let temporary = path.with_extension("tmp");
    fs::write(&temporary, format!("#!/bin/sh\n{body}\n")).expect("write fake backend");
    let mut permissions = fs::metadata(&temporary).expect("metadata").permissions();
    permissions.set_mode(0o700);
    fs::set_permissions(&temporary, permissions).expect("permissions");
    fs::rename(temporary, &path).expect("publish fake backend");
    path
}

fn fake_backend_with_ready_marker(name: &str, ready: &Path) -> PathBuf {
    let ready_path = ready.to_string_lossy().replace('\'', "'\\''");
    fake_backend(
        name,
        &format!("printf ready > '{ready_path}'\nexec sleep 30"),
    )
}

fn cleanup(path: &Path, temporary: &Path) {
    let _ = fs::remove_file(path);
    let _ = fs::remove_file(temporary);
}

#[test]
fn ffmpeg_fake_is_killed_when_cancelled() {
    let backend_path = std::env::temp_dir().join(format!(
        "rovex-process-cancel-ready-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    let backend = fake_backend_with_ready_marker("cancel", &backend_path.with_extension("ready"));
    let ready = backend_path.with_extension("ready");
    let source = backend.with_extension("input");
    let temporary = backend.with_extension("output");
    fs::write(&source, b"input").expect("source");
    let cancel = Arc::new(AtomicBool::new(false));
    let signal = Arc::clone(&cancel);
    let backend_for_thread = backend.clone();
    let source_for_thread = source.clone();
    let temporary_for_thread = temporary.clone();
    let handle = thread::spawn(move || {
        spawn_ffmpeg_with_timeout(
            &backend_for_thread,
            &source_for_thread,
            &temporary_for_thread,
            ConversionKind::Png,
            signal.as_ref(),
            &mut |_| {},
            Duration::from_secs(30),
        )
    });
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while !ready.exists() && std::time::Instant::now() < deadline {
        thread::sleep(Duration::from_millis(10));
    }
    assert!(ready.exists(), "fake backend não iniciou o processo longo");
    cancel.store(true, Ordering::Release);
    let error = handle.join().expect("worker").expect_err("cancel");
    assert!(
        matches!(error, ConversionError::Cancelled),
        "cancelamento retornou erro inesperado: {error:?}"
    );
    cleanup(&backend, &source);
    let _ = fs::remove_file(temporary);
    let _ = fs::remove_file(ready);
}

#[test]
fn cancelamento_encerra_descendente_que_mantem_pipe_aberto() {
    let ready = std::env::temp_dir().join(format!(
        "rovex-process-descendant-ready-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    let ready_path = ready.to_string_lossy().replace('\'', "'\\''");
    let backend = fake_backend(
        "cancel-descendant",
        &format!("sleep 3 >&2 &\nprintf ready > '{ready_path}'\nexec sleep 3"),
    );
    let source = backend.with_extension("input");
    let temporary = backend.with_extension("output");
    fs::write(&source, b"input").expect("source");
    let cancel = Arc::new(AtomicBool::new(false));
    let signal = Arc::clone(&cancel);
    let backend_for_thread = backend.clone();
    let source_for_thread = source.clone();
    let temporary_for_thread = temporary.clone();
    let started = std::time::Instant::now();
    let handle = thread::spawn(move || {
        spawn_ffmpeg_with_timeout(
            &backend_for_thread,
            &source_for_thread,
            &temporary_for_thread,
            ConversionKind::Png,
            signal.as_ref(),
            &mut |_| {},
            Duration::from_secs(10),
        )
    });
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while !ready.exists() && std::time::Instant::now() < deadline {
        thread::sleep(Duration::from_millis(10));
    }
    assert!(ready.exists(), "fake backend não iniciou o descendente");
    cancel.store(true, Ordering::Release);
    let error = handle.join().expect("worker").expect_err("cancel");
    assert!(matches!(error, ConversionError::Cancelled));
    assert!(
        started.elapsed() < Duration::from_secs(2),
        "o leitor ficou bloqueado pelo descendente: {:?}",
        started.elapsed()
    );
    cleanup(&backend, &source);
    let _ = fs::remove_file(temporary);
    let _ = fs::remove_file(ready);
}

#[test]
fn ffprobe_fake_times_out_without_waiting_for_process_naturally() {
    let backend = fake_backend("timeout", "exec sleep 30");
    let destination = backend.with_extension("output");
    fs::write(&destination, b"output").expect("destination");
    let cancel = AtomicBool::new(false);
    let started = std::time::Instant::now();
    let error = run_ffprobe_with_timeout(
        &backend,
        &destination,
        "v:0",
        &cancel,
        Duration::from_millis(120),
    )
    .expect_err("timeout");
    assert!(
        matches!(
            error,
            ConversionError::Timeout {
                executable: "ffprobe",
                ..
            }
        ),
        "unexpected timeout result: {error:?}"
    );
    assert!(started.elapsed() < Duration::from_secs(3));
    cleanup(&backend, &destination);
}

#[test]
fn ffprobe_usa_argumentos_separados_e_nao_flag_exclusiva_do_ffmpeg() {
    let backend = Path::new("/bin/echo");
    let destination =
        std::env::temp_dir().join(format!("rovex-process-args-output-{}", std::process::id()));
    let output = run_ffprobe_with_timeout(
        backend,
        &destination,
        "a:0",
        &AtomicBool::new(false),
        Duration::from_secs(2),
    )
    .expect("fake ffprobe");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout
            .split_whitespace()
            .any(|line| line == "-select_streams")
    );
    assert!(stdout.split_whitespace().any(|line| line == "a:0"));
    assert!(!stdout.split_whitespace().any(|line| line == "-nostdin"));
    let _ = fs::remove_file(destination);
}
