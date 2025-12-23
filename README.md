# 🚀 Local Chat - P2P Chat Application

A serverless, peer-to-peer chat application written in Rust that enables real-time communication within local networks without requiring a central server.

## 🌟 Features

### ✅ Currently Implemented (MVP - Phase 1 & 2)
- **Serverless Architecture**: Direct P2P communication without central server
- **Local Network Discovery**: Automatic detection of devices in the same network subnet (UDP broadcast)
- **TCP Peer Connections**: Direct peer-to-peer TCP links between discovered peers
- **Real-time Messaging**: Broadcast chat among connected peers
- **Channels (Optional)**: Scope conversations by channel using `--channel/-c`; default is global room when unset
- **Nickname via CLI**: Set nickname using `--nick` or `-nick`
- **Rich Terminal UI**: Interactive interface built with ratatui
- **Vim Keybindings**: Normal/Insert mode navigation (press `i` to type, `Esc` to navigate)
- **Rust-Powered**: Built with modern Rust for safety, performance, and concurrency
- **Async I/O**: Non-blocking network operations using Tokio
- **Structured Logging**: Comprehensive logging with tracing
- **Cross-Platform**: Supports Windows, macOS, and Linux

### 🚧 Planned Features (Phase 3)
- **File Transfer**: Share files directly between peers
- **End-to-End Encryption**: Secure message and file transmission
- **Message History**: Session-based chat history (persistent)
- **UPnP/NAT Traversal**: P2P library support to traverse home routers (UPnP)

## 🏗️ Architecture

The application follows a modular architecture with clear separation of concerns:

```
src/
├── main.rs                 # Application entry point and orchestration
├── config.rs               # Configuration management
├── lib.rs                  # Library exports
├── network/                # Network layer
│   ├── mod.rs              # Module exports
│   ├── discovery.rs        # Peer discovery via UDP broadcast
│   ├── peer.rs             # TCP peer connection management
│   └── protocol.rs         # Network protocol definitions
├── message/                # Message handling
│   ├── mod.rs              # Module exports
│   ├── types.rs            # Message type definitions and serialization
│   └── handler.rs          # Message processing logic
└── ui/                     # User interface (ratatui TUI)
    ├── mod.rs              # Module exports
    ├── app.rs              # Application state management
    ├── tui.rs              # Terminal initialization/cleanup
    ├── terminal.rs         # Event handling and rendering
    ├── widgets.rs          # Custom UI widgets
    └── theme.rs            # UI theming and colors

core/                       # Shared core library
└── src/
    ├── lib.rs              # Core library exports
    └── message.rs          # Shared message types

web/                        # WebAssembly browser support (WIP)
└── src/
    └── lib.rs              # WASM bindings
```

## 🚀 Quick Start

### Prerequisites

- Rust 1.70+ (2021 edition)
- Cargo package manager

### Installation

1. **Clone the repository**
   ```bash
   git clone <repository-url>
   cd local-chat
   ```

2. **Build the project**
   ```bash
   cargo build --release
   ```

3. **Run the application**
   ```bash
   # Using cargo (note the `--` to separate cargo args from app args)
   cargo run -- --nick alice

   # With a channel (only peers with the same channel communicate)
   cargo run -- --nick alice -c dev

   # Run the built binary directly (no need for the separating `--`)
   ./target/debug/local-chat --nick alice -c dev
   ```

### Usage

1. **Start the application** on multiple devices within the same local network
2. **Nickname**: Set with `--nick` or `-nick` (required; positional nickname is not supported)
3. **Channel (optional)**: Use `--channel` or `-c` to isolate rooms; omit to join the global room
4. **Automatic Discovery**: Instances with matching channel discover each other
5. **Real-time Status**: Monitor connected peers and network status

### Keyboard Shortcuts

The application uses vim-style keybindings with Normal and Insert modes:

#### Normal Mode (default)
| Key | Action |
|-----|--------|
| `i` | Enter Insert mode (start typing) |
| `q` | Quit application |
| `Tab` | Cycle focus between panes |
| `j` / `↓` | Scroll messages down |
| `k` / `↑` | Scroll messages up |
| `g` | Scroll to top of messages |
| `G` | Scroll to bottom of messages |

#### Insert Mode
| Key | Action |
|-----|--------|
| `Esc` | Return to Normal mode |
| `Enter` | Send message |
| `←` / `→` | Move cursor |
| `Backspace` | Delete character before cursor |
| `Delete` | Delete character after cursor |
| `Home` / `Ctrl+a` | Move cursor to start |
| `End` / `Ctrl+e` | Move cursor to end |

## 📡 Network Protocol

### Discovery Protocol (UDP)
- **Port**: 7878 (default, configurable)
- **Method**: Broadcast messages to local network subnets
- **Message Format**: JSON-serialized discovery messages
- **Supported Networks**: 
  - 192.168.x.x (typical home networks)
  - 10.x.x.x (corporate networks)
  - 172.16-31.x.x (private networks)

### Communication Protocol (TCP)
- **Port Range**: 8000-8100 (configurable)
- **Message Format**: JSON with newline delimiters
- **Connection**: Direct peer-to-peer TCP connections

### Message Types
- `discovery`: Announce presence to network (includes optional `channel`)
- `discovery_response`: Respond to discovery requests (includes optional `channel`)
- `message`: Chat messages between peers (includes optional `channel`)
- `user_join`/`user_leave`: User presence notifications (include optional `channel`)
- `heartbeat`: Keep-alive messages

