use crate::converters::ConversionKind;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::desktop) enum OperationKind {
    Copy,
    Move,
    Rename,
    Delete,
    CreateDirectory,
}

#[derive(Debug, Clone)]
pub(in crate::desktop) struct ConversionRequest {
    pub(in crate::desktop) kind: ConversionKind,
    pub(in crate::desktop) sources: Vec<PathBuf>,
    pub(in crate::desktop) refresh_path: PathBuf,
}

#[derive(Debug)]
pub(in crate::desktop) struct ConversionOutcome {
    pub(in crate::desktop) kind: ConversionKind,
    pub(in crate::desktop) completed: usize,
    pub(in crate::desktop) failed: Vec<String>,
    pub(in crate::desktop) cancelled: bool,
}

impl ConversionOutcome {
    pub(in crate::desktop) fn message(&self) -> String {
        let mut message = if self.cancelled {
            format!(
                "Conversão cancelada: {} item(ns) concluído(s), {} falha(s).",
                self.completed,
                self.failed.len()
            )
        } else if self.failed.is_empty() {
            format!(
                "Conversão para {} concluída: {} item(ns).",
                self.kind.label(),
                self.completed
            )
        } else {
            format!(
                "Conversão para {} concluída parcialmente: {} item(ns) concluído(s), {} falha(s).",
                self.kind.label(),
                self.completed,
                self.failed.len()
            )
        };
        for failure in self.failed.iter().take(3) {
            message.push('\n');
            message.push_str(failure);
        }
        if self.failed.len() > 3 {
            message.push_str(&format!("\n… e mais {} falha(s).", self.failed.len() - 3));
        }
        message
    }

    pub(in crate::desktop) fn status(&self) -> String {
        if self.cancelled {
            "Conversão cancelada; a pasta será atualizada.".to_owned()
        } else if self.failed.is_empty() {
            "Conversão concluída; a pasta será atualizada.".to_owned()
        } else {
            "Conversão concluída parcialmente; verifique o resultado.".to_owned()
        }
    }
}

#[derive(Debug, Clone)]
pub(in crate::desktop) struct OperationRequest {
    pub(in crate::desktop) kind: OperationKind,
    pub(in crate::desktop) sources: Vec<PathBuf>,
    pub(in crate::desktop) destination_directory: Option<PathBuf>,
    pub(in crate::desktop) rename_name: Option<String>,
    pub(in crate::desktop) refresh_path: PathBuf,
}

#[derive(Debug)]
pub(in crate::desktop) struct OperationOutcome {
    pub(in crate::desktop) kind: OperationKind,
    pub(in crate::desktop) completed: usize,
    pub(in crate::desktop) failed: Vec<String>,
    pub(in crate::desktop) cancelled: bool,
}

impl OperationOutcome {
    pub(in crate::desktop) fn message(&self) -> String {
        let action = match self.kind {
            OperationKind::Copy => "cópia",
            OperationKind::Move => "movimentação",
            OperationKind::Rename => "renomeação",
            OperationKind::Delete => {
                #[cfg(windows)]
                {
                    "envio para a Lixeira"
                }
                #[cfg(not(windows))]
                {
                    "exclusão"
                }
            }
            OperationKind::CreateDirectory => "criação de pasta",
        };
        let mut message = if self.cancelled {
            format!(
                "Operação cancelada: {} item(ns) concluído(s), {} falha(s).",
                self.completed,
                self.failed.len()
            )
        } else if self.failed.is_empty() {
            format!("{} concluída: {} item(ns).", action, self.completed)
        } else {
            format!(
                "{} concluída parcialmente: {} item(ns) concluído(s), {} falha(s).",
                action,
                self.completed,
                self.failed.len()
            )
        };
        for failure in self.failed.iter().take(3) {
            message.push('\n');
            message.push_str(failure);
        }
        if self.failed.len() > 3 {
            message.push_str(&format!("\n… e mais {} falha(s).", self.failed.len() - 3));
        }
        message
    }

    pub(in crate::desktop) fn status(&self) -> String {
        if self.cancelled {
            "Operação cancelada; a pasta será atualizada.".to_owned()
        } else if self.failed.is_empty() {
            "Operação concluída; a pasta será atualizada.".to_owned()
        } else {
            "Operação concluída parcialmente; verifique o resultado.".to_owned()
        }
    }
}

#[derive(Debug)]
pub(in crate::desktop) struct OperationUpdate {
    pub(in crate::desktop) completed_items: usize,
    pub(in crate::desktop) total_items: usize,
    pub(in crate::desktop) current_bytes: u64,
    pub(in crate::desktop) current_total_bytes: u64,
    pub(in crate::desktop) explicit_percent: Option<u8>,
    pub(in crate::desktop) label: String,
}
