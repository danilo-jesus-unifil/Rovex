use copypasta::{ClipboardContext, ClipboardProvider};
use std::fmt;
use std::path::PathBuf;
use std::sync::Mutex;

const PAYLOAD_HEADER: &str = "ROVEX_CLIPBOARD_V1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipboardAction {
    Copy,
    Cut,
}

impl ClipboardAction {
    fn as_str(self) -> &'static str {
        match self {
            Self::Copy => "copy",
            Self::Cut => "cut",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        match value {
            "copy" => Some(Self::Copy),
            "cut" => Some(Self::Cut),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClipboardPayload {
    pub action: ClipboardAction,
    pub paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClipboardError {
    EmptySelection,
    UnsupportedContents,
    InvalidPayload,
    Backend(String),
}

impl fmt::Display for ClipboardError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptySelection => write!(formatter, "nenhum item selecionado para copiar"),
            Self::UnsupportedContents => {
                write!(formatter, "o clipboard não contém uma seleção do Rovex")
            }
            Self::InvalidPayload => write!(formatter, "o conteúdo do clipboard está inválido"),
            Self::Backend(message) => {
                write!(formatter, "não foi possível acessar o clipboard: {message}")
            }
        }
    }
}

impl std::error::Error for ClipboardError {}

fn encode_payload(payload: &ClipboardPayload) -> Result<String, ClipboardError> {
    if payload.paths.is_empty() {
        return Err(ClipboardError::EmptySelection);
    }
    let mut encoded = format!("{}\n{}\n", PAYLOAD_HEADER, payload.action.as_str());
    for path in &payload.paths {
        if path.as_os_str().is_empty() || path.to_string_lossy().contains('\n') {
            return Err(ClipboardError::InvalidPayload);
        }
        encoded.push_str(&path.to_string_lossy());
        encoded.push('\n');
    }
    Ok(encoded)
}

fn decode_payload(contents: &str) -> Result<ClipboardPayload, ClipboardError> {
    let mut lines = contents.lines();
    if lines.next() != Some(PAYLOAD_HEADER) {
        return Err(ClipboardError::UnsupportedContents);
    }
    let action = lines
        .next()
        .and_then(ClipboardAction::parse)
        .ok_or(ClipboardError::InvalidPayload)?;
    let paths: Vec<PathBuf> = lines
        .map(PathBuf::from)
        .filter(|path| !path.as_os_str().is_empty())
        .collect();
    if paths.is_empty() {
        return Err(ClipboardError::InvalidPayload);
    }
    Ok(ClipboardPayload { action, paths })
}

pub struct ClipboardStore {
    context: Mutex<ClipboardContext>,
}

impl ClipboardStore {
    pub fn new() -> Result<Self, ClipboardError> {
        let context =
            ClipboardContext::new().map_err(|error| ClipboardError::Backend(error.to_string()))?;
        Ok(Self {
            context: Mutex::new(context),
        })
    }

    pub fn set_paths(
        &self,
        paths: Vec<PathBuf>,
        action: ClipboardAction,
    ) -> Result<(), ClipboardError> {
        let encoded = encode_payload(&ClipboardPayload { action, paths })?;
        let mut context = self
            .context
            .lock()
            .map_err(|_| ClipboardError::Backend("estado do clipboard envenenado".into()))?;
        context
            .set_contents(encoded)
            .map_err(|error| ClipboardError::Backend(error.to_string()))
    }

    pub fn get_payload(&self) -> Result<ClipboardPayload, ClipboardError> {
        let mut context = self
            .context
            .lock()
            .map_err(|_| ClipboardError::Backend("estado do clipboard envenenado".into()))?;
        let contents = context
            .get_contents()
            .map_err(|error| ClipboardError::Backend(error.to_string()))?;
        decode_payload(&contents)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ClipboardAction, ClipboardError, ClipboardPayload, decode_payload, encode_payload,
    };
    use std::path::PathBuf;

    #[test]
    fn serializa_e_restaura_payload_de_copy() {
        let payload = ClipboardPayload {
            action: ClipboardAction::Copy,
            paths: vec![
                PathBuf::from("/tmp/relatório.txt"),
                PathBuf::from("/tmp/pasta com espaços"),
            ],
        };
        let encoded = encode_payload(&payload).expect("payload deve ser serializado");
        assert_eq!(decode_payload(&encoded), Ok(payload));
    }

    #[test]
    fn rejeita_clipboard_externo_ou_payload_vazio() {
        assert_eq!(
            decode_payload("texto comum"),
            Err(ClipboardError::UnsupportedContents)
        );
        assert_eq!(
            encode_payload(&ClipboardPayload {
                action: ClipboardAction::Cut,
                paths: Vec::new(),
            }),
            Err(ClipboardError::EmptySelection)
        );
    }
}
