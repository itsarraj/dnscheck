use clap::Parser;

use dnscheck::diff::all_agree;
use dnscheck::query::query_resolver;
use dnscheck::resolvers::RESOLVERS;

#[derive(Parser)]
#[command(
    name = "dnscheck",
    about = "Queries multiple public DNS-over-HTTPS resolvers at once and diffs the answers"
)]
struct Cli {
    /// Domain name to look up.
    domain: String,

    /// DNS record type.
    #[arg(short = 't', long, default_value = "A")]
    record_type: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    let futures = RESOLVERS
        .iter()
        .map(|r| query_resolver(&client, r, &cli.domain, &cli.record_type));
    let results = futures::future::join_all(futures).await;

    let name_width = RESOLVERS.iter().map(|r| r.name.len()).max().unwrap_or(0);
    let mut had_error = false;
    let mut ok_record_sets: Vec<Vec<String>> = Vec::new();

    for result in &results {
        match result {
            Ok(qr) => {
                let shown = if qr.records.is_empty() {
                    "(no records)".to_string()
                } else {
                    qr.records.join(", ")
                };
                println!("{:width$} : {shown}", qr.resolver_name, width = name_width);
                ok_record_sets.push(qr.records.clone());
            }
            Err(e) => {
                println!("{:width$} : ERROR — {e}", "", width = name_width);
                had_error = true;
            }
        }
    }

    println!();
    let consistent = all_agree(ok_record_sets.iter().map(|v| v.as_slice()));
    if !consistent {
        println!("✗ resolvers disagree — DNS propagation may still be in progress");
    } else if had_error {
        println!("⚠ some resolvers could not be reached — see errors above");
    } else {
        println!("✓ all resolvers agree");
    }

    if !consistent || had_error {
        std::process::exit(1);
    }
    Ok(())
}
