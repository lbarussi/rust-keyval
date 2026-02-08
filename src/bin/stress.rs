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

fn main() {
    let host = "127.0.0.1:6374";
    let clients = 50;
    let ops_per_client = 200;


    {
        let mut s = TcpStream::connect(host).unwrap();
        s.set_nodelay(true).unwrap();
        let r = send_and_read_all(&mut s, b"*1\r\n$8\r\nFLUSHALL\r\n");
        if !r.contains("+OK\r\n") {
            panic!("bad FLUSHALL resp: {:?}", r);
        }
    }

    let start = Instant::now();
    let mut handles = Vec::new();

    for id in 0..clients {
        let host = host.to_string();
        handles.push(std::thread::spawn(move || {
            let mut s = TcpStream::connect(&host).unwrap();
            s.set_nodelay(true).unwrap();

            for i in 0..ops_per_client {

                let r = send_and_read_all(&mut s, b"*2\r\n$4\r\nINCR\r\n$3\r\nctr\r\n");
                if r.contains("-ERR") || r.is_empty() || !r.contains(':') {
                    panic!("client {id} op {i}: bad INCR resp: {:?}", r);
                }


                let key = format!("k{id}");
                let val = format!("v{i}");

                let set = format!(
                    "*3\r\n$3\r\nSET\r\n${}\r\n{}\r\n${}\r\n{}\r\n",
                    key.len(), key, val.len(), val
                );
                let r = send_and_read_all(&mut s, set.as_bytes());
                if !r.contains("+OK\r\n") {
                    panic!("client {id} op {i}: bad SET resp: {:?}", r);
                }

                let get = format!(
                    "*2\r\n$3\r\nGET\r\n${}\r\n{}\r\n",
                    key.len(), key
                );
                let r = send_and_read_all(&mut s, get.as_bytes());


                let expected_bulk = format!("${}\r\n{}\r\n", val.len(), val);
                if !r.contains(&expected_bulk) {
                    panic!(
                        "client {id} op {i}: bad GET resp: {:?} (expected {:?})",
                        r, expected_bulk
                    );
                }
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }


    let expected = clients * ops_per_client;
    let mut s = TcpStream::connect(host).unwrap();
    s.set_nodelay(true).unwrap();
    let r = send_and_read_all(&mut s, b"*2\r\n$3\r\nGET\r\n$3\r\nctr\r\n");

    println!("FINAL ctr: {:?}", r);


    let expected_bulk = format!("${}\r\n{}\r\n", expected.to_string().len(), expected);
    if !r.contains(&expected_bulk) {
        eprintln!("FAIL: expected ctr bulk {:?}", expected_bulk);
        std::process::exit(1);
    }

    println!(
        "✅ stress passed: clients={} ops/client={} total_ops={} elapsed={:?}",
        clients,
        ops_per_client,
        expected,
        start.elapsed()
    );
}
