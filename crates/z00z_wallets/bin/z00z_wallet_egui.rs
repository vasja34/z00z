//! Minimal desktop launcher for the Z00Z Wallet egui UI.
//!
//! This binary is intentionally minimal: it boots `eframe` and hosts the YAML-driven
//! `MainView` built from the wallet flows directory.

#[cfg(feature = "egui")]
use std::{
    error::Error as StdError,
    io,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

#[cfg(feature = "egui")]
use eframe::egui;

#[cfg(feature = "egui")]
use uuid::Uuid;

#[cfg(feature = "egui")]
use z00z_utils::config::{ConfigSource, EnvConfig};

#[cfg(feature = "egui")]
use z00z_wallets::{rpc::types::wallet::WalletLifecycleEvent, services::WalletService};

#[cfg(feature = "egui")]
struct AppShell {
    main_view: z00z_wallets::egui_views::app_main_view::MainView,
    stop_after: Option<Duration>,
    started_at: Instant,
    is_close_sent: bool,
}

#[cfg(feature = "egui")]
impl AppShell {
    fn new(main_view: z00z_wallets::egui_views::app_main_view::MainView) -> Self {
        Self {
            main_view,
            stop_after: read_stop_timeout(),
            started_at: Instant::now(),
            is_close_sent: false,
        }
    }
}

#[cfg(feature = "egui")]
impl eframe::App for AppShell {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        eframe::App::update(&mut self.main_view, ctx, frame);

        let Some(stop_after) = self.stop_after else {
            return;
        };

        if self.is_close_sent {
            return;
        }

        let elapsed = self.started_at.elapsed();
        if elapsed >= stop_after {
            self.is_close_sent = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        ctx.request_repaint_after(stop_after.saturating_sub(elapsed));
    }
}

#[cfg(feature = "egui")]
fn read_stop_timeout() -> Option<Duration> {
    let raw = EnvConfig.get("Z00Z_EGUI_STOP_AFTER_SEC").ok().flatten()?;
    let secs = raw.trim().parse::<u64>().ok()?;
    Some(Duration::from_secs(secs))
}

#[cfg(feature = "egui")]
fn flows_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/flows")
}

#[cfg(feature = "egui")]
fn build_runtime() -> Option<Arc<tokio::runtime::Runtime>> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .ok()
        .map(Arc::new)
}

#[cfg(feature = "egui")]
fn wallet_data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("target")
        .join("z00z_wallet_egui")
        .join(Uuid::new_v4().to_string())
}

#[cfg(feature = "egui")]
fn build_service() -> Arc<WalletService> {
    Arc::new(WalletService::with_output_dir(wallet_data_dir()))
}

#[cfg(feature = "egui")]
fn native_options() -> eframe::NativeOptions {
    eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1263.0, 992.0])
            .with_min_inner_size([1263.0, 992.0]),
        ..Default::default()
    }
}

#[cfg(feature = "egui")]
fn build_app(
    runtime: &Option<Arc<tokio::runtime::Runtime>>,
    wallet_service: &Arc<WalletService>,
) -> Result<AppShell, Box<dyn StdError + Send + Sync>> {
    let mut app = z00z_wallets::egui_views::app_main_view::MainView::from_flows_dir(flows_dir())
        .map_err(|err| {
            let boxed: Box<dyn StdError + Send + Sync> =
                Box::new(io::Error::other(err.to_string()));
            boxed
        })?;

    if let Some(runtime) = runtime.as_ref() {
        let handle = runtime.handle().clone();
        let svc = Arc::clone(wallet_service);

        app.set_lifecycle_hook(Arc::new(move |event: WalletLifecycleEvent| {
            let svc = Arc::clone(&svc);
            handle.spawn(async move {
                let _ = svc.on_lifecycle_event(event).await;
            });
        }));
    }

    Ok(AppShell::new(app))
}

#[cfg(feature = "egui")]
fn smoke_headless_requested() -> bool {
    matches!(
        EnvConfig
            .get("Z00Z_EGUI_SMOKE_HEADLESS")
            .ok()
            .flatten()
            .as_deref(),
        Some("1") | Some("true") | Some("yes") | Some("on")
    )
}

#[cfg(feature = "egui")]
fn main() -> eframe::Result {
    let runtime = build_runtime();
    let wallet_service = build_service();

    if smoke_headless_requested() {
        build_app(&runtime, &wallet_service).map_err(eframe::Error::AppCreation)?;
        eprintln!("Z00Z egui headless smoke path enabled");
        return Ok(());
    }

    eframe::run_native(
        "Z00Z",
        native_options(),
        Box::new(move |_cc| {
            let app = build_app(&runtime, &wallet_service)?;
            Ok(Box::new(app))
        }),
    )
}

#[cfg(not(feature = "egui"))]
fn main() {
    eprintln!("This binary requires `--features egui`.");
}
