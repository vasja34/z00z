use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{Arc, Mutex, OnceLock},
};

use crate::output_roots;
use roxmltree::Document;
use z00z_core::assets::AssetWire;
use z00z_crypto::expert::encoding::SafePassword;
use z00z_simulator::scenario_1::stage_6::shared_cases;
use z00z_simulator::StageResult;
use z00z_utils::io::{create_dir_all, load_json, read_file, write_file};
use z00z_wallets::{
    domains::hashing::compute_wallet_file_id,
    rpc::types::{common::PersistWalletId, wallet::WalletSource},
    services::WalletService,
};
use zip::ZipArchive;

struct RunCase {
    out: PathBuf,
    stage4: StageResult,
}

static E2E18_RUN: OnceLock<RunCase> = OnceLock::new();

const ALICE_PASS: &str = "Alice_Pass_Z00Z_42!";

fn out_dir() -> PathBuf {
    output_roots::stage4_output_root()
}

fn persist_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn lock_persist() -> std::sync::MutexGuard<'static, ()> {
    persist_lock().lock().unwrap_or_else(|err| err.into_inner())
}

fn run_case() -> &'static RunCase {
    E2E18_RUN.get_or_init(|| RunCase {
        out: shared_cases::e2e18_stage6_out(),
        stage4: StageResult::Ok,
    })
}

fn load_json_value(path: &std::path::Path) -> serde_json::Value {
    load_json(path).unwrap_or_else(|_| panic!("load json value: {}", path.display()))
}

fn find_wallet<'a>(dump: &'a serde_json::Value, actor: &str) -> &'a serde_json::Value {
    dump["wallets"]
        .as_array()
        .expect("wallets array")
        .iter()
        .find(|row| row["actor"].as_str() == Some(actor))
        .unwrap_or_else(|| panic!("wallet row missing for actor={actor}"))
}

fn item_keys(wallet: &serde_json::Value) -> Vec<(String, u32, u64)> {
    let mut keys = wallet["items"]
        .as_array()
        .expect("wallet items")
        .iter()
        .map(|row| {
            (
                row["asset_id_hex"]
                    .as_str()
                    .expect("item asset_id_hex")
                    .to_string(),
                row["serial_id"].as_u64().expect("item serial_id") as u32,
                row["amount"].as_u64().expect("item amount"),
            )
        })
        .collect::<Vec<_>>();
    keys.sort();
    keys
}

fn persist_files(tx_dir: &std::path::Path) -> [PathBuf; 7] {
    let files = [
        tx_dir.join("wallets_state_before.json"),
        tx_dir.join("wallets_state_after.json"),
        tx_dir.join("wallets_state_diff.json"),
        tx_dir.join("wallets_pending.json"),
        tx_dir.join("wallets_confirmed.json"),
        tx_dir.join("wallets_state_report.md"),
        tx_dir.join("wallets_state_report.xlsx"),
    ];

    for path in &files {
        assert!(
            path.exists(),
            "stage-4 artifact missing: {}",
            path.display()
        );
    }

    files
}

fn wallet_path(out: &std::path::Path, wallet_id: &str) -> PathBuf {
    let file_id = compute_wallet_file_id(wallet_id);
    let file_hex = z00z_crypto::expert::encoding::to_hex(&file_id[..8]);
    out.join("wallets").join(format!("wallet_{file_hex}.wlt"))
}

fn check_persist_body(
    before: &serde_json::Value,
    after: &serde_json::Value,
    diff: &serde_json::Value,
    pending: &serde_json::Value,
    confirm: &serde_json::Value,
    md_text: &str,
    xlsx_size: u64,
) {
    assert!(!before["wallets"]
        .as_array()
        .expect("before wallets")
        .is_empty());
    assert!(!after["wallets"]
        .as_array()
        .expect("after wallets")
        .is_empty());
    assert!(!diff["rows"].as_array().expect("diff rows").is_empty());
    assert!(!pending.as_array().expect("pending rows").is_empty());
    assert!(!confirm.as_array().expect("confirm rows").is_empty());
    assert!(md_text.contains("wallet_id") && md_text.contains("lifecycle_status"));
    assert!(xlsx_size > 0, "xlsx report must not be empty");
}

