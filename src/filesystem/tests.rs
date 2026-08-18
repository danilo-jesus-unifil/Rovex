use super::{EntryKind, FileSystem, ListingOptions};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

fn temporary_directory() -> std::path::PathBuf {
    let unique = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "rovex-filesystem-test-{}-{unique}",
        std::process::id()
    ));
    fs::create_dir(&path).expect("o diretório temporário deve ser criado");
    path
}

#[test]
fn mensagem_de_acesso_negado_e_humanizada() {
    let error = super::FileSystemError::Io {
        operation: "ler diretório",
        path: std::path::PathBuf::from("/protegido"),
        kind: std::io::ErrorKind::PermissionDenied,
        raw_os_error: Some(13),
    };
    assert_eq!(
        error.to_string(),
        "não foi possível ler diretório em /protegido: o acesso foi negado"
    );
}

#[test]
fn lista_diretorios_antes_de_arquivos() {
    let root = temporary_directory();
    fs::create_dir(root.join("Pasta")).expect("a pasta deve ser criada");
    fs::write(root.join("arquivo.txt"), b"conteudo").expect("o arquivo deve ser criado");

    let entries = FileSystem
        .list_directory(&root)
        .expect("a listagem deve funcionar");

    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].kind, EntryKind::Directory);
    assert_eq!(entries[1].kind, EntryKind::File);
    fs::remove_dir_all(root).expect("o diretório de teste deve ser removido");
}

#[test]
fn rejeita_caminho_que_nao_e_diretorio() {
    let root = temporary_directory();
    let file = root.join("arquivo.txt");
    fs::write(&file, b"conteudo").expect("o arquivo deve ser criado");

    let result = FileSystem.list_directory(&file);
    assert!(matches!(
        result,
        Err(super::FileSystemError::NotDirectory { .. })
    ));
    fs::remove_dir_all(root).expect("o diretório de teste deve ser removido");
}

#[test]
fn lista_preserva_nome_unicode_espacos_e_pontuacao() {
    let root = temporary_directory();
    let name = "relatório com espaços — versão 2.0 🧪.txt";
    let file = root.join(name);
    fs::write(&file, b"conteudo").expect("o arquivo Unicode deve ser criado");

    let entries = FileSystem
        .list_directory(&root)
        .expect("a listagem com nome Unicode deve funcionar");
    let entry = entries
        .iter()
        .find(|entry| entry.path == file)
        .expect("o arquivo Unicode deve aparecer na listagem");
    assert_eq!(entry.display_name(), name);
    assert_eq!(entry.size, Some(8));
    fs::remove_dir_all(root).expect("o diretório de teste deve ser removido");
}

#[cfg(not(windows))]
#[test]
fn lista_preserva_ponto_final_em_sistemas_que_o_suportam() {
    let root = temporary_directory();
    let file = root.join("nome-com-ponto-final.");
    fs::write(&file, b"conteudo").expect("o arquivo com ponto final deve ser criado");

    let entries = FileSystem
        .list_directory(&root)
        .expect("a listagem do nome com ponto final deve funcionar");
    assert!(entries.iter().any(|entry| entry.path == file));
    fs::remove_dir_all(root).expect("o diretório de teste deve ser removido");
}

#[cfg(not(windows))]
#[test]
fn lista_caminho_com_muitos_componentes_sem_truncar() {
    let root = temporary_directory();
    let mut nested = root.clone();
    for index in 0..24 {
        nested.push(format!("segmento-{index:02}-abcdefgh"));
    }
    fs::create_dir_all(&nested).expect("o caminho aninhado deve ser criado");
    let file = nested.join("arquivo-final.txt");
    fs::write(&file, b"conteudo").expect("o arquivo aninhado deve ser criado");

    let entries = FileSystem
        .list_directory(&nested)
        .expect("a listagem do caminho aninhado deve funcionar");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].path, file);
    assert!(file.as_os_str().len() > 260);
    fs::remove_dir_all(root).expect("o diretório de teste deve ser removido");
}

#[cfg(unix)]
#[test]
fn identifica_link_sem_seguir_destino() {
    use std::os::unix::fs::symlink;

    let root = temporary_directory();
    let target = root.join("destino.txt");
    let link = root.join("atalho.txt");
    fs::write(&target, b"conteudo").expect("o destino deve ser criado");
    symlink(&target, &link).expect("o link deve ser criado");

    let entries = FileSystem
        .list_directory(&root)
        .expect("a listagem deve funcionar");
    let link_entry = entries
        .iter()
        .find(|entry| entry.path == link)
        .expect("o link deve aparecer na listagem");
    assert_eq!(link_entry.kind, EntryKind::Symlink);
    assert_eq!(link_entry.size, None);
    fs::remove_dir_all(root).expect("o diretório de teste deve ser removido");
}

#[test]
fn oculta_nomes_de_ponto_por_padrao_e_mostra_com_opcao() {
    let root = temporary_directory();
    let visible = root.join("visivel.txt");
    let hidden = root.join(".segredo.txt");
    fs::write(&visible, b"visivel").expect("o arquivo visível deve ser criado");
    fs::write(&hidden, b"oculto").expect("o arquivo oculto deve ser criado");

    let default_entries = FileSystem
        .list_directory(&root)
        .expect("a listagem padrão deve funcionar");
    assert!(default_entries.iter().any(|entry| entry.path == visible));
    assert!(!default_entries.iter().any(|entry| entry.path == hidden));

    let shown_entries = FileSystem
        .list_directory_with_options(
            &root,
            ListingOptions {
                show_hidden: true,
                show_system: false,
            },
        )
        .expect("a listagem com ocultos deve funcionar");
    let hidden_entry = shown_entries
        .iter()
        .find(|entry| entry.path == hidden)
        .expect("o arquivo oculto deve aparecer quando solicitado");
    assert!(hidden_entry.is_hidden);
    assert!(!hidden_entry.is_system);
    fs::remove_dir_all(root).expect("o diretório de teste deve ser removido");
}
