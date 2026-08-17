use super::super::locations::default_locations;
use super::{
    LoadedRow, NavigationHistory, SelectionState, TabManager, empty_state_text, filter_rows,
    filter_status, format_size, load_directory, parent_directory, row_icon, validate_rename_name,
};
use crate::filesystem::EntryKind;

use std::path::{Path, PathBuf};

#[test]
fn selecao_ctrl_shift_e_ctrl_a_mantem_intervalos_reais() {
    let keys = vec![
        "a".to_owned(),
        "b".to_owned(),
        "c".to_owned(),
        "d".to_owned(),
    ];
    let mut selection = SelectionState::default();

    selection.click("b", &keys, false, false);
    assert_eq!(selection.count(), 1);
    assert!(selection.selected.contains("b"));

    selection.click("d", &keys, true, false);
    assert_eq!(selection.count(), 2);
    assert!(selection.selected.contains("b"));
    assert!(selection.selected.contains("d"));

    selection.click("a", &keys, false, true);
    assert_eq!(selection.count(), 4);
    assert!(selection.selected.contains("a"));
    assert!(selection.selected.contains("b"));
    assert!(selection.selected.contains("c"));
    assert!(selection.selected.contains("d"));

    selection.select_all(keys.clone());
    assert_eq!(selection.count(), keys.len());
}

#[test]
fn renomear_recusa_traversal_e_preserva_nome_unicode() {
    assert!(validate_rename_name("").is_err());
    assert!(validate_rename_name("..").is_err());
    assert!(validate_rename_name("pasta/arquivo.txt").is_err());
    assert!(validate_rename_name("pasta\\arquivo.txt").is_err());
    assert_eq!(
        validate_rename_name(" relatório final.txt"),
        Ok("relatório final.txt".to_owned())
    );
}

#[test]
fn historico_navega_para_tras_e_para_frente_e_limpa_futuro() {
    let mut history = NavigationHistory::new(Path::new("/inicio").to_path_buf());
    assert!(!history.can_go_back());
    assert!(!history.can_go_forward());

    assert!(history.visit(Path::new("/projetos").to_path_buf()));
    assert!(history.can_go_back());
    assert_eq!(history.go_back(), Some(Path::new("/inicio").to_path_buf()));
    assert!(history.can_go_forward());
    assert_eq!(
        history.go_forward(),
        Some(Path::new("/projetos").to_path_buf())
    );

    assert!(history.visit(Path::new("/documentos").to_path_buf()));
    assert!(!history.can_go_forward());
}

#[test]
fn abas_preservam_historicos_independentes_e_nao_fecham_a_ultima() {
    let mut tabs = TabManager::new(Path::new("/inicio").to_path_buf());
    tabs.active_mut()
        .visit(Path::new("/projetos").to_path_buf());
    tabs.new_tab(Path::new("/documentos").to_path_buf());
    assert_eq!(tabs.histories.len(), 2);
    assert_eq!(tabs.active().current, Path::new("/documentos"));
    assert!(tabs.select(0));
    assert_eq!(tabs.active().current, Path::new("/projetos"));
    assert!(tabs.active().can_go_back());
    assert!(tabs.close(1));
    assert!(!tabs.close(0));
    assert_eq!(tabs.histories.len(), 1);
}

#[test]
fn icones_semanticos_diferenciam_pasta_arquivo_e_extensoes() {
    assert_eq!(
        row_icon("Fotos", EntryKind::Directory),
        ("▰", "Pasta", true)
    );
    assert_eq!(
        row_icon("imagem.png", EntryKind::File),
        ("◉", "Arquivo", false)
    );
    assert_eq!(
        row_icon("main.rs", EntryKind::File),
        ("Rs", "Arquivo", false)
    );
    assert_eq!(
        row_icon("atalho", EntryKind::Symlink),
        ("↗", "Link simbólico", false)
    );
}

#[test]
fn locais_padrao_so_incluem_diretorios_existentes() {
    let locations = default_locations(Path::new("."));
    assert!(
        locations
            .iter()
            .any(|location| location.path == Path::new("."))
    );
    assert!(locations.iter().all(|location| !location.label.is_empty()));
    assert!(locations.iter().all(|location| location.path.is_dir()));
}

