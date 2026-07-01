#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

mod alerts;
mod censorship;
mod da_health;
mod engine;
mod evidence_export;
mod provider;
mod publication;
mod status;

pub use alerts::{AlertKind, AlertSeverity, AlertSubject, WatcherAlert};
pub use censorship::CensorshipWatch;
pub use da_health::{ProviderOutcome, ProviderSignal, ProviderStage};
pub use engine::{WatcherBoundary, WatcherInput, WatcherService};
pub use evidence_export::{EvidenceKey, EvidenceRecord};
pub use provider::ProviderCompare;
pub use publication::{PublicationWatch, PublicationWatchErr};
pub use status::{AlertCounts, ObservationSnapshot};
