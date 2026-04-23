use crate::error::{Error, Result};
use is_terminal::IsTerminal;
use serde_json::Value;
use std::io::{self, ErrorKind, Write};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    /// Pretty-printed when stdout is a TTY, compact when piped.
    Auto,
    /// Always pretty.
    Pretty,
    /// Always compact (single-line).
    Compact,
}

impl Format {
    pub fn resolve(self) -> ResolvedFormat {
        match self {
            Format::Pretty => ResolvedFormat::Pretty,
            Format::Compact => ResolvedFormat::Compact,
            Format::Auto => {
                if io::stdout().is_terminal() {
                    ResolvedFormat::Pretty
                } else {
                    ResolvedFormat::Compact
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResolvedFormat {
    Pretty,
    Compact,
}

/// Emit a single JSON value to stdout, trailing newline.
pub fn emit_value<W: Write>(mut w: W, value: &Value, fmt: ResolvedFormat) -> io::Result<()> {
    match fmt {
        ResolvedFormat::Pretty => serde_json::to_writer_pretty(&mut w, value)?,
        ResolvedFormat::Compact => serde_json::to_writer(&mut w, value)?,
    }
    w.write_all(b"\n")
}

/// Emit a stream of JSON values as JSONL (one compact record per line, regardless of TTY).
pub fn emit_jsonl<W: Write, I: IntoIterator<Item = Value>>(mut w: W, iter: I) -> io::Result<()> {
    for v in iter {
        serde_json::to_writer(&mut w, &v)?;
        w.write_all(b"\n")?;
    }
    Ok(())
}

/// Emit an error to stderr as the documented JSON envelope.
pub fn emit_error<W: Write>(mut w: W, err: &Error) -> io::Result<()> {
    serde_json::to_writer(&mut w, &err.to_stderr_json())?;
    w.write_all(b"\n")
}

/// Map an I/O error from writing to stdout into the right `Error` variant.
/// `BrokenPipe` becomes `Error::BrokenPipe` (silent exit 0); everything else
/// is a transport failure (exit 3), not a usage error.
pub fn map_stdout_err(e: io::Error) -> Error {
    if e.kind() == ErrorKind::BrokenPipe {
        Error::BrokenPipe
    } else {
        Error::Transport(format!("stdout: {e}"))
    }
}

/// Write a value to stdout, handling broken pipes cleanly.
pub fn write_value(value: &Value, fmt: ResolvedFormat) -> Result<()> {
    emit_value(io::stdout().lock(), value, fmt).map_err(map_stdout_err)
}

/// Write a single JSON record + newline to a writer, mapping errors consistently.
/// Broken pipes become `Error::BrokenPipe`; other I/O becomes `Error::Transport`.
pub fn write_jsonl_line<W: Write>(mut w: W, v: &Value) -> Result<()> {
    serde_json::to_writer(&mut w, v).map_err(|e| {
        if e.is_io() {
            Error::BrokenPipe
        } else {
            Error::Transport(format!("serialize: {e}"))
        }
    })?;
    w.write_all(b"\n").map_err(map_stdout_err)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn compact_emits_single_line() {
        let mut buf = Vec::new();
        emit_value(&mut buf, &json!({"a": 1}), ResolvedFormat::Compact).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "{\"a\":1}\n");
    }

    #[test]
    fn pretty_emits_indented() {
        let mut buf = Vec::new();
        emit_value(&mut buf, &json!({"a": 1}), ResolvedFormat::Pretty).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("  \"a\": 1"));
    }

    #[test]
    fn jsonl_one_record_per_line() {
        let mut buf = Vec::new();
        emit_jsonl(&mut buf, vec![json!({"a": 1}), json!({"a": 2})]).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "{\"a\":1}\n{\"a\":2}\n");
    }

    #[test]
    fn error_envelope_goes_to_writer() {
        let e = Error::Usage("bad".into());
        let mut buf = Vec::new();
        emit_error(&mut buf, &e).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("\"message\":\"bad\""));
    }
}
