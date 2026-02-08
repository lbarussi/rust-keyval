use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::{Duration, Instant};

fn send_and_read_all(stream: &mut TcpStream, payload: &[u8]) -> String {
    stream.write_all(payload).unwrap();
    stream.flush().unwrap();

    stream.set_read_timeout(Some(Duration::from_millis(200))).unwrap();

    let mut out = Vec::new();
    let mut buf = [0u8; 4096];
    let deadline = Instant::now() + Duration::from_millis(1200);

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

    String::from_utf8_lossy(&out).to_string()
}

fn assert_contains(label: &str, got: &str, needle: &str) {
    if !got.contains(needle) {
        eprintln!("FAIL [{label}]: expected to contain {needle:?}");
        eprintln!("GOT: {got:?}");
        std::process::exit(1);
    }
}

fn assert_not_contains(label: &str, got: &str, needle: &str) {
    if got.contains(needle) {
        eprintln!("FAIL [{label}]: expected NOT to contain {needle:?}");
        eprintln!("GOT: {got:?}");
        std::process::exit(1);
    }
}

fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:6374").unwrap();
    stream.set_nodelay(true).unwrap();


    let resp = send_and_read_all(&mut stream, b"*1\r\n$8\r\nFLUSHALL\r\n");
    assert_contains("FLUSHALL", &resp, "+OK\r\n");


    let pipeline = b"*3\r\n$3\r\nSET\r\n$1\r\na\r\n$1\r\n1\r\n*2\r\n$3\r\nGET\r\n$1\r\na\r\n";
    let resp = send_and_read_all(&mut stream, pipeline);
    println!("PIPELINE RESP: {:?}", resp);
    assert_contains("PIPELINE", &resp, "+OK\r\n");
    assert_contains("PIPELINE", &resp, "$1\r\n1\r\n");
    assert_not_contains("PIPELINE", &resp, "-ERR");

    
    let part1 = b"*2\r\n$3\r\nGET\r\n$3\r\nfo";
    let part2 = b"o\r\n";

    stream.write_all(part1).unwrap();
    stream.flush().unwrap();

    let resp2 = send_and_read_all(&mut stream, part2);
    println!("FRAG RESP: {:?}", resp2);
    assert_contains("FRAG", &resp2, "$-1\r\n");
    assert_not_contains("FRAG", &resp2, "-ERR");


    let r = send_and_read_all(&mut stream, b"*2\r\n$4\r\nINCR\r\n$3\r\nctr\r\n");
    println!("INCR1: {:?}", r);
    assert_contains("INCR1", &r, ":1\r\n");

    let r = send_and_read_all(&mut stream, b"*2\r\n$4\r\nINCR\r\n$3\r\nctr\r\n");
    println!("INCR2: {:?}", r);
    assert_contains("INCR2", &r, ":2\r\n");


    let r = send_and_read_all(&mut stream, b"*3\r\n$3\r\nSET\r\n$1\r\nk\r\n$1\r\nv\r\n");
    assert_contains("SET k v", &r, "+OK\r\n");

    let r = send_and_read_all(&mut stream, b"*2\r\n$6\r\nEXISTS\r\n$1\r\nk\r\n");
    println!("EXISTS k: {:?}", r);
    assert_contains("EXISTS k", &r, ":1\r\n");

    let r = send_and_read_all(&mut stream, b"*2\r\n$3\r\nDEL\r\n$1\r\nk\r\n");
    println!("DEL k: {:?}", r);
    assert_contains("DEL k", &r, ":1\r\n");

    let r = send_and_read_all(&mut stream, b"*2\r\n$6\r\nEXISTS\r\n$1\r\nk\r\n");
    println!("EXISTS k (after del): {:?}", r);
    assert_contains("EXISTS k after del", &r, ":0\r\n");


    let r = send_and_read_all(&mut stream, b"*3\r\n$3\r\nSET\r\n$1\r\nt\r\n$3\r\n123\r\n");
    assert_contains("SET t 123", &r, "+OK\r\n");

    let r = send_and_read_all(&mut stream, b"*3\r\n$6\r\nEXPIRE\r\n$1\r\nt\r\n$1\r\n1\r\n");
    println!("EXPIRE t 1: {:?}", r);
    assert_contains("EXPIRE t 1", &r, ":1\r\n");

    std::thread::sleep(Duration::from_secs(2));

    let r = send_and_read_all(&mut stream, b"*2\r\n$3\r\nGET\r\n$1\r\nt\r\n");
    println!("GET t after expire: {:?}", r);
    assert_contains("GET t expired", &r, "$-1\r\n");


    let r = send_and_read_all(&mut stream, b"*3\r\n$3\r\nSET\r\n$1\r\na\r\n$1\r\n1\r\n");
    assert_contains("SET a 1", &r, "+OK\r\n");
    let r = send_and_read_all(&mut stream, b"*3\r\n$3\r\nSET\r\n$1\r\nb\r\n$1\r\n2\r\n");
    assert_contains("SET b 2", &r, "+OK\r\n");

    let r = send_and_read_all(&mut stream, b"*1\r\n$4\r\nKEYS\r\n");
    println!("KEYS: {:?}", r);

    assert_contains("KEYS", &r, "$1\r\na\r\n");
    assert_contains("KEYS", &r, "$1\r\nb\r\n");
    assert_not_contains("KEYS", &r, "-ERR");

    let r = send_and_read_all(&mut stream, b"*1\r\n$8\r\nFLUSHALL\r\n");
    println!("FLUSHALL: {:?}", r);
    assert_contains("FLUSHALL", &r, "+OK\r\n");

    let r = send_and_read_all(&mut stream, b"*1\r\n$4\r\nKEYS\r\n");
    println!("KEYS after flush: {:?}", r);


    assert_not_contains("KEYS after flush", &r, "$1\r\na\r\n");
    assert_not_contains("KEYS after flush", &r, "$1\r\nb\r\n");
    assert_not_contains("KEYS after flush", &r, "-ERR");

    println!("✅ tester passed (RESP2)");
}