fn zip_text(zip: &mut ZipArchive<std::fs::File>, name: &str) -> String {
    let mut file = zip
        .by_name(name)
        .unwrap_or_else(|_| panic!("zip entry: {name}"));
    let mut text = String::new();
    use std::io::Read as _;
    file.read_to_string(&mut text)
        .unwrap_or_else(|_| panic!("zip utf8: {name}"));
    text
}

fn shared_vals(doc: &Document<'_>) -> Vec<String> {
    doc.descendants()
        .filter(|node| {
            node.has_tag_name((
                doc.root_element().tag_name().namespace().unwrap_or(""),
                "si",
            ))
        })
        .map(|node| {
            node.descendants()
                .filter(|item| {
                    item.has_tag_name((
                        doc.root_element().tag_name().namespace().unwrap_or(""),
                        "t",
                    ))
                })
                .filter_map(|item| item.text())
                .collect::<String>()
        })
        .collect()
}

fn sheet_map(doc: &Document<'_>) -> Vec<(String, String)> {
    doc.descendants()
        .filter(|node| node.tag_name().name() == "sheet")
        .map(|node| {
            let name = node.attribute("name").expect("sheet name").to_string();
            let rel = node
                .attribute((
                    "http://schemas.openxmlformats.org/officeDocument/2006/relationships",
                    "id",
                ))
                .expect("sheet rel")
                .to_string();
            (name, rel)
        })
        .collect()
}

fn rel_map(doc: &Document<'_>) -> BTreeMap<String, String> {
    doc.descendants()
        .filter(|node| node.tag_name().name() == "Relationship")
        .map(|node| {
            (
                node.attribute("Id").expect("rel id").to_string(),
                node.attribute("Target").expect("rel target").to_string(),
            )
        })
        .collect()
}

fn col_num(cell_ref: &str) -> u32 {
    cell_ref
        .chars()
        .take_while(|ch| ch.is_ascii_alphabetic())
        .fold(0u32, |acc, ch| {
            acc * 26 + (u32::from(ch.to_ascii_uppercase() as u8) - u32::from(b'A') + 1)
        })
}

fn sheet_rows(doc: &Document<'_>, shared: &[String]) -> Vec<BTreeMap<u32, String>> {
    doc.descendants()
        .filter(|node| node.tag_name().name() == "row")
        .map(|row| {
            row.children()
                .filter(|node| node.tag_name().name() == "c")
                .map(|cell| {
                    let cell_ref = cell.attribute("r").expect("cell ref");
                    let cell_col = col_num(cell_ref);
                    let cell_type = cell.attribute("t").unwrap_or("");
                    let cell_val = if cell_type == "s" {
                        let idx = cell
                            .descendants()
                            .find(|node| node.tag_name().name() == "v")
                            .and_then(|node| node.text())
                            .expect("shared idx")
                            .parse::<usize>()
                            .expect("shared idx parse");
                        shared[idx].clone()
                    } else if cell_type == "inlineStr" {
                        cell.descendants()
                            .filter(|node| node.tag_name().name() == "t")
                            .filter_map(|node| node.text())
                            .collect::<String>()
                    } else {
                        cell.descendants()
                            .find(|node| node.tag_name().name() == "v")
                            .and_then(|node| node.text())
                            .unwrap_or("")
                            .to_string()
                    };
                    (cell_col, cell_val)
                })
                .collect::<BTreeMap<_, _>>()
        })
        .collect()
}

