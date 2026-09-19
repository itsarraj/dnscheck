/// A public resolver's DNS-over-HTTPS JSON endpoint. All three speak
/// the same convenience JSON format (`?name=&type=` + `Accept:
/// application/dns-json`) — a de facto standard Google's DoH API
/// popularized, that Cloudflare and AdGuard both also implement,
/// letting one parser handle all three. Quad9 and OpenDNS were tried
/// and dropped from this list: both only speak RFC 8484 wire-format
/// DoH, not this JSON convenience API, confirmed live (Quad9's JSON
/// endpoint timed out; OpenDNS's returned 400).
pub struct Resolver {
    pub name: &'static str,
    pub url: &'static str,
}

pub const RESOLVERS: [Resolver; 3] = [
    Resolver {
        name: "Cloudflare",
        url: "https://cloudflare-dns.com/dns-query",
    },
    Resolver {
        name: "Google",
        url: "https://dns.google/resolve",
    },
    Resolver {
        name: "AdGuard",
        url: "https://dns.adguard-dns.com/resolve",
    },
];
