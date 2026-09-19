# dnscheck

Queries multiple public DNS-over-HTTPS resolvers at once and diffs
their answers — the CLI version of pasting a domain into
whatsmydns.net, for checking DNS propagation after a change without
leaving the terminal.

## Usage

```bash
dnscheck github.com
dnscheck github.com -t AAAA
dnscheck github.com -t MX
```

```
$ dnscheck github.com
Cloudflare : 20.207.73.82
Google     : 20.207.73.82
AdGuard    : 20.207.73.82

✓ all resolvers agree
```

Queries Cloudflare, Google, and AdGuard's public DoH JSON endpoints
concurrently. Exit code `0` if all resolvers agree and all were
reachable; `1` if they disagree (propagation still in progress) or if
any resolver couldn't be reached — scriptable in a deploy pipeline
("don't declare the DNS cutover done until this passes").

**Why these three and not Quad9 or OpenDNS**: Cloudflare, Google, and
AdGuard all implement the same convenience JSON API (`?name=&type=` +
`Accept: application/dns-json`) that this tool relies on for one
shared parser. Quad9 and OpenDNS were tried live and dropped: Quad9's
JSON endpoint timed out from this environment, OpenDNS's returned
HTTP 400 — both appear to only speak the binary RFC 8484 wire format,
not this JSON convenience layer.

## Status: built and verified against real, live public DNS — including a real parsing bug caught along the way

- **8 unit tests**: DoH JSON response parsing (a realistic Cloudflare-
  style response, a response with the `Answer` key omitted entirely,
  and — the one that mattered — AdGuard's actual `"Answer": null`
  shape) and the agreement-diffing logic (identical sets, differing
  sets, a lone resolver, zero resolvers, and an explicit note that
  comparison is order-sensitive so callers must sort first, which
  `query.rs` does).
- **Live-verified against real DNS, both the easy case and the edge
  cases**:
  - `github.com` (A): all three resolvers returned the same address,
    exit 0.
  - `cloudflare.com` (A, round-robin, 2 real addresses): all three
    returned the same *set* of two addresses in the same sorted order,
    proving the "sort before comparing" step actually neutralizes
    harmless answer-order differences rather than false-flagging them.
  - `github.com` (MX): all three agreed on the real mail exchanger
    record.
  - A genuinely nonexistent domain: all three correctly reported DNS
    status 3 (NXDOMAIN) instead of crashing or showing an empty
    answer indistinguishable from "record type not present."
- **A real bug this live-testing caught**: querying any record type
  with zero real answers (`AAAA` on a v4-only host, or the NXDOMAIN
  case above) made every AdGuard query fail with "unparseable
  response," while Cloudflare and Google succeeded on the identical
  query. Diffing the raw JSON bodies showed why: Cloudflare and Google
  simply *omit* the `Answer` key when there's nothing to answer, which
  `#[serde(default)]` handles — but AdGuard instead sends an explicit
  `"Answer": null`, which `#[serde(default)]` does **not** cover
  (that attribute only substitutes when a key is *missing*, not when
  it's present with a `null` value). Fixed by changing the field to
  `Option<Vec<AnswerRecord>>`; both zero-answer shapes are now covered
  by dedicated regression tests, and the fix was re-verified live
  against the exact two domains that had failed.

**Deliberate scope limits, stated plainly**: no DNSSEC validation
status reporting (the `AD` flag is parsed by none of this tool's
output — it's available in the API responses but not surfaced);
TXT-record comparison can show cosmetic quoting differences between
resolvers as a "disagreement" since raw `data` strings are compared
verbatim, not semantically normalized; only 3 resolvers, hardcoded —
no `--resolver` flag to add others in this v1.
