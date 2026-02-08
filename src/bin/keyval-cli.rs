use std::borrow::Cow;
use std::env;
use std::io::{self, Read, Stdin, Stdout, Write};
use std::net::TcpStream;
use std::time::{Duration, Instant};

fn encode_cmd(parts: &[String]) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    out.extend_from_slice(format!("*{}\r\n", parts.len()).as_bytes());
    for p in parts {
        let b = p.as_bytes();
        out.extend_from_slice(format!("${}\r\n", b.len()).as_bytes());
        out.extend_from_slice(b);
        out.extend_from_slice(b"\r\n");
    }
    out
}

#[derive(Debug)]
enum Resp {
    Simple(String),
    Error(String),
    Integer(i64),
    Bulk(Option<Vec<u8>>),
    Array(Vec<Resp>),
}

fn read_line(buf: &[u8], i: &mut usize) -> Option<Vec<u8>> {
    let start: usize = *i;
    while *i + 1 < buf.len() {
        if buf[*i] == b'\r' && buf[*i + 1] == b'\n' {
            let line: Vec<u8> = buf[start..*i].to_vec();
            *i += 2;
            return Some(line);
        }
        *i += 1;
    }
    *i = start;
    None
}

fn parse_resp_one(buf: &[u8]) -> Result<Option<(Resp, usize)>, String> {
    if buf.is_empty() {
        return Ok(None);
    }
    let mut i: usize = 0;
    let prefix: char = buf[0] as char;
    i += 1;

    match prefix {
        '+' => {
            let line: Vec<u8> = read_line(buf, &mut i).ok_or_else(|| "incomplete +".to_string())?;
            let s: String = String::from_utf8_lossy(&line).to_string();
            Ok(Some((Resp::Simple(s), i)))
        }
        '-' => {
            let line: Vec<u8> = read_line(buf, &mut i).ok_or_else(|| "incomplete -".to_string())?;
            let s: String = String::from_utf8_lossy(&line).to_string();
            Ok(Some((Resp::Error(s), i)))
        }
        ':' => {
            let line: Vec<u8> = read_line(buf, &mut i).ok_or_else(|| "incomplete :".to_string())?;
            let s = String::from_utf8_lossy(&line);
            let n: i64 = s.parse().map_err(|_| "invalid integer".to_string())?;
            Ok(Some((Resp::Integer(n), i)))
        }
        '$' => {
            let line: Vec<u8> = read_line(buf, &mut i).ok_or_else(|| "incomplete $".to_string())?;
            let s: Cow<str> = String::from_utf8_lossy(&line);
            let len: i64 = s.parse().map_err(|_| "invalid bulk len".to_string())?;
            if len == -1 {
                return Ok(Some((Resp::Bulk(None), i)));
            }
            if len < -1 {
                return Err("bulk len < -1".into());
            }
            let len: usize = len as usize;
            if i + len + 2 > buf.len() {
                return Ok(None);
            }
            let data: Vec<u8> = buf[i..i + len].to_vec();
            i += len;
            if buf.get(i) != Some(&b'\r') || buf.get(i + 1) != Some(&b'\n') {
                return Err("bulk missing CRLF".into());
            }
            i += 2;
            Ok(Some((Resp::Bulk(Some(data)), i)))
        }
        '*' => {
            let line: Vec<u8> = read_line(buf, &mut i).ok_or_else(|| "incomplete *".to_string())?;
            let s: Cow<str> = String::from_utf8_lossy(&line);
            let n: i64 = s.parse().map_err(|_| "invalid array len".to_string())?;
            if n < 0 {
                return Err("array len < 0".into());
            }
            let n: usize = n as usize;

            let mut items: Vec<Resp> = Vec::with_capacity(n);
            let mut consumed: usize = i;

            for _ in 0..n {
                match parse_resp_one(&buf[consumed..])? {
                    Some((v, used)) => {
                        items.push(v);
                        consumed += used;
                    }
                    None => return Ok(None),
                }
            }
            Ok(Some((Resp::Array(items), consumed)))
        }
        _ => Err(format!("unknown RESP prefix: {}", prefix)),
    }
}

