use serde::Deserialize;

use crate::resolvers::Resolver;

#[derive(Deserialize)]
struct DohResponse {
    #[serde(rename = "Status")]
    status: i32,
    // `Option`, not a bare `Vec` — Cloudflare/Google omit the `Answer`
    // key entirely when there's nothing to answer (which `#[serde(default)]`
    // alone handles), but AdGuard instead sends an explicit JSON
    // `"Answer": null`, which `#[serde(default)]` does *not* cover:
    // that attribute only substitutes a default for a *missing* key,
    // not for a key present with a `null` value. Confirmed live by
    // diffing AdGuard's raw response for a zero-answer query against
    // Cloudflare's and Google's.
    #[serde(rename = "Answer", default)]
    answer: Option<Vec<AnswerRecord>>,
}

#[derive(Deserialize)]
struct AnswerRecord {
    data: String,
}

pub struct QueryResult {
    pub resolver_name: &'static str,
    /// Sorted and deduped so two resolvers returning the same record
    /// set in a different order (a normal, meaningless difference in
    /// round-robin DNS) don't get reported as a mismatch.
    pub records: Vec<String>,
}

/// Queries one resolver's DoH JSON endpoint for `name`'s `record_type`
/// records. A non-zero DNS `Status` (SERVFAIL, NXDOMAIN, etc.) is a
/// legitimate, distinct *answer* ("this name doesn't resolve here"),
/// not a transport error — it surfaces as `Ok` with a one-line
/// placeholder record noting the status code, not as `Err`. `Err` is
/// reserved for actually failing to reach or parse the resolver.
pub async fn query_resolver(
    client: &reqwest::Client,
    resolver: &Resolver,
    name: &str,
    record_type: &str,
) -> Result<QueryResult, String> {
    let response = client
        .get(resolver.url)
        .query(&[("name", name), ("type", record_type)])
        .header("accept", "application/dns-json")
        .send()
        .await
        .map_err(|e| format!("{}: request failed: {e}", resolver.name))?;

    if !response.status().is_success() {
        return Err(format!(
            "{}: HTTP {}",
            resolver.name,
            response.status().as_u16()
        ));
    }

    let parsed: DohResponse = response
        .json()
        .await
        .map_err(|e| format!("{}: unparseable response: {e}", resolver.name))?;

    let mut records: Vec<String> = parsed
        .answer
        .unwrap_or_default()
        .into_iter()
        .map(|a| a.data)
        .collect();
    records.sort();
    records.dedup();

    if parsed.status != 0 && records.is_empty() {
        return Ok(QueryResult {
            resolver_name: resolver.name,
            records: vec![format!("(no answer — DNS status code {})", parsed.status)],
        });
    }

    Ok(QueryResult {
        resolver_name: resolver.name,
        records,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_realistic_cloudflare_style_response() {
        let json = r#"{"Status":0,"TC":false,"RD":true,"RA":true,"AD":false,"CD":false,
            "Question":[{"name":"github.com","type":1}],
            "Answer":[{"name":"github.com","type":1,"TTL":46,"data":"20.207.73.82"}]}"#;
        let parsed: DohResponse = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.status, 0);
        let answer = parsed.answer.unwrap();
        assert_eq!(answer.len(), 1);
        assert_eq!(answer[0].data, "20.207.73.82");
    }

    #[test]
    fn parses_a_response_with_no_answer_key_at_all() {
        let json = r#"{"Status":3,"Question":[{"name":"nope.invalid","type":1}]}"#;
        let parsed: DohResponse = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.status, 3);
        assert!(parsed.answer.unwrap_or_default().is_empty());
    }

    #[test]
    fn parses_adguards_explicit_null_answer_without_erroring() {
        // Real, live-observed AdGuard response shape: unlike Cloudflare
        // and Google, which omit the key, AdGuard sends `"Answer":
        // null` outright — this must not fail to deserialize.
        let json = r#"{"Status":0,"Answer":null,"Question":[{"name":"github.com.","type":28}]}"#;
        let parsed: DohResponse = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.status, 0);
        assert!(parsed.answer.unwrap_or_default().is_empty());
    }
}
