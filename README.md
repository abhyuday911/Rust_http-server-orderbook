# rust_http — An In-Memory Order Matching Engine Over HTTP

Most trading engine tutorials give you a sorted list and call it a day. This project builds the entire pipeline: authentication, session management, order intake, price-time priority matching, and a deterministic reply contract — all running on a single async runtime with zero external databases. Every order flows through a channel, gets matched by an isolated engine task, and the caller gets a typed result back before the HTTP response closes. No polling. No eventual consistency. One roundtrip.

---

## Architectural Fundamentals

### 1. Actor-Isolated Matching Engine

The matching engine runs as a standalone `tokio::spawn` task. It never shares a thread with your HTTP handlers. Communication happens exclusively through a bounded `mpsc` channel (`capacity: 100`), which acts as natural backpressure — if the engine can't keep up, order submission blocks at the HTTP layer, not inside the book.

Every inbound message (`EngineMessage`) carries a `oneshot::Sender`. The engine processes the order, determines the outcome, and fires exactly one reply back through that sender. The HTTP handler `await`s the `oneshot::Receiver` and maps the result directly to an HTTP status code. This is a **request-reply actor pattern** — the engine owns the book, nobody else touches it, and every caller gets a deterministic response.

```
HTTP Handler ──▶ mpsc::channel(100) ──▶ Engine Task
                                            │
                                    Mutex<OrderBook>
                                            │
HTTP Handler ◀── oneshot::channel ◀─────────┘
```

**Why this matters:** The engine holds a `Mutex<OrderBook>` lock for the duration of each match cycle. Because there is exactly one consumer on the `mpsc` channel, there is zero lock contention on the book. The mutex exists purely as a safety contract — not as a performance bottleneck.

---

### 2. The `BTreeMap<Reverse<u64>>` Trick

An orderbook is two sorted data structures fighting over the same liquidity. Asks are trivial — a `BTreeMap<u64, Vec<Order>>` gives you the lowest ask via `.keys().next()`. Bids are the problem. You need the _highest_ bid first, but `BTreeMap` iterates ascending.

The solution: wrap bid price keys in `std::cmp::Reverse`.

```rust
bids: BTreeMap<Reverse<u64>, Vec<Order>>  // highest bid = first key
asks: BTreeMap<u64, Vec<Order>>           // lowest ask  = first key
```

Both sides now yield best price via `.keys().next()` — same code path, same O(log n) access, no secondary index. Orders at the same price level sit in a `Vec<Order>`, which enforces **FIFO (price-time priority)** at each level. The first order in wins. When its quantity is fully consumed, `.remove(0)` shifts the next order up. When the `Vec` drains, the entire price level gets pruned from the tree.

---

### 3. Matching Logic: Aggressive Fill or Rest

Both market and limit orders walk the opposing side of the book greedily:

- **Market Orders** eat through the best available levels until quantity is exhausted or liquidity runs dry. There is no limit price guard — you get whatever the book offers. The engine then replies with one of three states: `FullySettled`, `PartiallySettled`, or `CompletelyRejected`.

- **Limit Orders** do the same aggressive walk, but with a twist: _unfilled remainder gets inserted into the book_. A limit buy at $100 will first consume any asks ≤ $100, then park the leftover quantity at the $100 bid level. This is **immediate-or-cancel matching with passive resting** — the same model used by real exchanges. If the entire quantity fills during the walk, the engine replies `ImmeadiatelySettled` (never touching the book). If nothing matches, the order rests and the engine replies with `AddedToOrderBook(order_id)`.

The engine assigns monotonically increasing `order_id` values (`next_order_id` counter on the book), guaranteeing unique identification without UUIDs or external ID generators.

---

### 4. Auth & Session Layer

User management runs entirely in-memory via `HashMap<String, User>` behind an `Arc<Mutex<>>`.

