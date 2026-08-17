use super::{
    ConversionError, ConversionKind, convert_file, output_path, push_path_or_directory_candidates,
    read_limited_output, resolve_backend, resolve_backend_from_candidates,
};
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::AtomicBool;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn diagnostico_de_backend_tem_limite_de_tamanho() {
    let oversized = vec![b'x'; super::MAX_PROCESS_OUTPUT_BYTES + 1];
    let error = read_limited_output(oversized.as_slice())
        .expect_err("diagnóstico acima do limite deve ser recusado");
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
}

#[test]
fn conversores_reconhecem_extensoes_sem_diferenciar_maiusculas() {
    assert!(ConversionKind::JpegXl.accepts(Path::new("foto.JPEG")));
    assert!(ConversionKind::Opus.accepts(Path::new("faixa.WAV")));
    assert!(!ConversionKind::Opus.accepts(Path::new("foto.jpg")));
}

#[test]
fn saida_usa_nome_irmao_e_evita_mesmo_caminho() {
    let root = std::env::current_dir().unwrap().join("fixtures");
    let jxl = output_path(&root.join("foto.jpg"), ConversionKind::JpegXl).unwrap();
    assert_eq!(jxl, root.join("foto.jxl"));
    let same = output_path(&root.join("foto.jxl"), ConversionKind::JpegXl).unwrap();
    assert_eq!(same, root.join("foto.converted.jxl"));
}

