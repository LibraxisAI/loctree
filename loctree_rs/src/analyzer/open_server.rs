use std::io::{self, BufRead, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;
use std::thread;

static OPEN_SERVER_BASE: OnceLock<String> = OnceLock::new();

pub(crate) fn url_encode_component(input: &str) -> String {
    input
        .bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{:02X}", b),
        })
        .collect()
}

pub(crate) fn url_decode_component(input: &str) -> Option<String> {
    let mut out = String::new();
    let mut iter = input.as_bytes().iter().cloned();
    while let Some(b) = iter.next() {
        if b == b'%' {
            let hi = iter.next()?;
            let lo = iter.next()?;
            let hex = [hi, lo];
            let s = std::str::from_utf8(&hex).ok()?;
            let v = u8::from_str_radix(s, 16).ok()?;
            out.push(v as char);
        } else {
            out.push(b as char);
        }
    }
    Some(out)
}

pub(crate) fn open_in_browser(path: &Path) {
    let Ok(canon) = path.canonicalize() else {
        eprintln!(
            "[loctree][warn] Could not resolve report path for auto-open: {}",
            path.display()
        );
        return;
    };

    let target = canon.to_string_lossy().to_string();
    if target.bytes().any(|b| b < 0x20) {
        eprintln!(
            "[loctree][warn] Skipping auto-open for suspicious path: {}",
            target
        );
        return;
    }

    #[cfg(target_os = "macos")]
    let try_cmds = vec![("open", vec![target.as_str()])];
    #[cfg(target_os = "windows")]
    let try_cmds = vec![(
        "powershell",
        vec![
            "-NoProfile",
            "-Command",
            "Start-Process",
            "-FilePath",
            target.as_str(),
        ],
    )];
    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    let try_cmds = vec![("xdg-open", vec![target.as_str()])];

    for (program, args) in try_cmds {
        if Command::new(program).args(args.clone()).spawn().is_ok() {
            return;
        }
    }
    eprintln!(
        "[loctree][warn] Could not open report automatically: {}",
        target
    );
}

pub(crate) fn start_open_server(
    roots: Vec<PathBuf>,
    editor_cmd: Option<String>,
) -> Option<(String, thread::JoinHandle<()>)> {
    let listener = TcpListener::bind("127.0.0.1:0").ok()?;
    let port = listener.local_addr().ok()?.port();
    let base = format!("http://127.0.0.1:{port}");
    let _ = OPEN_SERVER_BASE.set(base.clone());

    let handle = thread::spawn(move || {
        for mut stream in listener.incoming().flatten() {
            let mut buf = String::new();
            let mut reader = io::BufReader::new(&stream);
            if reader.read_line(&mut buf).is_ok() {
                handle_open_request(&mut stream, &roots, editor_cmd.as_ref(), buf.trim());
            }
        }
    });
    Some((base, handle))
}

pub(crate) fn current_open_base() -> Option<String> {
    OPEN_SERVER_BASE.get().cloned()
}

fn open_file_in_editor(
    full_path: &Path,
    line: usize,
    editor_cmd: Option<&String>,
) -> io::Result<()> {
    if let Some(tpl) = editor_cmd {
        let replaced = tpl
            .replace("{file}", full_path.to_string_lossy().as_ref())
            .replace("{line}", &line.to_string());
        let parts: Vec<String> = replaced.split_whitespace().map(|s| s.to_string()).collect();
        if let Some((prog, args)) = parts.split_first() {
            let status = Command::new(prog).args(args).status()?;
            if status.success() {
                return Ok(());
            }
        }
    } else if Command::new("code")
        .arg("-g")
        .arg(format!("{}:{}", full_path.to_string_lossy(), line.max(1)))
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
    {
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    let fallback = Command::new("open")
        .arg(full_path)
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    #[cfg(target_os = "windows")]
    let fallback = Command::new("cmd")
        .args(["/C", "start", full_path.to_string_lossy().as_ref()])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    let fallback = Command::new("xdg-open")
        .arg(full_path)
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if fallback {
        Ok(())
    } else {
        Err(io::Error::other("could not open file via editor"))
    }
}

fn handle_open_request(
    stream: &mut TcpStream,
    roots: &[PathBuf],
    editor_cmd: Option<&String>,
    request_line: &str,
) {
    let mut parts = request_line.split_whitespace();
    let _method = parts.next();
    let target = parts.next().unwrap_or("/");
    if !target.starts_with("/open?") {
        let _ = stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n");
        return;
    }
    let query = &target[6..];
    let mut file = None;
    let mut line = 1usize;
    for pair in query.split('&') {
        if let Some((k, v)) = pair.split_once('=') {
            match k {
                "f" => file = url_decode_component(v),
                "l" => {
                    line = v.parse::<usize>().unwrap_or(1).max(1);
                }
                _ => {}
            }
        }
    }
    let Some(rel_or_abs) = file else {
        let _ = stream.write_all(b"HTTP/1.1 400 Bad Request\r\n\r\n");
        return;
    };

    let mut candidate = None;
    let path_obj = PathBuf::from(&rel_or_abs);
    if path_obj.is_absolute() {
        if let Ok(canon) = path_obj.canonicalize() {
            if roots.iter().any(|r| canon.starts_with(r)) {
                candidate = Some(canon);
            }
        }
    } else {
        for root in roots {
            let joined = root.join(&path_obj);
            if let Ok(canon) = joined.canonicalize() {
                if canon.starts_with(root) {
                    candidate = Some(canon);
                    break;
                }
            }
        }
    }

    let Some(full) = candidate else {
        let _ = stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n");
        return;
    };

    let status = open_file_in_editor(&full, line, editor_cmd);
    let response = if status.is_ok() {
        "HTTP/1.1 200 OK\r\n\r\nopened"
    } else {
        "HTTP/1.1 500 Internal Server Error\r\n\r\nopen failed"
    };
    let _ = stream.write_all(response.as_bytes());
}
