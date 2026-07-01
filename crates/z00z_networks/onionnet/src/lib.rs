#![doc = include_str!("../README.md")]
//! OnionNet placeholder crate.
//!
//! This crate reserves the Phase 115 OnionNet module boundary now so later
//! implementation can land without moving the namespace or rebuilding the
//! network design around a different crate shape.
//! OnionNet is a node-owned privacy overlay, not an RPC transport alias and not
//! a separate application service.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

/// OnionNet settings and validation surfaces.
pub mod config {
    /// Placeholder configuration root for future OnionNet settings.
    #[derive(Debug, Clone, Default, PartialEq, Eq)]
    pub struct OnionNetConfig;
}

/// Node transport identity and routing-key ownership.
pub mod identity {
    /// Placeholder node-owned transport identity type.
    #[derive(Debug, Clone, Default, PartialEq, Eq)]
    pub struct NodeTransportIdentity;
}

/// Bootstrap descriptors and manifest refresh boundaries.
pub mod bootstrap {
    /// Placeholder bootstrap manifest contract.
    #[derive(Debug, Clone, Default, PartialEq, Eq)]
    pub struct BootstrapManifest;
}

/// QUIC transport seams for node-to-node carriage.
pub mod transport_quic {
    /// Placeholder QUIC transport configuration.
    #[derive(Debug, Clone, Default, PartialEq, Eq)]
    pub struct QuinnTransportConfig;
}

/// Link-layer cryptographic framing reused from `z00z_crypto`.
pub mod link_crypto {
    /// Placeholder link crypto profile.
    #[derive(Debug, Clone, Default, PartialEq, Eq)]
    pub struct LinkCryptoProfile;
}

/// Fixed-geometry packet classes and framing.
pub mod packet {
    /// OnionNet traffic classes reserved by the Phase 115 packet model.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum PacketClass {
        /// Canonical user payload transport.
        Data,
        /// Indistinguishable padding traffic.
        Cover,
        /// Health and packet-loss probes.
        Loop,
        /// Control-plane traffic such as descriptors or setup.
        Control,
    }
}

/// Sphinx-path build, unwrap, and replay-tag processing seams.
pub mod sphinx_path {
    /// Placeholder Sphinx path policy root.
    #[derive(Debug, Clone, Default, PartialEq, Eq)]
    pub struct SphinxPathPolicy;
}

/// Short-lived ingress sessions and replay windows.
pub mod session {
    /// Placeholder session window policy.
    #[derive(Debug, Clone, Default, PartialEq, Eq)]
    pub struct SessionWindowPolicy;
}

/// Public ingress adapters such as Tor, WebSocket, or bridge entrypoints.
pub mod bridge_api {
    /// Placeholder bridge ingress adapter trait.
    pub trait BridgeIngressAdapter {}
}

/// Edge admission and first-hop privacy boundary.
pub mod edge {
    /// Placeholder edge admission policy.
    #[derive(Debug, Clone, Default, PartialEq, Eq)]
    pub struct EdgeAdmissionPolicy;
}

/// Relay forwarding responsibilities.
pub mod relay {
    /// Placeholder relay forwarding policy.
    #[derive(Debug, Clone, Default, PartialEq, Eq)]
    pub struct RelayForwardingPolicy;
}

/// Exit normalization and canonical runtime handoff seams.
pub mod exit {
    /// Placeholder exit handoff trait.
    pub trait ExitHandoff<Payload> {
        /// Hand off a canonical payload to the downstream ingress boundary.
        fn handoff(&self, payload: Payload);
    }
}

/// Metrics, tracing, and health surfaces.
pub mod telemetry {
    /// Placeholder telemetry surface root.
    #[derive(Debug, Clone, Default, PartialEq, Eq)]
    pub struct TelemetrySurface;
}

pub use bootstrap::BootstrapManifest;
pub use bridge_api::BridgeIngressAdapter;
pub use config::OnionNetConfig;
pub use edge::EdgeAdmissionPolicy;
pub use exit::ExitHandoff;
pub use identity::NodeTransportIdentity;
pub use link_crypto::LinkCryptoProfile;
pub use packet::PacketClass;
pub use relay::RelayForwardingPolicy;
pub use session::SessionWindowPolicy;
pub use sphinx_path::SphinxPathPolicy;
pub use telemetry::TelemetrySurface;
pub use transport_quic::QuinnTransportConfig;
