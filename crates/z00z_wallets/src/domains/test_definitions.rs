use super::*;
use z00z_crypto::expert::traits::DomainSeparation;

const DOMAINS_SNAPSHOT: &str = include_str!("../../docs/domains_snapshot.txt");

fn snapshot_lines_from_str(s: &str) -> Vec<String> {
    let mut lines: Vec<String> = s
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .filter(|line| !line.starts_with('#'))
        .map(ToString::to_string)
        .collect();
    lines.sort();
    lines
}

fn extract_first_string_literal(s: &str) -> Option<String> {
    let start = s.find('"')?;
    let rest = &s[start + 1..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn collect_domain_mappings_from_source() -> Vec<(String, String)> {
    let src = include_str!("definitions.rs");
    let mut mappings: Vec<(String, String)> = Vec::new();
    let lines: Vec<&str> = src.lines().collect();

    let mut i = 0usize;
    while i < lines.len() {
        let trimmed = lines[i].trim_start();
        if !trimmed.starts_with("hash_domain!(") {
            i += 1;
            continue;
        }

        let mut buf = String::from(trimmed);
        while !buf.contains(");") && i + 1 < lines.len() {
            i += 1;
            buf.push(' ');
            buf.push_str(lines[i].trim());
        }

        let invocation = buf.strip_prefix("hash_domain!(").unwrap_or("").trim();
        let invocation = invocation.strip_suffix(");").unwrap_or(invocation).trim();

        let Some((type_part, args_part)) = invocation.split_once(',') else {
            i += 1;
            continue;
        };

        let type_name = type_part.trim();
        if type_name.is_empty() {
            i += 1;
            continue;
        }

        let Some(domain) = extract_first_string_literal(args_part) else {
            i += 1;
            continue;
        };

        mappings.push((type_name.to_string(), domain));
        i += 1;
    }

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("impl ") || !trimmed.contains("DomainSeparation for") {
            continue;
        }

        let Some(after) = trimmed.split_once("DomainSeparation for") else {
            continue;
        };
        let type_name = after
            .1
            .trim_start()
            .split(|ch: char| ch.is_whitespace() || ch == '{')
            .next()
            .unwrap_or("")
            .trim();
        if type_name.is_empty() {
            continue;
        }

        let mut domain: Option<String> = None;
        let mut in_domain_fn = false;
        for &current_line in lines.iter().skip(i) {
            if !in_domain_fn {
                if current_line.contains("fn domain") {
                    in_domain_fn = true;
                }
                continue;
            }

            if let Some(lit) = extract_first_string_literal(current_line) {
                domain = Some(lit);
                break;
            }
        }

        if let Some(domain) = domain {
            mappings.push((type_name.to_string(), domain));
        }
    }

    mappings
}

fn canonical_domain_lines() -> Vec<String> {
    let mut mappings = collect_domain_mappings_from_source();
    mappings.sort_by(|(a, _), (b, _)| a.cmp(b));
    mappings
        .into_iter()
        .map(|(ty, domain)| format!("{ty}={domain}"))
        .collect()
}

#[test]
fn test_domain_strings_are_frozen() {
    let expected = snapshot_lines_from_str(DOMAINS_SNAPSHOT);
    let actual = canonical_domain_lines();

    assert_eq!(
        actual, expected,
        "Domain snapshot mismatch. Run the ignored test `test_print_domain_snapshot` and update `docs/domains_snapshot.txt` intentionally."
    );
}

#[test]
fn test_domain_strings_are_unique() {
    let mappings = collect_domain_mappings_from_source();
    assert!(
        mappings.len() >= 10,
        "Expected at least 10 domain mappings; source extraction looks broken"
    );

    let mut seen_types = std::collections::HashSet::new();
    for (ty, _) in &mappings {
        assert!(seen_types.insert(ty), "Duplicate domain type mapping: {ty}");
    }

    let mut seen_domains = std::collections::HashSet::new();
    for (_, domain) in &mappings {
        assert!(
            seen_domains.insert(domain),
            "Duplicate domain found: {domain}"
        );
    }
}

#[test]
fn test_receiver_cache_domains_stable() {
    assert_eq!(
        ReceiverCacheHmacTestDomain::domain(),
        "app/z00z_wallets/address/receiver_cache/test"
    );
    assert_eq!(
        ReceiverCacheHmacProdDomain::domain(),
        "app/z00z_wallets/address/receiver_cache/production"
    );
}

#[test]
fn test_pack_domains_frozen() {
    assert_eq!(
        PackKeyProdDomain::domain(),
        "z00z.wallet.stealth.pack_key.prod"
    );
    assert_eq!(
        PackNonceProdDomain::domain(),
        "z00z.wallet.stealth.pack_nonce.prod"
    );
    assert_eq!(
        WalletLeafAdHashProdDomain::domain(),
        "z00z.wallet.stealth.leaf_ad.prod"
    );
}

#[test]
#[ignore]
fn test_print_domain_snapshot() {
    let lines = canonical_domain_lines();
    assert!(!lines.is_empty());
}
