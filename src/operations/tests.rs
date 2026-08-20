use super::{
    OperationError, copy_file_atomic, copy_file_atomic_with_progress, create_directory,
    delete_entry, publish_file_no_replace, rename_entry,
};
use std::fs;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

fn temporary_directory() -> std::path::PathBuf {
    let unique = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "rovex-operation-test-{}-{unique}",
        std::process::id()
    ));
    fs::create_dir(&path).expect("o diretório deve ser criado");
    path
}

#[test]
fn mensagem_de_operacao_negada_e_humanizada() {
    let error = OperationError::FileSystem {
        operation: "copiar arquivo",
        path: std::path::PathBuf::from("/protegido.txt"),
        kind: std::io::ErrorKind::PermissionDenied,
        raw_os_error: Some(5),
    };
    assert_eq!(
        error.to_string(),
        "não foi possível copiar arquivo em /protegido.txt: o acesso foi negado"
    );
}

#[test]
fn copia_e_publica_arquivo_somente_apos_validacao() {
    let root = temporary_directory();
    let source = root.join("origem.txt");
    let destination = root.join("destino.txt");
    fs::write(&source, b"dados locais").expect("a origem deve ser criada");

    let report = copy_file_atomic(&source, &destination).expect("a cópia deve funcionar");
    assert_eq!(report.bytes_copied, 12);
    assert_eq!(
        fs::read(&destination).expect("o destino deve existir"),
        b"dados locais"
    );
    assert!(!root.join(".destino.txt.rovex-tmp").exists());
    fs::remove_dir_all(root).expect("o diretório deve ser removido");
}

#[test]
fn copia_reporta_progresso_e_publica_ate_o_fim() {
    let root = temporary_directory();
    let source = root.join("origem-grande.bin");
    let destination = root.join("destino-grande.bin");
    let contents = vec![0x5a; 128 * 1024];
    fs::write(&source, &contents).expect("a origem deve ser criada");
    let cancel = AtomicBool::new(false);
    let mut updates = Vec::new();

    let report = copy_file_atomic_with_progress(&source, &destination, &cancel, |progress| {
        updates.push(progress);
    })
    .expect("a cópia com progresso deve funcionar");

    assert_eq!(report.bytes_copied, contents.len() as u64);
    assert!(!updates.is_empty());
    assert_eq!(updates.last().unwrap().bytes_copied, contents.len() as u64);
    assert_eq!(fs::read(&destination).unwrap(), contents);
    fs::remove_dir_all(root).expect("o diretório deve ser removido");
}

#[test]
fn cancelamento_nao_publica_destino_parcial() {
    let root = temporary_directory();
    let source = root.join("origem-cancelada.bin");
    let destination = root.join("destino-cancelado.bin");
    fs::write(&source, vec![0x33; 128 * 1024]).expect("a origem deve ser criada");
    let cancel = AtomicBool::new(false);

    let result = copy_file_atomic_with_progress(&source, &destination, &cancel, |_| {
        cancel.store(true, Ordering::Release);
    });

    assert!(matches!(result, Err(OperationError::Cancelled)));
    assert!(!destination.exists());
    assert!(!fs::read_dir(&root).unwrap().any(|entry| {
        entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .contains("rovex-tmp")
    }));
    fs::remove_dir_all(root).expect("o diretório deve ser removido");
}

#[test]
fn nao_sobrescreve_destino_existente() {
    let root = temporary_directory();
    let source = root.join("origem.txt");
    let destination = root.join("destino.txt");
    fs::write(&source, b"novo").expect("a origem deve ser criada");
    fs::write(&destination, b"antigo").expect("o destino deve ser criado");

    let result = copy_file_atomic(&source, &destination);
    assert!(matches!(result, Err(OperationError::Validation(_))));
    assert_eq!(
        fs::read(&destination).expect("o destino deve permanecer"),
        b"antigo"
    );
    fs::remove_dir_all(root).expect("o diretório deve ser removido");
}