#[test]
fn filtro_localiza_nome_sem_varrer_subpastas() {
    let rows = vec![
        LoadedRow {
            key: "foto".to_owned(),
            path: PathBuf::from("foto"),
            name: "Foto.JPG".to_owned(),
            kind: "Arquivo".to_owned(),
            icon: "●".to_owned(),
            details: "4 KB".to_owned(),
            is_directory: false,
        },
        LoadedRow {
            key: "projetos".to_owned(),
            path: PathBuf::from("projetos"),
            name: "Projetos".to_owned(),
            kind: "Pasta".to_owned(),
            icon: "▰".to_owned(),
            details: "—".to_owned(),
            is_directory: true,
        },
    ];

    let filtered = filter_rows(&rows, "jpg");
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].name, "Foto.JPG");
    assert_eq!(filter_rows(&rows, "   ").len(), 2);
    assert_eq!(filter_status(2, 1, "jpg"), "1 de 2 itens");
}

#[test]
#[ignore = "benchmark manual de performance"]
fn benchmark_filtro_100k() {
    use std::time::Instant;

    let rows = (0..100_000)
        .map(|index| LoadedRow {
            key: format!("/tmp/file-{index:05}.txt"),
            path: PathBuf::from(format!("/tmp/file-{index:05}.txt")),
            name: format!("file-{index:05}.txt"),
            kind: "Arquivo".to_owned(),
            icon: "●".to_owned(),
            details: "1 B".to_owned(),
            is_directory: false,
        })
        .collect::<Vec<_>>();
    let started = Instant::now();
    let filtered = filter_rows(&rows, "99999");
    let elapsed = started.elapsed();
    eprintln!(
        "benchmark_filter_100k elapsed_ms={} matches={}",
        elapsed.as_secs_f64() * 1000.0,
        filtered.len()
    );
    assert_eq!(filtered.len(), 1);
}

#[test]
fn estados_vazios_diferenciam_pasta_e_filtro() {
    assert_eq!(empty_state_text(0, 0, ""), "Esta pasta está vazia.");
    assert_eq!(
        empty_state_text(4, 0, "pdf"),
        "Nenhum item corresponde ao filtro."
    );
    assert_eq!(empty_state_text(4, 4, "pdf"), "");
}

#[test]
fn filtro_sem_resultado_exibe_estado_vazio_controlado() {
    let status = filter_status(4, 0, "pdf");
    assert_eq!(status, "Nenhum item corresponde a ‘pdf’ (4 itens na pasta)");
}

#[test]
fn formata_tamanho_sem_overflow_visual() {
    assert_eq!(format_size(0), "0 B");
    assert_eq!(format_size(1024), "1.0 KB");
    assert_eq!(format_size(1024 * 1024), "1.0 MB");
}

#[test]
fn encontra_pasta_pai_sem_transformar_raiz_em_pasta_vazia() {
    assert_eq!(
        parent_directory(Path::new("/tmp/rovex")),
        Some(Path::new("/tmp").to_path_buf())
    );
    assert_eq!(parent_directory(Path::new("/")), None);
}

#[test]
fn carrega_diretorio_real_sem_fingir_sucesso() {
    let path = std::env::current_dir().expect("o diretório atual deve existir");
    let loaded = load_directory(path);
    assert!(!loaded.rows.is_empty());
    assert!(loaded.status.ends_with("itens"));
}

#[cfg(unix)]
#[test]
fn preserva_caminhos_de_nomes_invalidos_sem_colidir_chaves() {
    use std::ffi::OsString;
    use std::fs;
    use std::os::unix::ffi::OsStringExt;

    let root = std::env::temp_dir().join(format!("rovex-invalid-name-test-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir(&root).expect("a pasta de teste deve ser criada");
    let first = root.join(OsString::from_vec(vec![0xff, b'.', b't', b'x', b't']));
    let second = root.join(OsString::from_vec(vec![0xfe, b'.', b't', b'x', b't']));
    fs::write(&first, b"a").expect("o primeiro arquivo deve ser criado");
    fs::write(&second, b"b").expect("o segundo arquivo deve ser criado");

    let loaded = load_directory(root.clone());
    assert_eq!(loaded.rows.len(), 2);
    assert_ne!(loaded.rows[0].key, loaded.rows[1].key);
    assert_ne!(loaded.rows[0].path, loaded.rows[1].path);
    fs::remove_dir_all(root).expect("a pasta de teste deve ser removida");
}

#[test]
fn erro_de_diretorio_inexistente_vira_status_controlado() {
    let path = std::env::temp_dir().join("rovex-path-that-does-not-exist");
    let loaded = load_directory(path);
    assert!(loaded.rows.is_empty());
    assert!(loaded.is_error);
    assert!(
        loaded
            .status
            .starts_with("Não foi possível listar a pasta:")
    );
}
