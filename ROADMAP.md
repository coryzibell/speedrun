# Speedo Roadmap

This document outlines planned features and improvements for speedo, a fast network speed test tool built in Rust.

## Philosophy

Maintain speedo's core principles:
- Fast and lightweight
- No external system dependencies (no curl, wget, or system SSL libs)
- Cross-platform compatibility
- Simple and intuitive user experience

---

## 🌐 Server Management

### Remote Server List Management (like tealdeer)
**Priority: High** | **Target: v0.4.0**

- Host server list on GitHub/CDN as JSON/TOML
- Auto-update on launch or via `speedo --update-servers`
- Fallback to embedded list if fetch fails
- Community-contributed servers with verification
- Include metadata: location, provider, reliability score, uptime stats

### Automatic Closest Server Selection
**Priority: High** | **Target: v0.4.0**

- Geolocation-based server selection using IP APIs
- ICMP ping test to all servers, select lowest latency
- `--auto` flag to run against closest server automatically
- Cache closest server result for performance
- Allow manual override of auto-selection

### Server Health Monitoring
**Priority: Medium** | **Target: v0.5.0**

- Mark unreliable servers automatically
- Crowdsourced uptime/speed data
- `--verify-servers` command to test all servers and report status
- Automatic removal of dead servers from rotation

---

## 🚀 Performance & Testing Enhancements

### Multi-Server Testing
**Priority: High** | **Target: v0.5.0**

- `--all` flag to test all servers sequentially
- `--parallel N` to test N servers simultaneously
- Aggregate report showing best/worst/average speeds
- Export results to CSV/JSON for analysis
- Visual comparison of server performance

### Upload Speed Testing
**Priority: Medium** | **Target: v0.6.0**

- Find servers supporting POST/PUT endpoints
- Generate random data in-memory for upload tests
- Bidirectional test mode (download then upload)
- Report upload speed alongside download speed
- Detect asymmetric connection speeds

### Latency & Jitter Measurements
**Priority: Medium** | **Target: v0.6.0**

- Measure ping/latency alongside download speed
- Track jitter (latency variance) during transfer
- Report packet loss if detectable
- Connection quality score (good/fair/poor)

### Historical Tracking
**Priority: High** | **Target: v0.5.0**

- Save test results to local database (SQLite)
- `--history` to view past tests with graphs
- Trend analysis: "Your speed improved 15% this month"
- Export history to various formats (CSV, JSON)
- Statistics: min/max/average over time periods

### QoS Testing
**Priority: Low** | **Target: v0.7.0**

- Test multiple file sizes (10MB, 100MB, 1GB)
- Sustained speed test over longer duration (e.g., 60 seconds)
- Report whether ISP throttles large downloads
- Buffer bloat detection
- Peak vs sustained speed analysis

---

## 📊 Output & Reporting

### Advanced Output Formats
**Priority: High** | **Target: v0.5.0**

- JSON output for scripting: `--json`
- Machine-readable output: `--format csv`
- YAML output option
- Graph generation using ASCII art or image export
- Comparison mode: benchmark before/after network changes

### Integration Features
**Priority: Low** | **Target: v0.7.0**

- Webhook support to send results to monitoring tools
- Prometheus metrics export
- GitHub Actions integration for CI/CD network testing
- InfluxDB/Grafana integration
- MQTT publishing for IoT scenarios

---

## 🔧 Configuration & Customization

### Advanced Config Options
**Priority: Medium** | **Target: v0.5.0**

- Timeout settings (connection, read, total)
- Retry logic with exponential backoff
- Concurrent connections (like aria2)
- Proxy support (HTTP/SOCKS5)
- Custom headers per server
- Rate limiting options

### Profile Support
**Priority: Medium** | **Target: v0.6.0**

- Named profiles: `--profile home`, `--profile office`
- Different server preferences per profile
- Environment-specific configurations
- Quick switching between profiles
- Profile inheritance and defaults

---

## 🎨 UI/UX Improvements

### Better Interactive Mode
**Priority: Medium** | **Target: v0.5.0**

- Search/filter servers by location or name
- Favorite servers with quick access
- Recently used servers at top of menu
- Colorized server list by region/provider
- Multi-select for batch testing
- Server sorting options (alphabetical, by speed, by latency)

### Real-time Network Stats
**Priority: Low** | **Target: v0.7.0**

- Show active connections count
- Display TCP window size and congestion info
- MTU detection and recommendations
- Current throughput graph during download
- Connection statistics breakdown

### Notification Support
**Priority: Low** | **Target: v0.6.0**

- Desktop notifications when test completes
- Sound on completion (optional)
- Email/SMS alerts for monitoring scenarios
- Custom notification commands/scripts

---

## 🌍 Advanced Features

### CDN Performance Testing
**Priority: Low** | **Target: v0.8.0**

- Test multiple CDN providers simultaneously
- Regional performance comparison
- Cache hit/miss detection
- Edge location identification

