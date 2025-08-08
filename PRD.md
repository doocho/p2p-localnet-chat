# Local Chat - P2P Chat Application PRD

## Project Overview

### Product Name
Local Chat

### Project Goal
Develop a real-time chat application in Rust that operates in a P2P manner within local networks without requiring a central server

### Core Values
- **Serverless**: Direct P2P communication without central server
- **Local Network Optimized**: Automatic connection with devices in the same network subnet (e.g., 10.1.1.X)
- **Simplicity**: Instant chat capability with just program execution

## Functional Requirements

### Core Features

#### 1. Automatic Network Discovery and Connection
- **Function**: Automatically detect devices running the same program on the local network when the program starts
- **Details**:
  - Automatic detection of current network subnet (e.g., 10.1.1.X, 192.168.1.X)
  - Peer discovery through UDP broadcast
  - Automatic TCP connection setup
- **Priority**: P0 (Critical)

#### 2. P2P Messaging
- **Function**: Real-time message exchange with discovered peers
- **Details**:
  - 1:1 private messaging
  - Group chat (all connected peers)
  - Message delivery status (sending, delivered, failed)
- **Priority**: P0 (Critical)

#### 3. User Interface
- **Function**: Intuitive and easy-to-use CLI/TUI interface
- **Details**:
  - Display list of connected peers
  - Message history display
  - Real-time message input/output
  - User nickname configuration
- **Priority**: P0 (Critical)

### Extended Features

#### 4. File Transfer
- **Function**: File sharing between peers
- **Details**:
  - Drag and drop support
  - Transfer progress indicator
  - File size limit configuration
- **Priority**: P1 (Optional)

#### 5. Encryption
- **Function**: Message and file transfer encryption
- **Details**:
  - End-to-end encryption
  - Key exchange protocol
- **Priority**: P1 (Optional)

## Technical Requirements

### Development Environment
- **Language**: Rust (Edition 2021+)
- **Minimum Supported Version**: rustc 1.70+

### Key Dependencies
- **Networking**: tokio (async runtime)
- **Network Scanning**: local-ip-address (local IP detection)
- **Serialization**: serde, serde_json (message protocol)
- **TUI**: crossterm + ratatui (terminal UI)
- **Logging**: tracing, tracing-subscriber

### Architecture

#### Network Protocol
1. **Discovery Protocol** (UDP)
   - Port: 7878 (default)
   - Broadcast message: `{"type": "discovery", "username": "username", "port": TCP_port}`

2. **Communication Protocol** (TCP)
   - Port: Dynamic allocation
   - Message format: JSON
   ```json
   {
     "type": "message|file|user_join|user_leave",
     "sender": "username",
     "timestamp": "ISO_8601",
     "content": "message_content",
     "recipient": "recipient|all"
   }
   ```

#### Module Structure
```
src/
├── main.rs                 # Program entry point
├── network/
│   ├── mod.rs              # Network module
│   ├── discovery.rs        # Peer discovery
│   ├── peer.rs             # Peer connection management
│   └── protocol.rs         # Protocol definitions
├── ui/
│   ├── mod.rs              # UI module
│   ├── app.rs              # App state management
│   └── terminal.rs         # Terminal UI
├── message/
│   ├── mod.rs              # Message module
│   ├── types.rs            # Message type definitions
│   └── handler.rs          # Message handling
└── config.rs               # Configuration management
```

## User Scenarios

### Basic Usage Scenario
1. User runs the program: `cargo run` or `./local-chat`
2. Program automatically scans the local network
3. Automatically connects with other devices running the same program
4. Displays list of connected users
5. Starts real-time chatting

### User Experience
```
Local Chat v1.0.0
Connected to network: 10.1.1.0/24

🟢 Online Users (3):
  - Alice (10.1.1.5)
  - Bob (10.1.1.12)
  - You (10.1.1.8)

[14:30] Alice: Hello everyone!
[14:31] Bob: Nice to meet you
[14:32] You: > Hi! Nice to meet you all.

Type your message (Press Enter to send, Ctrl+C to quit):
> _
```

## Performance Requirements

### Network Performance
- Peer discovery time: < 5 seconds
- Message transmission latency: < 100ms (local network)
- Maximum concurrent connected peers: 50 users

### Resource Usage
- Memory usage: < 50MB (normal operation)
- CPU utilization: < 5% (idle state)
- Network overhead: < 1KB/min (heartbeat)

## Security Considerations

### Basic Security
- Allow communication only within local network
- Block external internet connections
- User input validation and sanitization

### Advanced Security (P1)
- Message encryption (AES-256)
- Peer authentication mechanism
- Malicious message filtering

## Deployment and Installation

### Build Targets
- Windows (x86_64-pc-windows-msvc)
- macOS (x86_64-apple-darwin, aarch64-apple-darwin)
- Linux (x86_64-unknown-linux-gnu)

### Installation Methods
1. **Build from source**: `cargo build --release`
2. **Binary distribution**: GitHub Releases
3. **Package managers**: Homebrew (macOS), Chocolatey (Windows)

## Development Schedule

### Phase 1: MVP (2 weeks)
- [ ] Basic network discovery
- [ ] P2P connection setup
- [ ] Simple CLI messaging

### Phase 2: UI Improvement (1 week)
- [ ] TUI interface implementation
- [ ] User experience enhancement
- [ ] Error handling strengthening

### Phase 3: Advanced Features (2 weeks)
- [ ] File transfer
- [ ] Encryption
- [ ] Configuration system

## Success Metrics

### Functional Metrics
- 95%+ peer discovery success rate
- 99%+ message transmission success rate
- Network connection time within 5 seconds

### User Experience Metrics
- First use within 3 steps after installation
- Intuitive UI usable without additional instructions
- Stable operation in various network environments

## Risks and Constraints

### Technical Risks
- Connection failures due to network firewalls
- P2P connection limitations in NAT environments
- Platform-specific network API differences

### Business Constraints
- Operation only in local networks (no internet connectivity)
- Limited user management without central server
- No permanent message history storage (deleted when session ends)

---

*This PRD provides development guidelines for the Local Chat P2P chat application.*