#[test]
fn resolvedor_tenta_candidatos_em_ordem_e_aceita_apenas_arquivo_regular() {
    let root = std::env::temp_dir().join(format!(
        "rovex-backend-resolver-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&root).expect("criar diretório do teste");
    let missing = root.join("missing");
    let backend = root.join("ffmpeg-real");
    fs::write(&backend, b"backend de teste").expect("criar backend de teste");
    let candidates = vec![missing, backend.clone(), backend.clone()];
    let resolved = resolve_backend_from_candidates("ffmpeg", &candidates)
        .expect("resolver deve avançar até o arquivo regular");
    assert_eq!(resolved.path, backend);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn override_sem_extensao_tambem_considera_executavel_dentro_da_pasta() {
    let directory = std::env::temp_dir().join("rovex-ffmpeg-install");
    let mut candidates = Vec::new();
    push_path_or_directory_candidates(&mut candidates, directory.clone(), "ffmpeg");
    assert!(candidates.contains(&directory));
    assert!(candidates.contains(&directory.with_extension("exe")));
    assert!(candidates.contains(&directory.join("ffmpeg")));
    assert!(candidates.contains(&directory.join("ffmpeg.exe")));
}

#[test]
fn resolvedor_recusa_caminho_relativo() {
    let backend = PathBuf::from("ffmpeg.exe");
    let error = resolve_backend_from_candidates("ffmpeg", std::slice::from_ref(&backend))
        .expect_err("o worker não deve receber caminho relativo");
    assert!(matches!(
        error,
        ConversionError::BackendUnavailable {
            executable: "ffmpeg",
            attempts: 1
        }
    ));
}

#[test]
fn resolvedor_recusa_diretorio_mesmo_com_nome_de_backend() {
    let root = std::env::temp_dir().join(format!(
        "rovex-backend-resolver-directory-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&root).expect("criar diretório do teste");
    let directory = root.join("ffmpeg.exe");
    fs::create_dir(&directory).expect("criar diretório com nome de executável");
    let error = resolve_backend_from_candidates("ffmpeg", std::slice::from_ref(&directory))
        .expect_err("diretório não pode ser tratado como backend");
    assert!(matches!(
        error,
        ConversionError::BackendUnavailable {
            executable: "ffmpeg",
            attempts: 1
        }
    ));
    let _ = fs::remove_dir_all(root);
}

#[cfg(unix)]
#[test]
fn resolvedor_aceita_link_para_backend_regular() {
    let root = std::env::temp_dir().join(format!(
        "rovex-backend-symlink-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&root).expect("criar diretório do teste");
    let backend = root.join("ffmpeg-real");
    let link = root.join("ffmpeg-link");
    fs::write(&backend, b"backend de teste").expect("criar backend de teste");
    symlink(&backend, &link).expect("criar link do backend");
    let resolved = resolve_backend_from_candidates("ffmpeg", std::slice::from_ref(&link))
        .expect("resolver deve aceitar link para arquivo regular");
    assert_eq!(resolved.path, link);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn resolvedor_retorna_erro_estruturado_com_numero_de_tentativas() {
    let candidates = vec![
        std::env::temp_dir().join("rovex-missing-ffmpeg-a"),
        std::env::temp_dir().join("rovex-missing-ffmpeg-b"),
    ];
    let error = resolve_backend_from_candidates("ffmpeg", &candidates)
        .expect_err("nenhum candidato deve ser tratado como backend");
    assert!(matches!(
        error,
        ConversionError::BackendUnavailable {
            executable: "ffmpeg",
            attempts: 2
        }
    ));
}

#[test]
fn cancelamento_antes_do_backend_nao_publica_saida() {
    let source = std::env::temp_dir().join(format!(
        "rovex-converter-cancel-{}-{}.png",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::write(&source, b"entrada de teste").expect("criar origem temporária");
    let cancel = AtomicBool::new(true);
    let error = convert_file(&source, ConversionKind::JpegXl, &cancel, |_| {})
        .expect_err("cancelamento deve impedir conversão");
    assert!(matches!(error, super::ConversionError::Cancelled));
    assert!(!source.with_extension("jxl").exists());
    let _ = fs::remove_file(source);
}

#[test]
#[ignore = "requer FFmpeg e ffprobe instalados no ambiente"]
fn conversoes_reais_publicam_saidas_validadas_pelo_ffprobe() {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("relógio monotônico disponível")
        .as_nanos();
    let directory = std::env::temp_dir().join(format!(
        "rovex-converter-test-{}-{timestamp}",
        std::process::id()
    ));
    fs::create_dir(&directory).expect("criar diretório temporário");
    let image = directory.join("entrada.png");
    let audio = directory.join("entrada.wav");
    let ffmpeg = resolve_backend("ffmpeg").expect("resolver encontrar ffmpeg para fixture");
    let create_image = Command::new(&ffmpeg.path)
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-nostdin",
            "-f",
            "lavfi",
            "-i",
            "color=c=blue:s=8x8",
            "-frames:v",
            "1",
        ])
        .arg(&image)
        .status()
        .expect("executar ffmpeg para a imagem");
    assert!(create_image.success());
    let create_audio = Command::new(&ffmpeg.path)
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-nostdin",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=440:duration=0.15",
            "-c:a",
            "pcm_s16le",
        ])
        .arg(&audio)
        .status()
        .expect("executar ffmpeg para o áudio");
    assert!(create_audio.success());

    let cancel = AtomicBool::new(false);
    let jxl = convert_file(&image, ConversionKind::JpegXl, &cancel, |_| {})
        .expect("converter imagem para JXL");
    assert!(jxl.destination.is_file());
    assert!(fs::metadata(&jxl.destination).unwrap().len() > 0);
    let png = convert_file(&image, ConversionKind::Png, &cancel, |_| {})
        .expect("converter imagem para PNG");
    assert!(png.destination.is_file());
    let opus = convert_file(&audio, ConversionKind::Opus, &cancel, |_| {})
        .expect("converter áudio para Opus");
    assert!(opus.destination.is_file());
    let flac = convert_file(&audio, ConversionKind::Flac, &cancel, |_| {})
        .expect("converter áudio para FLAC");
    assert!(flac.destination.is_file());

    let second = convert_file(&image, ConversionKind::JpegXl, &cancel, |_| {})
        .expect_err("recusar saída JXL já existente");
    assert!(matches!(
        second,
        super::ConversionError::OutputExists { .. }
    ));
    let _ = fs::remove_dir_all(&directory);
}
