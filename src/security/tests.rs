use super::{DestinationPolicy, ValidationError, ensure_not_root, validate_destination};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

fn temporary_directory() -> std::path::PathBuf {
    let unique = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "rovex-security-test-{}-{unique}",
        std::process::id()
    ));
    fs::create_dir(&path).expect("o diretório deve ser criado");
    path
}

#[test]
fn recusa_destino_existente_por_padrao() {
    let root = temporary_directory();
    let destination = root.join("existente.txt");
    fs::write(&destination, b"conteudo").expect("o arquivo deve ser criado");

    let result = validate_destination(None, &destination, DestinationPolicy::default());
    assert!(matches!(
        result,
        Err(ValidationError::ExistingDestination { .. })
    ));
    fs::remove_dir_all(root).expect("o diretório deve ser removido");
}

#[test]
fn recusa_raiz() {
    let result = ensure_not_root(std::path::Path::new("/"));
    assert!(matches!(
        result,
        Err(ValidationError::RootOperationDenied { .. })
    ));
}

#[test]
fn normaliza_parent_no_diretorio_pai() {
    let root = temporary_directory();
    let nested = root.join("nested");
    fs::create_dir(&nested).expect("a pasta aninhada deve ser criada");
    let destination = nested.join("..").join("destino.txt");

    let normalized = validate_destination(None, &destination, DestinationPolicy::default())
        .expect("o destino deve ser normalizado");
    let expected = fs::canonicalize(&root)
        .expect("a raiz do teste deve ser normalizável")
        .join("destino.txt");
    assert_eq!(normalized, expected);
    fs::remove_dir_all(root).expect("o diretório deve ser removido");
}

#[test]
fn detecta_mesma_origem_mesmo_com_caminho_equivalente() {
    let root = temporary_directory();
    let nested = root.join("nested");
    let source = root.join("origem.txt");
    fs::create_dir(&nested).expect("a pasta aninhada deve ser criada");
    fs::write(&source, b"conteudo").expect("a origem deve ser criada");
    let equivalent_destination = nested.join("..").join("origem.txt");

    let result = validate_destination(
        Some(&source),
        &equivalent_destination,
        DestinationPolicy::default(),
    );
    assert!(matches!(
        result,
        Err(ValidationError::SameSourceAndDestination { .. })
    ));
    fs::remove_dir_all(root).expect("o diretório deve ser removido");
}

#[test]
fn recusa_destino_relativo_com_mensagem_clara() {
    let result = validate_destination(
        None,
        std::path::Path::new("destino.txt"),
        DestinationPolicy::default(),
    );
    assert!(matches!(
        result,
        Err(ValidationError::InvalidPath {
            reason: "caminho relativo ambíguo; use um caminho absoluto",
            ..
        })
    ));
}

#[cfg(windows)]
#[test]
fn recusa_componente_junction_no_diretorio_pai() {
    use std::process::Command;

    let root = temporary_directory();
    let outside = temporary_directory();
    let junction = root.join("junction");
    let command = format!(
        "mklink /J \"{}\" \"{}\"",
        junction.display(),
        outside.display()
    );
    let status = Command::new("cmd.exe")
        .args(["/D", "/C", &command])
        .status()
        .expect("o cmd.exe deve estar disponível no Windows");
    if !status.success() {
        fs::remove_dir_all(root).expect("a raiz do teste deve ser removida");
        fs::remove_dir_all(outside).expect("o destino externo deve ser removido");
        return;
    }

    let result = validate_destination(
        None,
        &junction.join("novo.txt"),
        DestinationPolicy::default(),
    );
    assert!(matches!(
        result,
        Err(ValidationError::InvalidPath {
            reason: "componente reparse point no diretório pai",
            ..
        })
    ));
    fs::remove_dir_all(root).expect("a raiz do teste deve ser removida");
    fs::remove_dir_all(outside).expect("o destino externo deve ser removido");
}

#[cfg(unix)]
#[test]
fn recusa_componente_symlink_no_diretorio_pai() {
    use std::os::unix::fs::symlink;

    let root = temporary_directory();
    let outside = temporary_directory();
    let link = root.join("atalho");
    symlink(&outside, &link).expect("o symlink deve ser criado");
    let result = validate_destination(None, &link.join("novo.txt"), DestinationPolicy::default());
    assert!(matches!(result, Err(ValidationError::InvalidPath { .. })));
    fs::remove_dir_all(root).expect("a raiz do teste deve ser removida");
    fs::remove_dir_all(outside).expect("o destino externo deve ser removido");
}

#[test]
fn recusa_componente_final_ambiguo() {
    let root = temporary_directory();
    let result = validate_destination(None, &root.join(".."), DestinationPolicy::default());
    assert!(matches!(result, Err(ValidationError::InvalidPath { .. })));
    fs::remove_dir_all(root).expect("o diretório deve ser removido");
}
