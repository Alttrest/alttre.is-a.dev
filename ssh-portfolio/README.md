# Alttre SSH Portfolio (`russh` + `ratatui`)

Interactive terminal portfolio server for **Ali Turan (`@Alttre`)** — built in Rust with `russh` and `ratatui`.

Instead of spawning a login shell (`bash`/`fish`), every incoming SSH connection gets its own isolated, in-memory Ratatui event loop (`App` + `Terminal<CrosstermBackend<TerminalHandle>>`) wired directly to the SSH channel socket.

## Architecture

- **`src/ssh/mod.rs`** — `russh::server::Server` implementation, persistent Ed25519 host key management, session limits (`Semaphore`), and graceful shutdown (`Ctrl+C` / `SIGTERM`).
- **`src/ssh/session.rs`** — Per-connection `russh::server::Handler` spawning a lock-free actor task (`tokio::select!` over incoming SSH bytes, PTY resize events, and a 100ms animation tick). Non-interactive `ssh host <cmd>` requests receive a crisp ANSI business card and exit cleanly.
- **`src/ssh/terminal.rs`** — In-memory `TerminalHandle` (`std::io::Write`) forwarding flushed Ratatui diff frames over a bounded `mpsc` channel to `russh::server::Handle::data(...)`, with alt-screen/cursor/SGR-mouse enter & restore sequences.
- **`src/ssh/input.rs`** — Pure VT100/ANSI/UTF-8 + SGR mouse parser converting raw SSH channel bytes into `InputEvent`s (including Turkish UTF-8 characters and split packets).
- **`src/ui/` & `src/data/`** — Zero-`russh` Ratatui portfolio UI faithful to `alttre.is-a.dev`'s ThinkPad TrackPoint Red (`#DA291C`), Obsidian (`#1E1B18`), Warm Cream (`#FAF6EE`), and Sage (`#5E7060`) palette, with live **EN/TR** toggle (`e`), **Dark/Cream** theme toggle (`t`), mouse support, interactive project filters, skill cross-links, flippable competition lanyard cards, and Dev Logs reader.

## Quick Start (Local)

```bash
cd ssh-portfolio
cargo run
```

Then connect from another terminal:

```bash
ssh -p 2222 localhost
```

Non-interactive business card mode:

```bash
ssh -p 2222 localhost card
```

## Controls

| Key | Action |
|---|---|
| `1`–`6` / `Tab` / `Shift+Tab` | Switch sections (Intro, Projects, Skills, Competitions, About, Contact) |
| `↑` `↓` / `j` `k` / Mouse Wheel | Scroll / move selection |
| `←` `→` / `f` | Filter projects or switch cards |
| `Enter` / `Space` | Open project/dev-log detail or flip competition lanyard badge |
| `e` | Toggle **English / Türkçe** |
| `t` | Toggle **Obsidian Dark / Warm Cream** theme |
| `?` | Help overlay |
| `q` / `Ctrl+C` / `Ctrl+D` | Quit & restore terminal |

## Environment Variables

| Variable | Default | Description |
|---|---|---|
| `SSH_BIND` | `0.0.0.0` | Bind IP address |
| `SSH_PORT` | `2222` | Listening TCP port (set `22` when deploying behind a dedicated host/IP) |
| `SSH_HOST_KEY` | `host_key` | Path to OpenSSH Ed25519 host key (auto-generated if missing; empty = ephemeral) |
| `MAX_SESSIONS` | `100` | Max concurrent interactive SSH sessions |
| `IDLE_TIMEOUT_SECS` | `600` | Inactivity disconnect timeout in seconds |
| `RUST_LOG` | `info` | Tracing filter (`debug`, `info`, `ssh_portfolio=debug`) |

## Docker Deploy (VPS / Fly.io / Railway)

```bash
docker build -t alttre-ssh-portfolio ./ssh-portfolio
docker run -d \
  --name alttre-ssh \
  -p 22:2222 \
  -v alttre_ssh_data:/data \
  --restart unless-stopped \
  alttre-ssh-portfolio
```

Point a DNS `A` record (e.g. `ssh.alttre.is-a.dev` or a VPS IP) to the container host, and visitors can connect directly with:

```bash
ssh ssh.alttre.is-a.dev
```

## Testing

```bash
cargo test
```
