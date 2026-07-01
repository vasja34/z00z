//! Main application window view (egui/eframe).
//!
//! This is the primary desktop window for the Z00Z wallet application.
//! It provides the complete user interface including:
//! - Header with wallet information and action buttons
//! - Sidebar with wallet list, network selection, and navigation buttons
//! - Main content area with tabbed interface
//! - Status bar showing balance and sync progress
//!
//! The view is driven by `flows/main.yaml` (entry points + UI layout hints),
//! but uses the existing `StateMachine` rendering API for the content area.

use crate::{WalletError, WalletResult};
use std::path::Path;
use std::sync::Arc;
use z00z_utils::codec::{Codec, YamlCodec};
use z00z_utils::config::YamlValue;
use z00z_utils::io::read_to_string;

// TEMPORARY: state_views module removed - will be replaced with RPC-based rendering
// use crate::egui_views::state_views;
use eframe::egui;

#[cfg(not(target_arch = "wasm32"))]
use crate::rpc::types::wallet::WalletLifecycleEvent;

include!("ui_config.rs");
include!("ui_state_machine.rs");
include!("ui_theme.rs");
include!("tab_registry.rs");
include!("main_view_loaders.rs");
include!("main_view.rs");