fn read_resp_message(stream: &mut TcpStream) -> Result<Resp, String> {
    stream.set_read_timeout(Some(Duration::from_millis(500))).unwrap();
    let mut acc: Vec<u8> = Vec::<u8>::new();
    let mut buf = [0u8; 4096];
    let deadline: Instant = Instant::now() + Duration::from_millis(3000);

    loop {
        if let Some((msg, used)) = parse_resp_one(&acc).map_err(|e| e.to_string())? {
            acc.drain(..used);
            return Ok(msg);
        }

        match stream.read(&mut buf) {
            Ok(0) => return Err("connection closed".into()),
            Ok(n) => acc.extend_from_slice(&buf[..n]),
            Err(e) => {
                if e.kind() == io::ErrorKind::WouldBlock || e.kind() == io::ErrorKind::TimedOut {
                    if Instant::now() > deadline {
                        return Err("timeout waiting response".into());
                    }
                    continue;
                }
                return Err(format!("read error: {e}"));
            }
        }
    }
}

fn pretty_print(resp: &Resp) {
    match resp {
        Resp::Simple(s) => println!("{}", s),
        Resp::Error(s) => eprintln!("(error) {}", s),
        Resp::Integer(n) => println!("{}", n),
        Resp::Bulk(None) => println!("(nil)"),
        Resp::Bulk(Some(b)) => println!("{}", String::from_utf8_lossy(b)),
        Resp::Array(items) => {
            for (idx, it) in items.iter().enumerate() {
                print!("{}) ", idx + 1);
                match it {
                    Resp::Bulk(Some(b)) => println!("{}", String::from_utf8_lossy(b)),
                    Resp::Bulk(None) => println!("(nil)"),
                    Resp::Simple(s) => println!("{}", s),
                    Resp::Integer(n) => println!("{}", n),
                    Resp::Error(s) => println!("(error) {}", s),
                    Resp::Array(_) => {
                        println!("{:?}", it);
                    }
                }
            }
            if items.is_empty() {
                println!("(empty array)");
            }
        }
    }
}

fn split_args(line: &str) -> Result<Vec<String>, String> {
    let mut out: Vec<String> = Vec::new();
    let mut cur: String = String::new();
    let mut in_quotes: bool = false;
    let mut escape: bool = false;

    for c in line.chars() {
        if escape {
            cur.push(c);
            escape = false;
            continue;
        }
        if c == '\\' {
            escape = true;
            continue;
        }
        if c == '"' {
            in_quotes = !in_quotes;
            continue;
        }
        if !in_quotes && c.is_whitespace() {
            if !cur.is_empty() {
                out.push(cur.clone());
                cur.clear();
            }
            continue;
        }
        cur.push(c);
    }

    if escape {
        return Err("dangling escape (\\) at end".into());
    }
    if in_quotes {
        return Err("unclosed quote (\")".into());
    }
    if !cur.is_empty() {
        out.push(cur);
    }

    Ok(out)
}

fn main() -> Result<(), String> {
    let mut host: String = "127.0.0.1".to_string();
    let mut port: String = "6374".to_string();

    let args: Vec<String> = env::args().collect();
    let mut i: usize = 1;
    let mut cmd_parts: Vec<String> = Vec::new();

    while i < args.len() {
        match args[i].as_str() {
            "--host" => {
                i += 1;
                host = args.get(i).ok_or("missing --host value")?.clone();
            }
            "--port" => {
                i += 1;
                port = args.get(i).ok_or("missing --port value")?.clone();
            }
            _ => {
                cmd_parts = args[i..].to_vec();
                break;
            }
        }
        i += 1;
    }

    let addr: String = format!("{}:{}", host, port);
    let mut stream: TcpStream = TcpStream::connect(&addr).map_err(|e| format!("connect {}: {}", addr, e))?;
    stream.set_nodelay(true).unwrap();

    if !cmd_parts.is_empty() {
        let payload: Vec<u8> = encode_cmd(&cmd_parts);
        stream.write_all(&payload).map_err(|e| e.to_string())?;
        stream.flush().map_err(|e| e.to_string())?;

        let resp: Resp = read_resp_message(&mut stream)?;
        pretty_print(&resp);
        return Ok(());
    }

    let stdin: Stdin = io::stdin();
    let mut stdout: Stdout = io::stdout();

    loop {
        print!("keyval> ");
        stdout.flush().unwrap();

        let mut line: String = String::new();
        if stdin.read_line(&mut line).unwrap() == 0 {
            break;
        }
        let line: &str = line.trim();
        if line.is_empty() {
            continue;
        }
        if line.eq_ignore_ascii_case("quit") || line.eq_ignore_ascii_case("exit") {
            break;
        }

        let parts: Vec<String> = split_args(line)?;
        if parts.is_empty() {
            continue;
        }

        let payload: Vec<u8> = encode_cmd(&parts);
        stream.write_all(&payload).map_err(|e| e.to_string())?;
        stream.flush().map_err(|e| e.to_string())?;

        let resp: Resp = read_resp_message(&mut stream)?;
        pretty_print(&resp);
    }

    Ok(())
}