#[test]
fn publicacao_atomica_nao_sobrescreve_destino_criado_depois_da_validacao() {
    let root = temporary_directory();
    let temporary = root.join(".destino.rovex-tmp");
    let destination = root.join("destino.txt");
    fs::write(&temporary, b"novo").expect("o temporário deve ser criado");
    fs::write(&destination, b"antigo").expect("o destino deve ser criado");

    let result = publish_file_no_replace(&temporary, &destination);
    assert!(matches!(result, Err(OperationError::FileSystem { .. })));
    assert_eq!(
        fs::read(&destination).expect("o destino deve permanecer"),
        b"antigo"
    );
    assert!(temporary.exists());
    fs::remove_dir_all(root).expect("o diretório deve ser removido");
}

#[test]
fn fallback_nao_remove_destino_preexistente_quando_create_new_falha() {
    let root = temporary_directory();
    let temporary = root.join(".destino.rovex-tmp");
    let destination = root.join("destino.txt");
    fs::write(&temporary, b"novo").expect("o temporário deve ser criado");
    fs::write(&destination, b"antigo").expect("o destino deve ser criado");

    let result = super::copy_temporary_no_replace(&temporary, &destination);
    assert!(matches!(result, Err(OperationError::FileSystem { .. })));
    assert_eq!(
        fs::read(&destination).expect("o destino deve permanecer"),
        b"antigo"
    );
    assert!(temporary.exists());
    fs::remove_dir_all(root).expect("o diretório deve ser removido");
}

#[test]
fn renomeia_cria_e_exclui_entrada() {
    let root = temporary_directory();
    let directory = root.join("nova-pasta");
    create_directory(&directory).expect("a pasta deve ser criada");
    let source = root.join("origem.txt");
    let destination = root.join("renomeado.txt");
    fs::write(&source, b"conteudo").expect("a origem deve ser criada");
    rename_entry(&source, &destination).expect("a entrada deve ser renomeada");
    delete_entry(&destination).expect("o arquivo deve ser excluído");
    delete_entry(&directory).expect("a pasta vazia deve ser excluída");
    fs::remove_dir_all(root).expect("o diretório deve ser removido");
}

#[test]
fn copia_e_renomeia_nomes_unicode_e_com_espacos() {
    let root = temporary_directory();
    let source = root.join("origem — relatório 🧪.txt");
    let copied = root.join("cópia com espaços.txt");
    let renamed = root.join("resultado final — 2.txt");
    fs::write(&source, b"conteudo Unicode").expect("a origem Unicode deve ser criada");

    copy_file_atomic(&source, &copied).expect("a cópia Unicode deve funcionar");
    rename_entry(&copied, &renamed).expect("a renomeação Unicode deve funcionar");
    assert_eq!(
        fs::read(&renamed).expect("o destino deve existir"),
        b"conteudo Unicode"
    );
    delete_entry(&source).expect("a origem deve ser excluída");
    delete_entry(&renamed).expect("o resultado deve ser excluído");
    fs::remove_dir_all(root).expect("o diretório deve ser removido");
}

#[cfg(windows)]
#[test]
fn nomes_reservados_do_windows_sao_rejeitados_pelo_sistema() {
    let root = temporary_directory();
    for name in [
        "CON",
        "PRN",
        "AUX",
        "NUL",
        "COM1",
        "COM9",
        "COM¹",
        "COM².txt",
        "COM³",
        "LPT1",
        "LPT9.log",
        "LPT¹",
        "LPT².txt",
        "LPT³",
    ] {
        let path = root.join(name);
        assert!(
            create_directory(&path).is_err(),
            "o nome reservado {name} não deve ser criado"
        );
    }
    fs::remove_dir_all(root).expect("o diretório deve ser removido");
}

#[test]
fn nao_exclui_diretorio_nao_vazio() {
    let root = temporary_directory();
    let directory = root.join("com-arquivo");
    create_directory(&directory).expect("a pasta deve ser criada");
    fs::write(directory.join("arquivo.txt"), b"conteudo")
        .expect("o arquivo interno deve ser criado");

    let result = delete_entry(&directory);
    assert!(matches!(
        result,
        Err(OperationError::DirectoryNotEmpty { .. })
    ));
    assert!(directory.exists());
    fs::remove_dir_all(root).expect("o diretório deve ser removido");
}