fn load_xlsx(path: &std::path::Path) -> BTreeMap<String, Vec<BTreeMap<u32, String>>> {
    let file =
        std::fs::File::open(path).unwrap_or_else(|_| panic!("open xlsx: {}", path.display()));
    let mut zip = ZipArchive::new(file).expect("xlsx zip");

    let shared_xml = zip_text(&mut zip, "xl/sharedStrings.xml");
    let shared_doc = Document::parse(&shared_xml).expect("shared xml");
    let shared = shared_vals(&shared_doc);

    let book_xml = zip_text(&mut zip, "xl/workbook.xml");
    let book_doc = Document::parse(&book_xml).expect("workbook xml");
    let sheets = sheet_map(&book_doc);

    let rels_xml = zip_text(&mut zip, "xl/_rels/workbook.xml.rels");
    let rels_doc = Document::parse(&rels_xml).expect("rels xml");
    let rels = rel_map(&rels_doc);

    let mut out = BTreeMap::new();
    for (name, rel) in sheets {
        let target = rels
            .get(&rel)
            .unwrap_or_else(|| panic!("sheet target: {name}"));
        let sheet_xml = zip_text(&mut zip, &format!("xl/{target}"));
        let sheet_doc = Document::parse(&sheet_xml).expect("sheet xml");
        out.insert(name, sheet_rows(&sheet_doc, &shared));
    }
    out
}

fn cell_text(row: &BTreeMap<u32, String>, col: u32) -> String {
    row.get(&col).cloned().unwrap_or_default()
}

fn cell_u64(row: &BTreeMap<u32, String>, col: u32) -> u64 {
    let raw = cell_text(row, col);
    if raw.is_empty() {
        return 0;
    }
    raw.parse::<f64>().expect("numeric cell") as u64
}

fn check_xlsx_body(
    xlsx_path: &std::path::Path,
    before: &serde_json::Value,
    after: &serde_json::Value,
    diff: &serde_json::Value,
    pending: &serde_json::Value,
    confirm: &serde_json::Value,
    selected: &[serde_json::Value],
) {
    let book = load_xlsx(xlsx_path);
    let sheet_names: Vec<&str> = book.keys().map(String::as_str).collect();
    assert_eq!(
        sheet_names,
        vec![
            "confirmed",
            "diff",
            "pending",
            "selected_inputs",
            "serial_after",
            "summary",
            "tx_economics",
        ]
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>()
    );

    let row_count = |name: &str| book.get(name).expect("sheet").len().saturating_sub(1);
    assert_eq!(
        row_count("pending"),
        pending.as_array().expect("pending rows").len()
    );
    assert_eq!(
        row_count("confirmed"),
        confirm.as_array().expect("confirm rows").len()
    );
    assert_eq!(row_count("selected_inputs"), selected.len());
    assert_eq!(
        row_count("diff"),
        diff["rows"].as_array().expect("diff rows").len()
    );
    assert_eq!(
        row_count("serial_after"),
        after["wallets"]
            .as_array()
            .expect("after wallets")
            .iter()
            .map(|row| row["serial_dist"].as_array().expect("serial_dist").len())
            .sum::<usize>()
    );

    let pending_rows = book.get("pending").expect("pending sheet");
    let pending_sum: u64 = pending_rows
        .iter()
        .skip(1)
        .map(|row| cell_u64(row, 6))
        .sum();
    let pending_json = pending.as_array().expect("pending rows");
    assert_eq!(
        pending_sum,
        pending_json
            .iter()
            .map(|row| row["amount"].as_u64().expect("pending amount"))
            .sum::<u64>()
    );
    let pending_serials: Vec<u32> = pending_rows
        .iter()
        .skip(1)
        .map(|row| cell_u64(row, 4) as u32)
        .collect();
    let json_serials: Vec<u32> = pending_json
        .iter()
        .map(|row| row["serial_id"].as_u64().expect("pending serial") as u32)
        .collect();
    assert_eq!(pending_serials, json_serials);

    let confirm_rows = book.get("confirmed").expect("confirm sheet");
    let confirm_sum: u64 = confirm_rows
        .iter()
        .skip(1)
        .map(|row| cell_u64(row, 6))
        .sum();
    assert_eq!(
        confirm_sum,
        confirm
            .as_array()
            .expect("confirm rows")
            .iter()
            .map(|row| row["amount"].as_u64().expect("confirm amount"))
            .sum::<u64>()
    );

    let mut xlsx_serial = BTreeMap::new();
    for row in book
        .get("serial_after")
        .expect("serial_after sheet")
        .iter()
        .skip(1)
    {
        xlsx_serial.insert(
            (cell_text(row, 1), cell_u64(row, 2) as u32),
            (cell_u64(row, 3) as usize, cell_u64(row, 4)),
        );
    }
    let mut json_serial = BTreeMap::new();
    for wallet in after["wallets"].as_array().expect("after wallets") {
        for row in wallet["serial_dist"].as_array().expect("serial_dist") {
            json_serial.insert(
                (
                    wallet["actor"].as_str().expect("after actor").to_string(),
                    row["serial_id"].as_u64().expect("serial_id") as u32,
                ),
                (
                    row["row_count"].as_u64().expect("row_count") as usize,
                    row["total_amount"].as_u64().expect("total_amount"),
                ),
            );
        }
    }
    assert_eq!(xlsx_serial, json_serial);

    let mut xlsx_sum = BTreeMap::new();
    for row in book.get("summary").expect("summary sheet").iter().skip(1) {
        xlsx_sum.insert(
            cell_text(row, 2),
            (
                cell_text(row, 3),
                cell_u64(row, 4),
                cell_u64(row, 5),
                cell_u64(row, 6),
                cell_u64(row, 7),
                cell_u64(row, 8),
                cell_u64(row, 9),
            ),
        );
    }
    for before_wallet in before["wallets"].as_array().expect("before wallets") {
        let wallet_id = before_wallet["wallet_id"].as_str().expect("wallet_id");
        let after_wallet = after["wallets"]
            .as_array()
            .expect("after wallets")
            .iter()
            .find(|row| row["wallet_id"].as_str() == Some(wallet_id))
            .expect("matching wallet");
        let got = xlsx_sum.get(wallet_id).expect("summary wallet");
        let exp = (
            if after_wallet["wlt_exists"].as_bool().expect("wlt_exists") {
                "yes".to_string()
            } else {
                "no".to_string()
            },
            before_wallet["wlt_size_bytes"]
                .as_u64()
                .expect("before size"),
            after_wallet["wlt_size_bytes"].as_u64().expect("after size"),
            before_wallet["item_count"].as_u64().expect("before items"),
            after_wallet["item_count"].as_u64().expect("after items"),
            before_wallet["serial_dist"]
                .as_array()
                .expect("before serial_dist")
                .len() as u64,
            after_wallet["serial_dist"]
                .as_array()
                .expect("after serial_dist")
                .len() as u64,
        );
        assert_eq!(got, &exp);
    }
}

