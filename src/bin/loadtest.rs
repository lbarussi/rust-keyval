use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

fn resp_bulk(s: &str) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(format!("${}\r\n", s.as_bytes().len()).as_bytes());
    out.extend_from_slice(s.as_bytes());
    out.extend_from_slice(b"\r\n");
    out
}

fn resp_array(cmd: &[&str]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(format!("*{}\r\n", cmd.len()).as_bytes());
    for part in cmd {
        out.extend_from_slice(&resp_bulk(part));
    }
    out
}

fn read_all(stream: &mut TcpStream, timeout_ms: u64) -> Vec<u8> {
    stream
        .set_read_timeout(Some(Duration::from_millis(timeout_ms)))
        .unwrap();

    let mut out = Vec::new();
    let mut buf = [0u8; 8192];
    let deadline = Instant::now() + Duration::from_millis(timeout_ms * 6);

    loop {
        match stream.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => out.extend_from_slice(&buf[..n]),
            Err(e) => {
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::TimedOut
                {
                    break;
                }
                panic!("read error: {e}");
            }
        }
        if Instant::now() > deadline {
            break;
        }
    }

    out
}

fn count_replies(mut b: &[u8]) -> usize {
    let mut n = 0;
    while !b.is_empty() {
        match b[0] {
            b'+' | b'-' | b':' => {
                if let Some(i) = memchr::memmem::find(b, b"\r\n") {
                    b = &b[i + 2..];
                    n += 1;
                } else {
                    break;
                }
            }
            b'$' => {
                if let Some(i) = memchr::memmem::find(b, b"\r\n") {
                    let header = &b[1..i];
                    let len = std::str::from_utf8(header).ok().and_then(|s| s.parse::<isize>().ok());
                    b = &b[i + 2..];
                    match len {
                        Some(-1) => {
                            n += 1;
                        }
                        Some(l) if l >= 0 => {
                            let l = l as usize;
                            if b.len() < l + 2 {
                                break;
                            }
                            b = &b[l + 2..];
                            n += 1;
                        }
                        _ => break,
                    }
                } else {
                    break;
                }
            }
            b'*' => {
                if let Some(i) = memchr::memmem::find(b, b"\r\n") {
                    let nitems = std::str::from_utf8(&b[1..i]).ok().and_then(|s| s.parse::<usize>().ok());
                    b = &b[i + 2..];
                    if let Some(nitems) = nitems {
                        for _ in 0..nitems {
                            if b.first() != Some(&b'$') { return n; }
                            if let Some(j) = memchr::memmem::find(b, b"\r\n") {
                                let len = std::str::from_utf8(&b[1..j]).ok().and_then(|s| s.parse::<isize>().ok());
                                b = &b[j + 2..];
                                match len {
                                    Some(-1) => {}
                                    Some(l) if l >= 0 => {
                                        let l = l as usize;
                                        if b.len() < l + 2 { return n; }
                                        b = &b[l + 2..];
                                    }
                                    _ => return n,
                                }
                            } else { return n; }
                        }
                        n += 1;
                    } else { break; }
                } else { break; }
            }
            _ => break,
        }
    }
    n
}

fn main() {
    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1:6374".into());
    let clients: usize = std::env::var("CLIENTS").ok().and_then(|v| v.parse().ok()).unwrap_or(200);
    let ops_per_client: usize = std::env::var("OPS").ok().and_then(|v| v.parse().ok()).unwrap_or(2000);
    let pipeline_size: usize = std::env::var("PIPE").ok().and_then(|v| v.parse().ok()).unwrap_or(20);
    let read_timeout_ms: u64 = std::env::var("RT").ok().and_then(|v| v.parse().ok()).unwrap_or(300);

    {
        let mut s = TcpStream::connect(&host).unwrap();
        s.set_nodelay(true).unwrap();
        let payload = resp_array(&["FLUSHALL"]);
        s.write_all(&payload).unwrap();
        s.flush().unwrap();
        let _ = read_all(&mut s, 400);
    }

    let ok = Arc::new(AtomicU64::new(0));
    let bad = Arc::new(AtomicU64::new(0));

    let start = Instant::now();
    let mut handles = Vec::new();

    for cid in 0..clients {
        let host = host.clone();
        let ok = ok.clone();
        let bad = bad.clone();

        handles.push(std::thread::spawn(move || {
            let mut s = TcpStream::connect(&host).unwrap();
            s.set_nodelay(true).unwrap();

            let global_ctr_key = "ctr";
            let my_ctr_key = format!("ctr:{cid}");
            let base_key = format!("k:{cid}:");

            let mut sent_ops = 0usize;

            while sent_ops < ops_per_client {
                let mut payload = Vec::new();
                let mut expected_replies = 0usize;

                for i in 0..pipeline_size {
                    if sent_ops >= ops_per_client { break; }
                    let k = format!("{base_key}{i}:{sent_ops}");
                    let v = format!("v{cid}:{sent_ops}");

                    match (sent_ops + i) % 5 {
                        0 => {
                            payload.extend_from_slice(&resp_array(&["SET", &k, &v]));
                            expected_replies += 1;
                        }
                        1 => {
                            payload.extend_from_slice(&resp_array(&["GET", &k]));
                            expected_replies += 1;
                        }
                        2 => {
                            payload.extend_from_slice(&resp_array(&["INCR", global_ctr_key]));
                            payload.extend_from_slice(&resp_array(&["INCR", &my_ctr_key]));
                            expected_replies += 2;
                        }
                        3 => {
                            payload.extend_from_slice(&resp_array(&["EXISTS", &k]));
                            expected_replies += 1;
                        }
                        _ => {
                            payload.extend_from_slice(&resp_array(&["DEL", &k]));
                            expected_replies += 1;
                        }
                    }

                    sent_ops += 1;
                }

                if let Err(_) = s.write_all(&payload) {
                    bad.fetch_add(expected_replies as u64, Ordering::Relaxed);
                    return;
                }
                let _ = s.flush();

                let buf = read_all(&mut s, read_timeout_ms);
                let got = count_replies(&buf);

                if got >= expected_replies {
                    ok.fetch_add(expected_replies as u64, Ordering::Relaxed);
                } else {
                    bad.fetch_add((expected_replies - got) as u64, Ordering::Relaxed);
                    ok.fetch_add(got as u64, Ordering::Relaxed);
                }
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    let elapsed = start.elapsed().as_secs_f64();
    let okv = ok.load(Ordering::Relaxed);
    let badv = bad.load(Ordering::Relaxed);
    let total = okv + badv;

    println!("=== LOADTEST RESULTS ===");
    println!("host={host}");
    println!("clients={clients} ops/client={ops_per_client} pipeline={pipeline_size}");
    println!("total_replies={total} ok={okv} bad={badv}");
    println!("elapsed={:.3}s", elapsed);
    println!("throughput={:.0} replies/sec", total as f64 / elapsed);

    if badv > 0 {
        eprintln!("❌ FAIL: had {} missing/invalid replies", badv);
        std::process::exit(1);
    }

    println!("✅ PASS");
}
