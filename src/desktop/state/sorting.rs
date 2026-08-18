use super::LoadedRow;
use std::cmp::Ordering;
use std::time::SystemTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::desktop) enum SortField {
    Type,
    Name,
    Size,
    Modified,
    Created,
    Accessed,
}

impl SortField {
    pub(in crate::desktop) fn from_column(column: i32) -> Option<Self> {
        match column {
            0 => Some(Self::Type),
            1 => Some(Self::Name),
            2 => Some(Self::Size),
            3 => Some(Self::Modified),
            4 => Some(Self::Created),
            5 => Some(Self::Accessed),
            _ => None,
        }
    }

    pub(in crate::desktop) const fn column(self) -> i32 {
        match self {
            Self::Type => 0,
            Self::Name => 1,
            Self::Size => 2,
            Self::Modified => 3,
            Self::Created => 4,
            Self::Accessed => 5,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::desktop) enum SortDirection {
    Ascending,
    Descending,
}

impl SortDirection {
    pub(in crate::desktop) const fn toggled(self) -> Self {
        match self {
            Self::Ascending => Self::Descending,
            Self::Descending => Self::Ascending,
        }
    }

    pub(in crate::desktop) const fn is_ascending(self) -> bool {
        matches!(self, Self::Ascending)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::desktop) struct SortSpec {
    pub(in crate::desktop) field: SortField,
    pub(in crate::desktop) direction: SortDirection,
}

impl Default for SortSpec {
    fn default() -> Self {
        Self {
            field: SortField::Name,
            direction: SortDirection::Ascending,
        }
    }
}

impl SortSpec {
    pub(in crate::desktop) fn toggle_column(self, column: i32) -> Self {
        let Some(field) = SortField::from_column(column) else {
            return self;
        };
        Self {
            direction: if field == self.field {
                self.direction.toggled()
            } else {
                SortDirection::Ascending
            },
            field,
        }
    }
}

fn compare_optional<T: Ord>(left: Option<T>, right: Option<T>) -> Ordering {
    left.cmp(&right)
}

fn compare_time(left: Option<SystemTime>, right: Option<SystemTime>) -> Ordering {
    compare_optional(left, right)
}

fn compare_rows(left: &LoadedRow, right: &LoadedRow, spec: SortSpec) -> Ordering {
    let directory_order = right.is_directory.cmp(&left.is_directory);
    if directory_order != Ordering::Equal {
        return directory_order;
    }

    let primary = match spec.field {
        SortField::Type => left
            .kind
            .to_ascii_lowercase()
            .cmp(&right.kind.to_ascii_lowercase()),
        SortField::Name => left.name.to_lowercase().cmp(&right.name.to_lowercase()),
        SortField::Size => compare_optional(left.size, right.size),
        SortField::Modified => compare_time(left.modified, right.modified),
        SortField::Created => compare_time(left.created, right.created),
        SortField::Accessed => compare_time(left.accessed, right.accessed),
    };
    let primary = if spec.direction.is_ascending() {
        primary
    } else {
        primary.reverse()
    };
    if primary != Ordering::Equal {
        return primary;
    }

    left.name
        .to_lowercase()
        .cmp(&right.name.to_lowercase())
        .then_with(|| left.key.cmp(&right.key))
}

pub(in crate::desktop) fn sort_rows(rows: &mut [LoadedRow], spec: SortSpec) {
    rows.sort_unstable_by(|left, right| compare_rows(left, right, spec));
}

#[cfg(test)]
mod tests {
    use super::{SortDirection, SortField, SortSpec, sort_rows};
    use crate::desktop::state::LoadedRow;
    use std::path::PathBuf;
    use std::time::{Duration, UNIX_EPOCH};

    fn row(
        key: &str,
        name: &str,
        is_directory: bool,
        size: Option<u64>,
        modified: u64,
    ) -> LoadedRow {
        LoadedRow {
            key: key.to_owned(),
            path: PathBuf::from(key),
            name: name.to_owned(),
            kind: if is_directory {
                "Pasta".to_owned()
            } else {
                "Arquivo".to_owned()
            },
            icon: String::new(),
            details: String::new(),
            size,
            modified: Some(UNIX_EPOCH + Duration::from_secs(modified)),
            created: Some(UNIX_EPOCH + Duration::from_secs(modified)),
            accessed: Some(UNIX_EPOCH + Duration::from_secs(modified)),
            is_directory,
        }
    }

    #[test]
    fn toggle_muda_direcao_so_na_mesma_coluna() {
        let default = SortSpec::default();
        assert_eq!(default.field, SortField::Name);
        assert_eq!(default.direction, SortDirection::Ascending);
        assert_eq!(
            default.toggle_column(SortField::Name.column()).direction,
            SortDirection::Descending
        );
        assert_eq!(
            default.toggle_column(SortField::Size.column()).direction,
            SortDirection::Ascending
        );
    }

    #[test]
    fn ordena_diretorios_antes_e_aplica_tamanho_com_desempate() {
        let mut rows = vec![
            row("file-large", "a.txt", false, Some(20), 1),
            row("dir-z", "zeta", true, None, 1),
            row("file-small", "b.txt", false, Some(2), 1),
            row("dir-a", "alpha", true, None, 1),
        ];
        sort_rows(
            &mut rows,
            SortSpec {
                field: SortField::Size,
                direction: SortDirection::Ascending,
            },
        );
        assert_eq!(
            rows.iter().map(|row| row.name.as_str()).collect::<Vec<_>>(),
            ["alpha", "zeta", "b.txt", "a.txt"]
        );
    }

    #[test]
    fn ordena_data_descendente_com_desempate_por_nome() {
        let mut rows = vec![
            row("old", "zeta.txt", false, Some(1), 10),
            row("new", "alpha.txt", false, Some(1), 20),
            row("same-a", "alpha-2.txt", false, Some(1), 20),
        ];
        sort_rows(
            &mut rows,
            SortSpec {
                field: SortField::Modified,
                direction: SortDirection::Descending,
            },
        );
        assert_eq!(rows[0].name, "alpha-2.txt");
        assert_eq!(rows[1].name, "alpha.txt");
        assert_eq!(rows[2].name, "zeta.txt");
    }
}