## 🛠️ Configuration

Default configuration can be customized in `src/config.rs`:

```rust
pub struct Config {
    pub discovery_port: u16,        // Default: 7878
    pub tcp_port_range: (u16, u16), // Default: (8000, 8100)
    pub username: String,           // Default: system username
    pub network_timeout: u64,       // Default: 10 seconds
    pub heartbeat_interval: u64,    // Default: 30 seconds
    pub channel: Option<String>,    // Default: None (global room)
}
```

## 📦 Dependencies

### Core Dependencies
- **tokio**: Async runtime for non-blocking I/O
- **serde** + **serde_json**: Serialization for network messages
- **uuid**: Unique peer identification
- **chrono**: Timestamp handling
- **anyhow**: Error handling
- **tracing**: Structured logging

### Network Dependencies
- **local-ip-address**: Local network detection
- **whoami**: System username detection
- **socket2**: Low-level socket configuration

### UI Dependencies
- **ratatui**: Terminal user interface framework
- **crossterm**: Cross-platform terminal manipulation

### WASM (Browser) Dependencies
- **wasm-bindgen**, **web-sys**, **js-sys**: WebAssembly and Web APIs
- **console_error_panic_hook**: Better error messages in browser console
- **once_cell**: Simple global state for browser bridge

## 🔧 Development

### Building
```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Check for compilation errors
cargo check
```

### Running Tests
```bash
cargo test
```

### Code Quality
```bash
# Format code
cargo fmt

# Lint code
cargo clippy

# Fix common issues
cargo fix
```

## 🚦 Current Status

### ✅ Phase 1: MVP (Completed)
- [x] Basic project structure
- [x] Network infrastructure
- [x] Message protocol definitions
- [x] Terminal UI foundation
- [x] Configuration system
- [x] Logging and error handling

### ✅ Phase 2: Core Features (Completed)
- [x] Real UDP broadcast discovery (native)
- [x] TCP peer connections (native)
- [x] Interactive terminal UI (native)
- [x] Real-time messaging (native)
- [x] Rich terminal interface with ratatui
- [x] Vim-style keybindings (Normal/Insert modes)

### 🚧 Phase 3: Advanced Features (In Progress)
- [ ] Browser demo via BroadcastChannel (WASM)
- [ ] File transfer capabilities
- [ ] End-to-end encryption
- [ ] Configuration files
- [ ] Network diagnostics
- [ ] Error handling improvements

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Development Guidelines
- Follow Rust conventions and idioms
- Add tests for new functionality
- Update documentation for API changes
- Use `cargo fmt` and `cargo clippy` before committing

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🔒 Security Considerations

### Current Security Model
- **Local Network Only**: Communication restricted to local network segments
- **No External Connections**: Blocks internet-based connections
- **Input Validation**: Sanitizes and validates all user inputs

### Future Security Enhancements
- **AES-256 Encryption**: End-to-end message encryption
- **Peer Authentication**: Cryptographic peer verification
- **Message Integrity**: Prevention of message tampering
- **Replay Protection**: Prevent message replay attacks

## 🐛 Troubleshooting

### Common Issues

1. **No peers discovered**
   - Ensure devices are on the same network subnet
   - Check firewall settings (UDP port 7878)
   - Verify network supports broadcast messages

2. **Connection failed**
   - Check TCP port range (8000-8100) availability
   - Ensure no conflicting applications
   - Verify network connectivity between devices

3. **Permission denied**
   - Some networks require administrator privileges for broadcast
   - Try running with elevated permissions if necessary

### Logging
Enable detailed logging with:
```bash
RUST_LOG=debug cargo run
```

## 📊 Performance

### Benchmarks
- **Memory Usage**: < 50MB (typical operation)
- **CPU Usage**: < 5% (idle state)
- **Network Overhead**: < 1KB/min (heartbeat)
- **Discovery Time**: < 5 seconds (local network)
- **Message Latency**: < 100ms (local network)

### Scalability
- **Maximum Peers**: 50 concurrent connections (configurable)
- **Message Throughput**: Limited by local network bandwidth
- **Resource Scaling**: Linear with number of active peers

## 🌐 Supported Platforms

| Platform | Status | Notes |
|----------|--------|-------|
| Linux x86_64 | ✅ Supported | Tested on Ubuntu 20.04+ |
| macOS x86_64 | ✅ Supported | Tested on macOS 12+ |
| macOS ARM64 | ✅ Supported | Native Apple Silicon support |
| Windows x86_64 | ✅ Supported | Tested on Windows 10+ |
| Other platforms | 🔄 Untested | Should work with Rust support |

## 📚 Documentation

- [Product Requirements Document (PRD)](PRD.md) - Detailed project specifications
- [Architecture Guide](docs/architecture.md) - System design and components
- [Network Protocol](docs/protocol.md) - Communication protocol details
- [API Documentation](docs/api.md) - Internal API reference

## 🎯 Roadmap

### Next Up
- Complete browser demo (WASM) with BroadcastChannel
- Add comprehensive testing
- Configuration file support

### Future
- File transfer functionality
- End-to-end message encryption
- Mobile app versions
- Web interface
- Plugin system
- Advanced network diagnostics

---

**Built with ❤️ in Rust** | **Made for local networks** | **Zero servers required**
