# speedo

Network speed test tool and file downloader. Built in rust, doesn't need curl, wget, or system ssl libs.

## Installation

### From crates.io

```bash
cargo install speedo
```

### Using cargo-binstall

```bash
cargo binstall speedo
```

### From source

```bash
cargo install --git https://github.com/coryzibell/speedo
```

## SYNOPSIS

```
speedo [URL]
speedo [-i|--interactive] [-n|--non-interactive] [-s|--speed-unit UNIT]
speedo [--json] [--format FORMAT] [--compact]
speedo --update-servers
speedo --help
speedo --version
```

## DESCRIPTION

speedo downloads a test file and reports the transfer speed. By default it runs non-interactively against Cloudflare's CDN. With the -i flag, it displays an enhanced interactive menu with 34+ speed test servers organized by region and provider.

The tool supports multiple output formats (human-readable, JSON, CSV) for scripting and automation, and automatically updates its server list from GitHub.

If a URL is provided as an argument, the file is downloaded to the current directory and the speed is reported.

Command-line flags override the config file settings.

## OPTIONS

**-i, --interactive**
    Show enhanced server selection menu with browse modes (by region, provider, search)

**-n, --non-interactive**
    Run quick test (override config)

**-s, --speed-unit UNIT**
    Speed unit format (bits-metric, bits-binary, bytes-metric, bytes-binary)

**--format FORMAT**
    Output format: json, csv, or human (default)

**--json**
    Output JSON format (shorthand for --format json)

**--compact**
    Use compact JSON output (no pretty printing)

**--update-servers**
    Update remote server list from GitHub

**-h, --help**
    Display help text

**-V, --version**
    Display version

## ARGUMENTS

**URL**
    URL to download (saves file to current directory)

## CONFIGURATION

Configuration is read from the first file found:
- ./speedo.toml
- ./.speedo.toml
- ~/.speedo.toml

### Example Configuration

```toml
# Default mode when no flags given (default: false)
interactive = false

# User agent string for requests
user_agent = "Mozilla/5.0"

# Speed unit format for progress display (default: "bytes-metric")
# Options:
#   "bits-metric" or "mbps"  - Megabits per second (Mbps, Gbps) - 1000-based
#   "bits-binary" or "mibps" - Mebibits per second (Mibps, Gibps) - 1024-based
#   "bytes-metric" or "mb/s" - Megabytes per second (MB/s, GB/s) - 1000-based (default)
#   "bytes-binary" or "mib/s" - Mebibytes per second (MiB/s, GiB/s) - 1024-based
speed_unit = "bytes-metric"

# Additional test servers
[[custom_servers]]
name = "My Server"
url = "https://example.com/testfile.bin"
```

See speedo.toml.example for details.

## EXAMPLES

Run a quick speed test (default server):
```
speedo
```

Download a specific file:
```
speedo https://example.com/testfile.zip
```

Show interactive menu with enhanced browse modes:
```
speedo -i
```

Update server list from GitHub:
```
speedo --update-servers
```

Force non-interactive mode (override config):
```
speedo -n
```

Use bits per second instead of bytes:
```
speedo --speed-unit bits-metric
```

Download with custom speed unit:
```
speedo -s bits-binary https://example.com/file.bin
```

Output results as JSON:
```
speedo --json
speedo -n --json --compact | jq '.results.speed.mbps'
```

Output results as CSV (for logging):
```
speedo -n --format csv >> speed_tests.csv
```

## OUTPUT

Non-interactive mode prints the transfer summary and speed:
```
Downloaded 95.37 MiB in 4.69s - 20.33 MB/s  (170.58 Mbps)
```

When downloading a URL, the saved filename is also printed:
```
Downloaded 95.37 MiB in 4.69s - 20.33 MB/s  (170.58 Mbps)
Saved: testfile.zip
```

Interactive mode displays a progress bar during download, then shows:
- Transfer summary (size and time)
- HTTP status code
- Connection time
- Time to first byte (TTFB)
- Total transfer time
- File size
- Transfer speed