- **Signup** hashes passwords with `bcrypt` (cost factor = default 12) and issues a `UUIDv4` session token set via `Set-Cookie`. Duplicate usernames are rejected with `409 Conflict`.
- **Signin** retrieves the user by username, runs `bcrypt::verify` against the stored hash, and returns user data on success.

Session IDs map to usernames in a separate `HashMap<String, String>`, ready for middleware lookup on authenticated routes.

---

### 5. Clean Module Boundaries

```
src/
├── main.rs                         # AppState, data types, server bootstrap
├── engine.rs                       # Matching engine (single consumer task)
└── controllers/
    └── v1/
        ├── mod.rs                  # Re-exports
        ├── index.rs                # Health check
        ├── sign_up.rs              # Registration + bcrypt + session
        ├── sign_in.rs              # Auth + bcrypt verify
        └── orders/
            ├── mod.rs              # Re-exports
            ├── create_limit_order.rs
            ├── create_market_order.rs
            └── get_orderbook.rs    # Snapshot of current book state
```

Controllers never mutate the orderbook. They serialize the order, push it through the `mpsc` channel, await the `oneshot` reply, and translate `EngineReply` variants into HTTP responses. The engine is the single source of truth.

---

## The Stack

| Layer             | Choice                            | Rationale                                                                                                          |
| ----------------- | --------------------------------- | ------------------------------------------------------------------------------------------------------------------ |
| **Runtime**       | `tokio`                           | Async task spawning, channel primitives (`mpsc`, `oneshot`, `Mutex`) — the engine runs as a first-class async task |
| **HTTP**          | `actix-web 4`                     | Battle-tested, runs on tokio, zero-copy JSON extraction via `web::Json<T>`                                         |
| **Serialization** | `serde` + `serde_json`            | Derive-based (de)serialization for all request/response types and the order book                                   |
| **Auth**          | `bcrypt` + `uuid`                 | Password hashing with configurable cost factor, UUIDv4 session tokens                                              |
| **Storage**       | In-memory (`HashMap`, `BTreeMap`) | Zero external dependencies. The entire state lives in `AppState`, shared via `Arc<Mutex<>>`                        |

---

## Quick Start

```bash
# Clone
git clone https://github.com/abhyuday911/Rust_http-server-orderbook.git
cd Rust_http-server-orderbook

# Build & run
cargo run
# Server starts on http://127.0.0.1:8080
```

### API

| Method | Endpoint               | Description                                                            |
| ------ | ---------------------- | ---------------------------------------------------------------------- |
| `GET`  | `/`                    | Health check                                                           |
| `POST` | `/signup`              | Register a new user (JSON body: `username`, `name`, `password`, `age`) |
| `POST` | `/signin`              | Authenticate (JSON body: `username`, `password`)                       |
| `POST` | `/create_limit_order`  | Submit a limit order to the engine                                     |
| `POST` | `/create_market_order` | Submit a market order to the engine                                    |
| `GET`  | `/get_orderbook`       | Snapshot of current bids, asks, and next order ID                      |

### Order Payload

```json
{
  "user_id": "alice",
  "amount": 10,
  "asset": "BTC",
  "price": 50000,
  "side": "Buy",
  "order_kind": "Limit"
}
```

`side`: `"Buy"` | `"Sell"` · `order_kind`: `"Limit"` | `"Market"`

---

## The Vision

- [ ] **Persistent storage** — Plug in a WAL or append-only log so the book survives restarts
- [ ] **Auth middleware** — Session-based guards on order routes using the existing session ID map
- [ ] **User portfolio tracking** — Per-user asset balances and open order lists
- [ ] **Average price calculation** — The `EngineReply` variants already carry price slots (`u32`); wire them through the matching walk
- [ ] **WebSocket price feed** — Real-time bid/ask stream for connected clients
- [ ] **Multi-asset books** — Route orders to per-asset engine instances
- [ ] **Benchmarks** — Throughput and latency profiling under synthetic order load

---

## License

MIT