fn check_alice_rows(
    out: &std::path::Path,
    before: &serde_json::Value,
    after: &serde_json::Value,
    diff: &serde_json::Value,
    pending: &serde_json::Value,
    confirm: &serde_json::Value,
) -> (String, PathBuf) {
    let alice_before = find_wallet(before, "alice");
    let alice_after = find_wallet(after, "alice");
    let alice_id = alice_after["wallet_id"].as_str().expect("alice wallet_id");

    assert!(alice_before["wlt_exists"].as_bool().unwrap_or(false));
    assert!(alice_after["wlt_exists"].as_bool().unwrap_or(false));
    assert!(alice_after["wlt_size_bytes"].as_u64().unwrap_or(0) > 0);
    assert!(diff["rows"]
        .as_array()
        .expect("diff rows array")
        .iter()
        .any(|row| {
            row["actor"].as_str() == Some("alice")
                && row["lifecycle_status"].as_str().unwrap_or("none") != "none"
        }));
    assert!(pending
        .as_array()
        .expect("pending rows array")
        .iter()
        .any(|row| {
            row["actor"].as_str() == Some("bob")
                && row["lifecycle_status"].as_str() == Some("pending_receive")
        }));
    assert!(confirm
        .as_array()
        .expect("confirm rows array")
        .iter()
        .any(|row| {
            row["actor"].as_str() == Some("bob")
                && row["lifecycle_status"].as_str() == Some("confirmed_receive")
        }));

    (alice_id.to_string(), wallet_path(out, alice_id))
}

