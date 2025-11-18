// Pre-configured speed test server definitions.
// Contains test file URLs from Cloudflare, Tele2, Hetzner, and Vultr.

pub struct TestServer {
    pub name: &'static str,
    pub url: &'static str,
}

pub const SERVERS: &[TestServer] = &[
    TestServer {
        name: "Cloudflare (CDN)",
        url: "https://speed.cloudflare.com/__down?bytes=100000000",
    },
    TestServer {
        name: "Tele2 (Global)",
        url: "http://speedtest.tele2.net/100MB.zip",
    },
    TestServer {
        name: "Hetzner (Nuremberg)",
        url: "https://nbg1-speed.hetzner.com/100MB.bin",
    },
    TestServer {
        name: "Hetzner (Falkenstein)",
        url: "https://fsn1-speed.hetzner.com/100MB.bin",
    },
    TestServer {
        name: "Hetzner (Helsinki)",
        url: "https://hel1-speed.hetzner.com/100MB.bin",
    },
    TestServer {
        name: "Hetzner (Ashburn VA)",
        url: "https://ash-speed.hetzner.com/100MB.bin",
    },
    TestServer {
        name: "Hetzner (Hillsboro OR)",
        url: "https://hil-speed.hetzner.com/100MB.bin",
    },
    TestServer {
        name: "Hetzner (Singapore)",
        url: "https://sin-speed.hetzner.com/100MB.bin",
    },
    TestServer {
        name: "Vultr (New Jersey)",
        url: "https://nj-us-ping.vultr.com/vultr.com.100MB.bin",
    },
    TestServer {
        name: "Vultr (Silicon Valley)",
        url: "https://sjo-ca-us-ping.vultr.com/vultr.com.100MB.bin",
    },
    TestServer {
        name: "Vultr (Singapore)",
        url: "https://sgp-ping.vultr.com/vultr.com.100MB.bin",
    },
];
