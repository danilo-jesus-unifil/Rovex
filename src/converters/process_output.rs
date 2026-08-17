use std::io;
use std::thread;

pub(crate) const MAX_PROCESS_OUTPUT_BYTES: usize = 64 * 1024;

pub(crate) fn read_limited_output<R: io::Read>(mut reader: R) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::with_capacity(MAX_PROCESS_OUTPUT_BYTES);
    let mut buffer = [0_u8; 8 * 1024];
    let mut exceeded = false;
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        if bytes.len() < MAX_PROCESS_OUTPUT_BYTES {
            let remaining = MAX_PROCESS_OUTPUT_BYTES - bytes.len();
            let retained = read.min(remaining);
            bytes.extend_from_slice(&buffer[..retained]);
            exceeded |= read > remaining;
        } else {
            exceeded = true;
        }
    }
    if exceeded {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "a saída do processo excedeu o limite de diagnóstico",
        ));
    }
    Ok(bytes)
}

fn spawn_output_reader<R: io::Read + Send + 'static>(
    reader: R,
    name: &'static str,
) -> io::Result<thread::JoinHandle<io::Result<Vec<u8>>>> {
    thread::Builder::new()
        .name(name.to_owned())
        .spawn(move || read_limited_output(reader))
}

pub(crate) fn join_output_reader(
    reader: Option<thread::JoinHandle<io::Result<Vec<u8>>>>,
) -> io::Result<Vec<u8>> {
    match reader {
        Some(reader) => reader.join().map_err(|_| {
            io::Error::other("o leitor de saída do processo terminou inesperadamente")
        })?,
        None => Ok(Vec::new()),
    }
}

pub(crate) fn start_output_reader<R: io::Read + Send + 'static>(
    reader: Option<R>,
    name: &'static str,
) -> io::Result<Option<thread::JoinHandle<io::Result<Vec<u8>>>>> {
    reader
        .map(|reader| spawn_output_reader(reader, name))
        .transpose()
}

pub(crate) fn stderr_message(bytes: &[u8]) -> String {
    let text = String::from_utf8_lossy(bytes);
    let text = text.trim();
    if text.is_empty() {
        "o processo terminou sem diagnóstico".to_owned()
    } else {
        text.lines()
            .rev()
            .take(3)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join(" ")
    }
}