fn reopen_rows(alice_id: String, alice_wlt: PathBuf) -> Vec<(String, u32, u64)> {
    let mut reopened = tokio::runtime::Runtime::new()
        .expect("tokio runtime")
        .block_on(async {
            let wallets_dir = alice_wlt.parent().expect("alice wallet dir").to_path_buf();
            let wallet_id = PersistWalletId(alice_id);
            let service = Arc::new(WalletService::with_output_dir(wallets_dir));

            service
                .open_wallet_source(WalletSource::Path {
                    path: alice_wlt.to_string_lossy().to_string(),
                })
                .await
                .expect("open alice wallet source");
            service
                .unlock_wallet_in_memory(&wallet_id, &SafePassword::from(ALICE_PASS))
                .await
                .expect("unlock alice after reopen");
            let rows = service
                .list_claimed_assets(&wallet_id)
                .await
                .expect("list claimed assets after reopen");
            service
                .lock_wallet(&wallet_id)
                .await
                .expect("lock alice after reopen");

            rows.into_iter()
                .map(|asset| {
                    let wire = AssetWire::from_asset(&asset);
                    (hex::encode(asset.asset_id()), wire.serial_id, wire.amount)
                })
                .collect::<Vec<_>>()
        });
    reopened.sort();
    reopened
}

fn write_persist_log(
    files: &[PathBuf; 7],
    pending: &serde_json::Value,
    confirm: &serde_json::Value,
    alice_id: &str,
    reopened_len: usize,
) {
    create_dir_all(out_dir()).expect("mkdir outputs/e2e18");
    let mut persist_log = String::from("E2E-18 wallet persistence\n");
    persist_log.push_str(&format!("before_file={}\n", files[0].display()));
    persist_log.push_str(&format!("after_file={}\n", files[1].display()));
    persist_log.push_str(&format!("diff_file={}\n", files[2].display()));
    persist_log.push_str(&format!(
        "pending_rows={}\n",
        pending.as_array().expect("pending").len()
    ));
    persist_log.push_str(&format!(
        "confirmed_rows={}\n",
        confirm.as_array().expect("confirm").len()
    ));
    persist_log.push_str(&format!("alice_wallet_id={}\n", alice_id));
    persist_log.push_str(&format!("reopened_assets={}\n", reopened_len));
    write_file(
        out_dir().join("wallet_persist_log.txt"),
        persist_log.as_bytes(),
    )
    .expect("write persist log");
}

#[test]
fn test_stage4_wallet_persistence() {
    if cfg!(debug_assertions) {
        return;
    }

    let _guard = lock_persist();
    let run = run_case();
    assert!(
        matches!(run.stage4, StageResult::Ok),
        "stage 4 must succeed"
    );

    let tx_dir = run.out.join("transactions");
    let files = persist_files(&tx_dir);
    let before = load_json_value(&files[0]);
    let after = load_json_value(&files[1]);
    let diff = load_json_value(&files[2]);
    let pending = load_json_value(&files[3]);
    let confirm = load_json_value(&files[4]);
    let selected: Vec<serde_json::Value> =
        load_json(tx_dir.join("wallets_selected_inputs.json")).expect("selected inputs");
    let md_text =
        String::from_utf8(read_file(&files[5]).expect("read md report")).expect("md report utf8");
    let xlsx_size = std::fs::metadata(&files[6]).expect("xlsx meta").len();

    check_persist_body(
        &before, &after, &diff, &pending, &confirm, &md_text, xlsx_size,
    );
    check_xlsx_body(
        &files[6], &before, &after, &diff, &pending, &confirm, &selected,
    );
    let alice_after = item_keys(find_wallet(&after, "alice"));
    let (alice_id, alice_wlt) =
        check_alice_rows(&run.out, &before, &after, &diff, &pending, &confirm);
    let reopened = reopen_rows(alice_id.clone(), alice_wlt);
    let stage13_report_path = tx_dir.join("wallet_tx_rpc_lifecycle.json");
    if stage13_report_path.exists() {
        let stage13 = load_json_value(&stage13_report_path);
        assert_eq!(
            alice_after.len(),
            stage13["sender_claimed_before"]
                .as_u64()
                .expect("stage13 sender_claimed_before") as usize
        );
        assert_eq!(
            reopened.len(),
            stage13["sender_claimed_after"]
                .as_u64()
                .expect("stage13 sender_claimed_after") as usize
        );
    } else {
        assert_eq!(reopened, alice_after);
    }
    write_persist_log(&files, &pending, &confirm, &alice_id, reopened.len());
}
