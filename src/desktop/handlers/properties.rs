use super::super::context::AppContext;
use super::super::state::{LoadedRow, SharedRows, SharedSelection};
use slint::{ModelRc, SharedString, VecModel};
use std::rc::Rc;
use std::sync::Arc;

fn selected_row(rows: &SharedRows, selection: &SharedSelection) -> Option<LoadedRow> {
    let selected = selection.lock().ok()?.selected.clone();
    if selected.len() != 1 {
        return None;
    }
    let rows = rows.lock().ok()?;
    rows.iter().find(|row| selected.contains(&row.key)).cloned()
}

fn property_lines(row: &LoadedRow) -> Vec<SharedString> {
    let location = row
        .path
        .parent()
        .map(|path| path.to_string_lossy().into_owned())
        .unwrap_or_else(|| "—".to_owned());
    let attributes = if row.kind == "Item do sistema" {
        "Sistema"
    } else if row.kind == "Pasta oculta" || row.kind == "Arquivo oculto" {
        "Oculto"
    } else if row.kind == "Link simbólico" {
        "Link simbólico"
    } else {
        "Normal"
    };
    [
        "General".to_owned(),
        format!("Nome: {}", row.name),
        format!("Tipo: {}", row.kind),
        format!("Localização: {location}"),
        format!("Tamanho: {}", row.details),
        format!(
            "Criado: {}",
            super::super::state::format_timestamp(row.created)
        ),
        format!(
            "Modificado: {}",
            super::super::state::format_timestamp(row.modified)
        ),
        format!(
            "Acessado: {}",
            super::super::state::format_timestamp(row.accessed)
        ),
        format!("Atributos: {attributes}"),
        "Security".to_owned(),
        "Este diálogo somente lê metadata; nenhuma permissão é alterada.".to_owned(),
        "Links e reparse points não são seguidos para exibir estas informações.".to_owned(),
        "Details".to_owned(),
        format!("Chave interna: {}", row.key),
        format!(
            "Diretório: {}",
            if row.is_directory { "sim" } else { "não" }
        ),
    ]
    .into_iter()
    .map(SharedString::from)
    .collect()
}

pub(in crate::desktop) fn register(ctx: &AppContext) {
    let Some(registration_ui) = ctx.ui_weak.upgrade() else {
        return;
    };
    let ui_weak = ctx.ui_weak.clone();
    let rows = Arc::clone(&ctx.directory_rows);
    let selection = Arc::clone(&ctx.selection);

    registration_ui.on_context_menu_properties_requested(move || {
        let Some(ui) = ui_weak.upgrade() else {
            return;
        };
        ui.set_context_menu_visible(false);
        let Some(row) = selected_row(&rows, &selection) else {
            ui.set_status_text("Selecione exatamente um item para ver propriedades".into());
            return;
        };
        ui.set_properties_title(SharedString::from(format!("Propriedades — {}", row.name)));
        ui.set_properties_lines(ModelRc::from(Rc::new(VecModel::from(property_lines(&row)))));
        ui.set_properties_visible(true);
    });
}