### JSON Output

```bash
speedo --json
```

```json
{
  "timestamp": "2025-11-19T05:00:00Z",
  "server": {
    "name": "Cloudflare CDN",
    "url": "https://speed.cloudflare.com/__down?bytes=100000000"
  },
  "results": {
    "status_code": 200,
    "bytes_downloaded": 100000000,
    "total_time": 4.532,
    "connect_time": 0.123,
    "ttfb": 0.245,
    "speed": {
      "mbps": 176.42,
      "mb_s": 22.05
    }
  }
}
```

### CSV Output

```bash
speedo --format csv
```

```
timestamp,server_name,server_url,bytes_downloaded,total_time,connect_time,ttfb,speed_mbps,status_code
2025-11-19T05:00:00Z,Cloudflare CDN,https://speed.cloudflare.com/__down?bytes=100000000,100000000,4.532,0.123,0.245,176.42,200
```

### Speed Unit Configuration

You can configure the speed display format in speedo.toml:
- **bits-metric** - Mbps, Gbps (megabits, gigabits per second - 1000-based)
- **bits-binary** - Mibps, Gibps (mebibits, gibibits per second - 1024-based)
- **bytes-metric** - MB/s, GB/s (megabytes, gigabytes per second - 1000-based) - default
- **bytes-binary** - MiB/s, GiB/s (mebibytes, gibibytes per second - 1024-based)

## SERVERS

speedo includes 46+ pre-configured speed test servers across all major regions, automatically updated from GitHub:

**Global/CDN:**
- Cloudflare (Global CDN) - default
- Tele2 (Global)

**North America (18 servers):**
- Hetzner: Ashburn VA, Hillsboro OR
- Vultr: New Jersey, Atlanta, Chicago, Dallas, Seattle, Silicon Valley, Los Angeles
- Linode: Newark, Atlanta, Dallas, Fremont, Chicago, Seattle, Miami
- OVH: Canada, USA

**Europe (15 servers):**
- Hetzner: Nuremberg, Falkenstein, Helsinki
- Vultr: Amsterdam, Frankfurt, Paris, London, Warsaw, Madrid, Stockholm
- Linode: London, Frankfurt, Amsterdam, Madrid
- OVH: Europe (Multi-region)

**Asia (8 servers):**
- Hetzner: Singapore
- Vultr: Singapore, Bangalore
- Linode: Singapore, Tokyo, Osaka, Chennai, Jakarta

**Oceania (3 servers):**
- Vultr: Sydney
- Linode: Sydney
- OVH: Australia

### Interactive Browse Modes

When running `speedo -i`, you can browse servers by:
- **🌍 All servers** - View complete list
- **🗺️ Region** - Browse by continent/region
- **🏢 Provider** - Browse by hosting company
- **🔍 Search** - Filter by location, name, provider

### Server Updates

The server list is automatically updated from GitHub and cached locally for 7 days. Force an update with:
```bash
speedo --update-servers
```

### Why AWS, GCP, Azure Aren't Included

Major cloud platforms (AWS, Google Cloud, Microsoft Azure) don't provide public HTTP speed test files like infrastructure providers do:

- **Infrastructure providers** (Hetzner, Vultr, Linode) offer dedicated speed test servers with standardized 100MB files at known URLs to help customers choose data centers
- **Cloud platforms** (AWS, GCP, Azure) expect customers to deploy their own test infrastructure using their services

The included providers represent all major hosting companies that offer **public, unauthenticated HTTP speed test downloads**. Cloud platforms would require API authentication or custom resource deployment, which is outside the scope of a simple speed test tool.

## BUILDING

```
cargo build --release
```

## FILES

- speedo.toml - configuration file
- ~/.speedo.toml - user configuration file
- ~/.local/share/speedo/servers.json - cached server list (Linux)
- ~/Library/Application Support/speedo/servers.json - cached server list (macOS)

## SEE ALSO

curl(1), wget(1)

## LICENSE

This project is licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

