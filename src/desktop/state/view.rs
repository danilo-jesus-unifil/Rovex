use super::super::{FileRow, MainWindow};
use super::listing::format_timestamp;
use super::{LoadedRow, SelectionState, SharedRows, SharedSelection, SortSpec, sort_rows};
use slint::{Model, SharedString, VecModel};
use std::path::PathBuf;

pub(in crate::desktop) fn filter_rows(
    rows: &[LoadedRow],
    query: &str,
    sort_spec: SortSpec,
) -> Vec<LoadedRow> {
    let normalized_query = query.trim().to_lowercase();
    let mut filtered = if normalized_query.is_empty() {
        rows.to_vec()
    } else {
        rows.iter()
            .filter(|row| row.name.to_lowercase().contains(&normalized_query))
            .cloned()
            .collect()
    };
    sort_rows(&mut filtered, sort_spec);
    filtered
}

pub(in crate::desktop) fn empty_state_text(
    total: usize,
    visible: usize,
    query: &str,
) -> &'static str {
    if total == 0 {
        "Esta pasta está vazia."
    } else if visible == 0 && !query.trim().is_empty() {
        "Nenhum item corresponde ao filtro."
    } else {
        ""
    }
}

pub(in crate::desktop) fn filter_status(total: usize, visible: usize, query: &str) -> String {
    if query.trim().is_empty() {
        return format!("{total} itens");
    }
    if visible == 0 {
        return format!(
            "Nenhum item corresponde a ‘{}’ ({total} itens na pasta)",
            query.trim()
        );
    }
    format!("{visible} de {total} itens")
}
pub(in crate::desktop) fn set_rows(
    ui: &MainWindow,
    rows: Vec<LoadedRow>,
    selection: &SelectionState,
) -> bool {
    let entries = ui.get_entries();
    let Some(model) = entries.as_any().downcast_ref::<VecModel<FileRow>>() else {
        return false;
    };

    model.set_vec(
        rows.into_iter()
            .map(|row| FileRow {
                selected: selection.selected.contains(&row.key),
                key: SharedString::from(row.key),
                name: SharedString::from(row.name),
                kind: SharedString::from(row.kind),
                icon: SharedString::from(row.icon),
                details: SharedString::from(row.details),
                modified: SharedString::from(format_timestamp(row.modified)),
                created: SharedString::from(format_timestamp(row.created)),
                accessed: SharedString::from(format_timestamp(row.accessed)),
                is_directory: row.is_directory,
            })
            .collect::<Vec<_>>(),
    );
    true
}

pub(in crate::desktop) fn update_selection_visuals(
    ui: &MainWindow,
    selection: &SelectionState,
) -> bool {
    let entries = ui.get_entries();
    let Some(model) = entries.as_any().downcast_ref::<VecModel<FileRow>>() else {
        return false;
    };

    for index in 0..model.row_count() {
        let Some(mut row) = model.row_data(index) else {
            continue;
        };
        let selected = selection.selected.contains(row.key.as_str());
        if row.selected != selected {
            row.selected = selected;
            model.set_row_data(index, row);
        }
    }
    true
}

pub(in crate::desktop) fn selection_status(selection: &SelectionState) -> String {
    match selection.count() {
        0 => String::new(),
        1 => "1 item selecionado".to_owned(),
        count => format!("{count} itens selecionados"),
    }
}

pub(in crate::desktop) fn selected_paths(
    rows: &SharedRows,
    selection: &SharedSelection,
) -> Vec<PathBuf> {
    let Ok(selection) = selection.lock() else {
        return Vec::new();
    };
    let Ok(rows) = rows.lock() else {
        return Vec::new();
    };
    rows.iter()
        .filter(|row| selection.selected.contains(&row.key))
        .map(|row| row.path.clone())
        .collect()
}

pub(in crate::desktop) fn validate_rename_name(name: &str) -> Result<String, &'static str> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("o novo nome não pode ser vazio");
    }
    if trimmed == "."
        || trimmed == ".."
        || trimmed.contains('/')
        || trimmed.contains('\\')
        || trimmed.contains('\0')
    {
        return Err("o novo nome deve ser um único nome de arquivo");
    }
    Ok(trimmed.to_owned())
}
