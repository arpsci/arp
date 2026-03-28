//! Evaluator/researcher `active` flags derived from row wiring.

use super::model::NodePayload;
use super::state::AgentRecord;

/// Refresh evaluator/researcher `active` from row selections (combiners may skip the row body when collapsed).
pub(super) fn sync_evaluator_researcher_activity(agents: &mut [AgentRecord]) {
    for r in agents.iter_mut() {
        match &mut r.data.payload {
            NodePayload::Evaluator(e) => {
                e.active = e.evaluate_all_workers || e.worker_node.is_some();
            }
            NodePayload::Researcher(res) => {
                res.active = res.worker_node.is_some();
            }
            _ => {}
        }
    }
}
