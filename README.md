# Rust Key-Value Server (RESP2) — `rust-keyval`

A lightweight Redis-like key-value server written in Rust, built for learning and portfolio purposes.

It implements a subset of Redis commands and speaks **RESP2** (Redis Serialization Protocol v2) over raw TCP, plus an optional **Prometheus metrics** endpoint for observability.

---

## Features

- **RESP2 protocol** over TCP (compatible with typical Redis clients at the protocol level)
- Commands implemented:
  - `SET`, `GET`, `INCR`, `DEL`, `EXISTS`
  - `EXPIRE` (TTL)
  - `KEYS`, `FLUSHALL`
- **Pipeline support** (multiple commands in the same TCP payload)
- **Fragmentation-safe parsing** (a command can arrive in multiple TCP chunks)
- **TTL cleaner** (background expiration)
- **Prometheus exporter** (`/metrics`) + Grafana/Prometheus stack via Docker Compose
- Load testing binaries (stress/load tests)

---

## Protocol: RESP2 (What it means)

This server uses **RESP2**, the same text-based wire protocol Redis uses.

In RESP2, commands are sent as arrays of bulk strings:

Example: `SET foo bar`

```text
*3\r\n
$3\r\nSET\r\n
$3\r\nfoo\r\n
$3\r\nbar\r\n
````

The server replies using RESP2 types:

* Simple string: `+OK\r\n`
* Error: `-ERR ...\r\n`
* Integer: `:1\r\n`
* Bulk string: `$3\r\nbar\r\n`
* Null bulk string: `$-1\r\n`
* Array: `*2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n`

**Why RESP2 matters:** it allows standard Redis client libraries to talk to the server without inventing a custom protocol.

---

## Requirements

* Rust (stable) + Cargo
* Linux recommended (for process metrics via `/proc`)
* Docker + Docker Compose (optional, for monitoring stack)

---

## Quick Start (Local)

### 1) Build & run the server

```bash
cargo run --bin rust-keyval
```

By default it binds to:

* TCP server: `127.0.0.1:6374`
* Metrics: `127.0.0.1:9100`

You can override using environment variables:

```bash
KEYVAL_BIND=0.0.0.0:6374 METRICS_BIND=0.0.0.0:9100 cargo run --bin rust-keyval
```

---

## Using the CLI (Redis-like)

This repo includes a small CLI client that speaks RESP2.

### SET / GET

```bash
cargo run --bin keyval-cli -- --host 127.0.0.1 --port 6374 SET a 1
cargo run --bin keyval-cli -- --host 127.0.0.1 --port 6374 GET a
```

Expected output:

```text
OK
1
```

### INCR

```bash
cargo run --bin keyval-cli -- --host 127.0.0.1 --port 6374 INCR ctr
cargo run --bin keyval-cli -- --host 127.0.0.1 --port 6374 INCR ctr
```

Expected:

```text
1
2
```

### EXISTS / DEL

```bash
cargo run --bin keyval-cli -- --host 127.0.0.1 --port 6374 EXISTS a
cargo run --bin keyval-cli -- --host 127.0.0.1 --port 6374 DEL a
cargo run --bin keyval-cli -- --host 127.0.0.1 --port 6374 EXISTS a
```

Expected:

```text
1
1
0
```

### EXPIRE

```bash
cargo run --bin keyval-cli -- --host 127.0.0.1 --port 6374 SET t 123
cargo run --bin keyval-cli -- --host 127.0.0.1 --port 6374 EXPIRE t 1
sleep 2
cargo run --bin keyval-cli -- --host 127.0.0.1 --port 6374 GET t
```

Expected:

```text
OK
1
NULL
```

### KEYS / FLUSHALL

```bash
cargo run --bin keyval-cli -- --host 127.0.0.1 --port 6374 SET a 1
cargo run --bin keyval-cli -- --host 127.0.0.1 --port 6374 SET b 2
cargo run --bin keyval-cli -- --host 127.0.0.1 --port 6374 KEYS
cargo run --bin keyval-cli -- --host 127.0.0.1 --port 6374 FLUSHALL
cargo run --bin keyval-cli -- --host 127.0.0.1 --port 6374 KEYS
```

---

## Monitoring (Prometheus + Grafana)

A full monitoring stack is provided under `./monitoring/`.

### 1) Start the stack

```bash
cd monitoring
docker compose up -d --build
```

This runs:

* `keyval` (TCP `6374`, metrics `9100`)
* `prometheus` (`9090`)
* `grafana` (`3000`)

### 2) Verify metrics

```bash
curl -s http://localhost:9100/metrics | head -n 30
```

You should see metrics like:

* `keyval_active_connections`
* `keyval_cmd_total{cmd="SET"} ...`
* `keyval_cmd_latency_seconds_bucket{cmd="SET",le="..."} ...`
* `keyval_keys_count`
* (optional) `process_resident_memory_bytes`, `process_cpu_seconds_total`

### 3) Prometheus UI

Open:

* [http://localhost:9090](http://localhost:9090)

Check:

* Status → Targets → `rust-keyval` should be **UP**

### 4) Grafana UI

Open:

* [http://localhost:3000](http://localhost:3000) (admin / admin)

Add a Prometheus datasource:

* URL: `http://prometheus:9090`

Suggested PromQL queries:

**Total QPS**

```promql
sum(rate(keyval_cmd_total[1m]))
```

**QPS by command**

```promql
sum by (cmd) (rate(keyval_cmd_total[1m]))
```

**p95 latency by command**

```promql
histogram_quantile(
  0.95,
  sum(rate(keyval_cmd_latency_seconds_bucket[5m])) by (le, cmd)
)
```

**Keys count**

```promql
keyval_keys_count
```

**RSS memory (MB)**

```promql
process_resident_memory_bytes / 1024 / 1024
```

**CPU % (approx, 1 core = 100%)**

```promql
rate(process_cpu_seconds_total[1m]) * 100
```

---

## Testing

### Basic RESP tests

```bash
./tests.sh
```

### Extended tester (pipeline + fragmentation + commands)

```bash
cargo run --bin tester
```

### Stress test (many clients, many ops)

```bash
cargo run --bin stress
```

### Heavy load test (parallel, pipelining, mixed commands)

```bash
cargo run --release --bin loadtest
```

You can tune load via env vars:

```bash
HOST=127.0.0.1:6374 CLIENTS=500 OPS=5000 PIPE=50 RT=400 cargo run --release --bin loadtest
```

Meaning:

* `CLIENTS`: number of concurrent TCP connections
* `OPS`: number of iterations per client
* `PIPE`: pipeline size (commands per TCP write)
* `RT`: read timeout in milliseconds

This simulates many clients issuing a mixed workload in parallel with aggressive pipelining.

---

## Notes & Limitations

* This is a learning/portfolio project, not a production-ready Redis replacement.
* Persistence is not implemented (in-memory only).
* Command coverage is intentionally small (focused on core mechanics).
* RESP2 is supported; RESP3 is not implemented.