### ISP Detection & Analysis
**Priority: Low** | **Target: v0.8.0**

- Detect user's ISP via API
- Compare speeds against ISP advertised rates
- Detect throttling or shaping patterns
- Time-of-day performance analysis
- ISP ranking in your area

### Scheduled Testing
**Priority: Medium** | **Target: v0.6.0**

- Cron-like scheduler built-in
- `--schedule "0 */6 * * *"` to run every 6 hours
- Automatic logging and alerting
- Daemon mode for continuous monitoring
- Email/webhook on significant speed changes

### Network Diagnostics
**Priority: Low** | **Target: v0.7.0**

- Traceroute to selected server
- MTU path discovery
- DNS resolution time tracking
- TLS handshake time breakdown
- Route performance analysis

### Smart Recommendations
**Priority: Low** | **Target: v0.8.0**

- "Your speed is 20% slower than usual"
- "Try server X for better performance"
- "Connection issues detected: high latency"
- Time-based recommendations (best time to download)
- Anomaly detection

---

## 🔒 Security & Privacy

### Privacy Mode
**Priority: Medium** | **Target: v0.5.0**

- No external API calls (no geolocation)
- No telemetry or analytics
- Verify server certificates strictly
- `--privacy` flag for full privacy mode
- Local-only operation option

### Custom CA Certificates
**Priority: Low** | **Target: v0.6.0**

- Support for corporate/custom CAs
- Certificate pinning for known servers
- Trust store configuration
- Certificate validation reporting

---

## 🛠️ Developer Features

### API Server Mode
**Priority: Low** | **Target: v0.8.0**

- `speedo --serve` to run as HTTP API
- REST endpoints for triggering tests
- WebSocket for real-time progress
- Authentication/authorization support
- Rate limiting for API endpoints

### Library/SDK
**Priority: Low** | **Target: v0.9.0**

- Publish as library crate for embedding
- Language bindings (Python, Node.js)
- Documented API for integration
- Examples and tutorials

### Plugin System
**Priority: Low** | **Target: v0.9.0**

- Custom server providers via plugins
- Custom output formatters
- Pre/post-test hooks
- Plugin marketplace/registry

---

## 📦 Distribution & Ecosystem

### Package Manager Integrations
**Priority: Medium** | **Target: Ongoing**

- ✅ Homebrew formula (started)
- ✅ cargo install (available)
- ✅ cargo-binstall (available)
- AUR package for Arch Linux
- apt/yum repositories
- Snap/Flatpak packages
- Windows installer with auto-updates
- Chocolatey package for Windows
- Scoop bucket for Windows

### Mobile Companion
**Priority: Low** | **Target: Future**

- Mobile app triggering tests on desktop
- Remote monitoring of network health
- Push notifications for monitoring alerts

### Web Dashboard
**Priority: Low** | **Target: Future**

- Optional web UI showing live tests
- Historical data visualization
- Multi-device aggregation
- Responsive design for mobile access

---

## Prioritized Short-term Roadmap

### v0.4.0 - Server Management (Q1 2025)
**Focus: Dynamic server infrastructure**

- Remote server list with auto-update
- Geolocation-based server selection
- `--auto` flag for closest server
- Server metadata (location, provider, reliability)

### v0.5.0 - Testing & Output Enhancements (Q2 2025)
**Focus: Better insights and flexibility**

- Multi-server testing (`--all`, `--parallel`)
- Historical tracking (SQLite database)
- JSON/CSV output formats
- Better interactive mode with search/favorites
- Privacy mode
- Advanced config options

### v0.6.0 - Advanced Testing Features (Q3 2025)
**Focus: Comprehensive network analysis**

- Upload speed testing
- Latency/jitter measurements
- Profile support
- Scheduled testing
- Notification support

### v0.7.0 - Network Diagnostics (Q4 2025)
**Focus: Deep network insights**

- QoS testing
- Network diagnostics (traceroute, MTU)
- Real-time network stats
- Integration features (webhooks, Prometheus)

### v0.8.0 - Intelligence & Ecosystem (2026)
**Focus: Smart features and broader reach**

- CDN performance testing
- ISP detection & analysis
- Smart recommendations
- API server mode

### v0.9.0+ - Platform & Extensions (Future)
**Focus: Expandability and integrations**

- Library/SDK
- Plugin system
- Enhanced distribution channels

---

## Community Contributions

We welcome contributions! Areas where community help would be valuable:

1. **Server contributions** - Add more test servers worldwide
2. **Translations** - Internationalization support
3. **Platform testing** - Verify functionality on different OS/architectures
4. **Documentation** - Examples, tutorials, use cases
5. **Feature implementations** - Pick any feature from this roadmap!

---

## Feedback & Suggestions

Have ideas for speedo? Please:
- Open an issue on GitHub
- Start a discussion in Discussions tab
- Submit a pull request

This roadmap is a living document and will be updated based on community feedback and priorities.

---

**Last Updated**: 2025-11-19  
**Current Version**: 0.3.0
