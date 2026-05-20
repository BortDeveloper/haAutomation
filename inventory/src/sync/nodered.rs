//! Node-RED flow synchronization (read-only ingestion).
//!
//! This module pulls flow snapshots from a Node-RED Admin API
//! for analysis ingestion. It does NOT mutate flows directly —
//! flow mutations are applied via a separate apply-proposal
//! path that consumes structured proposals from the stack-master
//! cockpit (see ADR-0011, docs/nodered-integration.md).

use anyhow::Result;

/// Pull a read-only snapshot of Node-RED flows for analysis.
///
/// Returns the raw `/flows` JSON from the Node-RED Admin API.
/// Authentication (adminAuth) is REQUIRED — unauthenticated
/// access is rejected with a clear error message.
pub fn pull_flows_snapshot(_base_url: &str, _admin_token: &str) -> Result<String> {
    unimplemented!(
        "NR-Integration WIP — branch feat/nodered-integration. \
         Not callable from main. See docs/nodered-integration.md."
    )
}

/// Apply a flow-change proposal received from the stack-master
/// cockpit. Proposal schema is defined in stack-master ADR-0011.
///
/// This function is deliberately separate from the sync path —
/// proposals must originate from the cockpit, not from this
/// module's own analysis.
pub fn apply_flow_proposal(_proposal_json: &str) -> Result<()> {
    unimplemented!(
        "Awaiting ADR-0011 proposal-schema finalization. \
         No direct flow mutation without cockpit proposal."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pull_flows_is_unimplemented_with_clear_message() {
        let result = std::panic::catch_unwind(|| {
            let _ = pull_flows_snapshot("http://example.local", "token");
        });
        assert!(result.is_err(), "expected panic from unimplemented stub");
    }

    #[test]
    fn apply_flow_proposal_is_unimplemented_with_clear_message() {
        let result = std::panic::catch_unwind(|| {
            let _ = apply_flow_proposal(r#"{"version":0}"#);
        });
        assert!(result.is_err(), "expected panic from unimplemented stub");
    }
}
