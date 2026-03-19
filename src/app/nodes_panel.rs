use super::AMSAgents;
use eframe::egui;
use rand::Rng;
use egui_snarl::ui::{BackgroundPattern, PinInfo, PinPlacement, SnarlStyle, SnarlViewer, SnarlWidget, WireStyle};
use egui_snarl::{InPin, NodeId, OutPin, Snarl};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, PartialEq, Eq)]
enum AgentNodeKind {
    Manager,
    Worker,
    Evaluator,
    Researcher,
    OutgoingHttp,
}

impl AgentNodeKind {
    fn label(&self) -> &'static str {
        match self {
            AgentNodeKind::Manager => "Agent Manager",
            AgentNodeKind::Worker => "Agent Worker",
            AgentNodeKind::Evaluator => "Agent Evaluator",
            AgentNodeKind::Researcher => "Agent Researcher",
            AgentNodeKind::OutgoingHttp => "Outgoing HTTP",
        }
    }

    fn inputs(&self) -> usize {
        match self {
            AgentNodeKind::Manager => 0,
            AgentNodeKind::Worker => 1,
            AgentNodeKind::Evaluator => 1,
            AgentNodeKind::Researcher => 1,
            AgentNodeKind::OutgoingHttp => 0,
        }
    }

    fn outputs(&self) -> usize {
        match self {
            AgentNodeKind::Manager => 1,
            AgentNodeKind::Worker => 1,
            AgentNodeKind::Evaluator => 0,
            AgentNodeKind::Researcher => 0,
            AgentNodeKind::OutgoingHttp => 0,
        }
    }
}

#[derive(Clone)]
struct NodeManagerData {
    name: String,
    global_id: String,
}

#[derive(Clone)]
struct NodeWorkerData {
    name: String,
    global_id: String,

    instruction_mode: String,
    instruction: String,

    analysis_mode: String,
    conversation_topic: String,
    conversation_topic_source: String,
    conversation_mode: String,

    conversation_partner_node: Option<NodeId>,

    conversation_active: bool,
    in_conversation: bool,

    /// Set when user clicks Start Conversation; processed after frame to spawn loop.
    start_requested: bool,
    /// Set when user clicks Stop Conversation; processed after frame to stop loop.
    stop_requested: bool,

    // Inferred from graph wires (Manager -> Worker)
    manager_node: Option<NodeId>,
}

#[derive(Clone)]
struct NodeEvaluatorData {
    name: String,
    global_id: String,

    analysis_mode: String,
    instruction: String,

    limit_token: bool,
    num_predict: String,

    active: bool,

    // Inferred from graph wires (Worker -> Evaluator -> Manager)
    worker_node: Option<NodeId>,
    manager_node: Option<NodeId>,
}

#[derive(Clone)]
struct NodeResearcherData {
    name: String,
    global_id: String,

    topic_mode: String,
    instruction: String,

    limit_token: bool,
    num_predict: String,

    active: bool,

    // Inferred from graph wires (Worker -> Researcher -> Manager)
    worker_node: Option<NodeId>,
    manager_node: Option<NodeId>,
}

#[derive(Clone)]
enum NodePayload {
    Manager(NodeManagerData),
    Worker(NodeWorkerData),
    Evaluator(NodeEvaluatorData),
    Researcher(NodeResearcherData),
    OutgoingHttp,
}

#[derive(Clone)]
pub(super) struct NodeData {
    kind: AgentNodeKind,
    pub label: String,
    payload: NodePayload,
}

impl NodeData {
    fn new_global_id() -> String {
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        const LEN: usize = 10;
        let mut rng = rand::rng();

        // Collisions are extremely unlikely; we don't coordinate across the whole app here.
        (0..LEN)
            .map(|_| {
                let idx = rng.random_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }

    fn new_manager() -> Self {
        let global_id = Self::new_global_id();
        Self {
            kind: AgentNodeKind::Manager,
            label: AgentNodeKind::Manager.label().to_string(),
            payload: NodePayload::Manager(NodeManagerData {
                name: AgentNodeKind::Manager.label().to_string(),
                global_id,
            }),
        }
    }

    fn new_worker() -> Self {
        let global_id = Self::new_global_id();
        Self {
            kind: AgentNodeKind::Worker,
            label: AgentNodeKind::Worker.label().to_string(),
            payload: NodePayload::Worker(NodeWorkerData {
                name: AgentNodeKind::Worker.label().to_string(),
                global_id,
                instruction_mode: "Assistant".to_string(),
                instruction: "You are a helpful assistant. Answer clearly, stay concise, and focus on the user request.".to_string(),
                analysis_mode: "European Politics".to_string(),
                conversation_topic: "Discuss European Politics and provide a concise overview of the main issue in one or two sentences.".to_string(),
                conversation_topic_source: "Own".to_string(),
                conversation_mode: "Shared".to_string(),
                conversation_partner_node: None,
                conversation_active: false,
                in_conversation: false,
                start_requested: false,
                stop_requested: false,
                manager_node: None,
            }),
        }
    }

    fn new_evaluator() -> Self {
        let global_id = Self::new_global_id();
        Self {
            kind: AgentNodeKind::Evaluator,
            label: AgentNodeKind::Evaluator.label().to_string(),
            payload: NodePayload::Evaluator(NodeEvaluatorData {
                name: AgentNodeKind::Evaluator.label().to_string(),
                global_id,
                analysis_mode: "Topic Extraction".to_string(),
                instruction: "Topic Extraction: extract the topic in 1 or 2 words. Identify what is the topic of the sentence being analysed.".to_string(),
                limit_token: false,
                num_predict: String::new(),
                active: false,
                worker_node: None,
                manager_node: None,
            }),
        }
    }

    fn new_researcher() -> Self {
        let global_id = Self::new_global_id();
        Self {
            kind: AgentNodeKind::Researcher,
            label: AgentNodeKind::Researcher.label().to_string(),
            payload: NodePayload::Researcher(NodeResearcherData {
                name: AgentNodeKind::Researcher.label().to_string(),
                global_id,
                topic_mode: "Articles".to_string(),
                instruction: "Generate article references connected to the message context. Prefer a mix of classic and recent pieces.".to_string(),
                limit_token: false,
                num_predict: String::new(),
                active: false,
                worker_node: None,
                manager_node: None,
            }),
        }
    }

    fn new_outgoing_http() -> Self {
        Self {
            kind: AgentNodeKind::OutgoingHttp,
            label: AgentNodeKind::OutgoingHttp.label().to_string(),
            payload: NodePayload::OutgoingHttp,
        }
    }

    fn set_name(&mut self, name: String) {
        match &mut self.payload {
            NodePayload::Manager(m) => m.name = name,
            NodePayload::Worker(w) => w.name = name,
            NodePayload::Evaluator(e) => e.name = name,
            NodePayload::Researcher(r) => r.name = name,
            NodePayload::OutgoingHttp => {}
        }
    }
}

pub(super) struct NodesPanelState {
    snarl: Snarl<NodeData>,
    wire_style: WireStyle,
}

impl Default for NodesPanelState {
    fn default() -> Self {
        let snarl = Snarl::new();
        Self {
            snarl,
            wire_style: WireStyle::Bezier5,
        }
    }
}

#[derive(Default)]
struct BasicNodeViewer;

impl BasicNodeViewer {
    fn numbered_name_for_kind(snarl: &Snarl<NodeData>, kind: AgentNodeKind) -> String {
        let idx = snarl
            .nodes_ids_data()
            .filter(|(_, n)| n.value.kind == kind)
            .count()
            + 1;
        format!("{} {}", idx, kind.label())
    }
}

impl SnarlViewer<NodeData> for BasicNodeViewer {
    fn title(&mut self, node: &NodeData) -> String {
        node.label.clone()
    }

    fn inputs(&mut self, _node: &NodeData) -> usize {
        _node.kind.inputs()
    }

    fn show_input(
        &mut self,
        _pin: &InPin,
        ui: &mut egui::Ui,
        _snarl: &mut Snarl<NodeData>,
    ) -> impl egui_snarl::ui::SnarlPin + 'static {
        ui.label("in");
        PinInfo::circle()
    }

    fn outputs(&mut self, node: &NodeData) -> usize {
        node.kind.outputs()
    }

    fn show_output(
        &mut self,
        _pin: &OutPin,
        ui: &mut egui::Ui,
        _snarl: &mut Snarl<NodeData>,
    ) -> impl egui_snarl::ui::SnarlPin + 'static {
        ui.label("out");
        PinInfo::circle()
    }

    fn has_body(&mut self, _node: &NodeData) -> bool {
        true
    }

    fn show_body(
        &mut self,
        node: NodeId,
        inputs: &[InPin],
        _outputs: &[OutPin],
        ui: &mut egui::Ui,
        snarl: &mut Snarl<NodeData>,
    ) {
        let node_kind = snarl.get_node(node).unwrap().kind.clone();

        match node_kind {
            AgentNodeKind::Manager => {
                let (name, global_id) = match &snarl.get_node(node).unwrap().payload {
                    NodePayload::Manager(m) => (m.name.clone(), m.global_id.clone()),
                    _ => unreachable!("kind mismatch"),
                };

                let mut erase = false;
                ui.vertical(|ui| {
                    ui.small(format!("Node {:?}", node));
                    ui.separator();
                    ui.label(egui::RichText::new(name).strong().size(12.0));
                    ui.separator();
                    ui.small(format!("Global ID: {}", global_id));
                    if ui.button("Erase").clicked() {
                        erase = true;
                    }
                });

                if erase {
                    snarl.remove_node(node);
                }
            }
            AgentNodeKind::Worker => {
                let my_manager_node = inputs
                    .get(0)
                    .and_then(|pin| pin.remotes.first())
                    .map(|out_pin_id| out_pin_id.node);

                // Compute read-only relationship data first (no mutable borrow of snarl).
                let worker_snapshot: Vec<(NodeId, NodeWorkerDataSnapshot)> = snarl
                    .nodes_ids_data()
                    .filter_map(|(id, node_data)| match &node_data.value.payload {
                        NodePayload::Worker(w) => Some((
                            id,
                            NodeWorkerDataSnapshot {
                                name: w.name.clone(),
                                conversation_partner_node: w.conversation_partner_node,
                                conversation_mode: w.conversation_mode.clone(),
                                manager_node: w.manager_node,
                            },
                        )),
                        _ => None,
                    })
                    .collect();

                let mut targeted_partner_mode_by_agent: HashMap<NodeId, String> = HashMap::new();
                let mut targeted_partner_ids: HashSet<NodeId> = HashSet::new();
                let mut chatting_partner_by_agent: HashMap<NodeId, String> = HashMap::new();

                for (agent_id, agent) in &worker_snapshot {
                    if let Some(partner_id) = agent.conversation_partner_node {
                        targeted_partner_mode_by_agent
                            .insert(partner_id, agent.conversation_mode.clone());
                        targeted_partner_ids.insert(partner_id);

                        let partner_name = worker_snapshot
                            .iter()
                            .find(|(id, _)| *id == partner_id)
                            .map(|(_, p)| p.name.clone())
                            .unwrap_or_else(|| "Unknown".to_string());

                        // Mirror the Workspace behavior: both sides show each other.
                        chatting_partner_by_agent.insert(*agent_id, partner_name.clone());
                        chatting_partner_by_agent.insert(partner_id, agent.name.clone());
                    }
                }

                let former_mode = targeted_partner_mode_by_agent.get(&node).cloned();
                let is_selected_by_other_agent = former_mode.is_some();
                let show_topic_when_selected = former_mode.is_some();

                let manager_name = my_manager_node
                    .and_then(|mid| snarl.get_node(mid))
                    .and_then(|nd| match &nd.payload {
                        NodePayload::Manager(m) => Some(m.name.clone()),
                        _ => None,
                    })
                    .unwrap_or_else(|| "Unassigned".to_string());

                // Compute options for "With:" dropdown (same manager wires).
                let mut worker_options: Vec<(NodeId, String)> = Vec::new();
                for (other_id, other) in &worker_snapshot {
                    if *other_id == node {
                        continue;
                    }
                    if my_manager_node.is_some() && other.manager_node != my_manager_node {
                        continue;
                    }
                    worker_options.push((*other_id, other.name.clone()));
                }

                // Update inferred manager_node for this worker node.
                if let Some(node_mut) = snarl.get_node_mut(node) {
                    if let NodePayload::Worker(w) = &mut node_mut.payload {
                        w.manager_node = my_manager_node;
                    }
                }

                let mut erase = false;
                {
                    let node_mut = snarl.get_node_mut(node).unwrap();
                    let worker_data = match &mut node_mut.payload {
                        NodePayload::Worker(w) => w,
                        _ => unreachable!("kind mismatch"),
                    };

                    ui.vertical(|ui| {
                            ui.small(format!("Node {:?}", node));
                            ui.separator();
                            AMSAgents::render_agent_worker_header(
                                ui,
                                &manager_name,
                            );
                            ui.separator();

                            ui.vertical(|ui| {
                                ui.spacing_mut().item_spacing = egui::Vec2::new(5.0, 2.0);

                                ui.vertical(|ui| {
                                    ui.vertical(|ui| {
                                        ui.spacing_mut().item_spacing =
                                            egui::Vec2::new(5.0, 2.0);

                                        ui.horizontal(|ui| {
                                            ui.label("Name:");
                                            ui.add(egui::TextEdit::singleline(
                                                &mut worker_data.name,
                                            ));
                                        });

                                        ui.horizontal(|ui| {
                                            ui.label("Instruction:");
                                            egui::ComboBox::from_id_salt(
                                                ui.id()
                                                    .with(node.0)
                                                    .with("instruction_mode"),
                                            )
                                            .selected_text(if worker_data
                                                .instruction_mode
                                                .is_empty()
                                            {
                                                "Select".to_string()
                                            } else {
                                                worker_data.instruction_mode.clone()
                                            })
                                            .show_ui(ui, |ui| {
                                                if ui
                                                    .selectable_label(
                                                        worker_data.instruction_mode
                                                            == "Assistant",
                                                        "Assistant",
                                                    )
                                                    .clicked()
                                                {
                                                    worker_data.instruction_mode =
                                                        "Assistant".to_string();
                                                    worker_data.instruction =
                                                        "You are a helpful assistant. Answer clearly, stay concise, and focus on the user request.".to_string();
                                                }
                                                if ui
                                                    .selectable_label(
                                                        worker_data.instruction_mode
                                                            == "Math Teacher",
                                                        "Math Teacher",
                                                    )
                                                    .clicked()
                                                {
                                                    worker_data.instruction_mode =
                                                        "Math Teacher".to_string();
                                                }
                                                if ui
                                                    .selectable_label(
                                                        worker_data.instruction_mode
                                                            == "Debate",
                                                        "Debate",
                                                    )
                                                    .clicked()
                                                {
                                                    worker_data.instruction_mode =
                                                        "Debate".to_string();
                                                }
                                            });
                                        });

                                        ui.horizontal(|ui| {
                                            ui.label("Instruction:");
                                            ui.add(egui::TextEdit::singleline(
                                                &mut worker_data.instruction,
                                            ));
                                        });
                                    });

                                    ui.separator();

                                    ui.vertical(|ui| {
                                        ui.spacing_mut().item_spacing =
                                            egui::Vec2::new(5.0, 2.0);

                                        if !is_selected_by_other_agent
                                            || show_topic_when_selected
                                        {
                                            ui.horizontal(|ui| {
                                                ui.label("Topic:");
                                                egui::ComboBox::from_id_salt(
                                                    ui.id()
                                                        .with(node.0)
                                                        .with("analysis_mode"),
                                                )
                                                .selected_text(if worker_data
                                                    .analysis_mode
                                                    .is_empty()
                                                {
                                                    "Select".to_string()
                                                } else {
                                                    worker_data.analysis_mode
                                                        .clone()
                                                })
                                                .show_ui(ui, |ui| {
                                                    if ui.selectable_label(
                                                        worker_data.analysis_mode
                                                            == "European Politics",
                                                        "European Politics",
                                                    ).clicked() {
                                                        worker_data.analysis_mode =
                                                            "European Politics"
                                                                .to_string();
                                                        worker_data.conversation_topic =
                                                            "Discuss European Politics and provide a concise overview of the main issue in one or two sentences."
                                                                .to_string();
                                                    }
                                                    if ui.selectable_label(
                                                        worker_data.analysis_mode
                                                            == "Mental Health",
                                                        "Mental Health",
                                                    ).clicked() {
                                                        worker_data.analysis_mode =
                                                            "Mental Health".to_string();
                                                        worker_data.conversation_topic =
                                                            "Discuss Mental Health and provide one or two practical insights about the topic."
                                                                .to_string();
                                                    }
                                                    if ui.selectable_label(
                                                        worker_data.analysis_mode
                                                            == "Electronics",
                                                        "Electronics",
                                                    ).clicked() {
                                                        worker_data.analysis_mode =
                                                            "Electronics".to_string();
                                                        worker_data.conversation_topic =
                                                            "Discuss Electronics and summarize one or two important points about the selected subject."
                                                                .to_string();
                                                    }
                                                });
                                            });
                                        }

                                        if !is_selected_by_other_agent
                                            || show_topic_when_selected
                                        {
                                            ui.horizontal(|ui| {
                                                ui.label("Topic:");
                                                ui.add(egui::TextEdit::singleline(
                                                    &mut worker_data
                                                        .conversation_topic,
                                                ));
                                            });

                                            ui.horizontal(|ui| {
                                                ui.label("Topic Source:");
                                                egui::ComboBox::from_id_salt(
                                                    ui.id().with(node.0).with(
                                                        "topic_source",
                                                    ),
                                                )
                                                .width(100.0)
                                                .selected_text(
                                                    worker_data
                                                        .conversation_topic_source
                                                        .clone(),
                                                )
                                                .show_ui(ui, |ui| {
                                                    ui.selectable_value(
                                                        &mut worker_data
                                                            .conversation_topic_source,
                                                        "Own".to_string(),
                                                        "Own",
                                                    );
                                                    ui.selectable_value(
                                                        &mut worker_data
                                                            .conversation_topic_source,
                                                        "Follow Partner"
                                                            .to_string(),
                                                        "Follow Partner",
                                                    );
                                                });
                                            });
                                        }

                                        if !is_selected_by_other_agent {
                                            ui.horizontal(|ui| {
                                                ui.label("Mode:");
                                                egui::ComboBox::from_id_salt(
                                                    ui.id().with(node.0).with(
                                                        "conversation_mode",
                                                    ),
                                                )
                                                .width(100.0)
                                                .selected_text(
                                                    worker_data
                                                        .conversation_mode
                                                        .clone(),
                                                )
                                                .show_ui(ui, |ui| {
                                                    if ui
                                                        .selectable_label(
                                                            worker_data
                                                                .conversation_mode
                                                                == "Shared",
                                                            "Shared",
                                                        )
                                                        .clicked()
                                                    {
                                                        worker_data.conversation_mode =
                                                            "Shared".to_string();
                                                    }
                                                    if ui
                                                        .selectable_label(
                                                            worker_data
                                                                .conversation_mode
                                                                == "Unique",
                                                            "Unique",
                                                        )
                                                        .clicked()
                                                    {
                                                        worker_data.conversation_mode =
                                                            "Unique".to_string();
                                                        worker_data.conversation_partner_node =
                                                            None;
                                                    }
                                                });
                                            });

                                            if worker_data.conversation_mode
                                                == "Shared"
                                                && !targeted_partner_ids
                                                    .contains(&node)
                                            {
                                                ui.horizontal(|ui| {
                                                    ui.label("With:");

                                                    let selected_text =
                                                        if let Some(pid) =
                                                            worker_data
                                                                .conversation_partner_node
                                                        {
                                                            worker_options
                                                                .iter()
                                                                .find(|(id, _)| {
                                                                    *id == pid
                                                                })
                                                                .map(|(_, name)| {
                                                                    name.clone()
                                                                })
                                                                .unwrap_or_else(|| {
                                                                    "Unknown".to_string()
                                                                })
                                                        } else {
                                                            "None".to_string()
                                                        };

                                                    egui::ComboBox::from_id_salt(
                                                        ui.id()
                                                            .with(node.0)
                                                            .with("partner"),
                                                    )
                                                    .width(100.0)
                                                    .selected_text(selected_text)
                                                    .show_ui(ui, |ui| {
                                                        ui.selectable_value(
                                                            &mut worker_data
                                                                .conversation_partner_node,
                                                            None,
                                                            "None",
                                                        );
                                                        for (other_id, other_name)
                                                            in &worker_options
                                                        {
                                                            ui.selectable_value(
                                                                &mut worker_data
                                                                    .conversation_partner_node,
                                                                Some(*other_id),
                                                                other_name,
                                                            );
                                                        }
                                                    });
                                                });
                                            }
                                        }

                                        if let Some(partner_name) =
                                            chatting_partner_by_agent.get(&node)
                                        {
                                            ui.horizontal(|ui| {
                                                ui.label(format!(
                                                    "Chatting with {}",
                                                    partner_name
                                                ));
                                            });
                                        }
                                    });
                                });
                            });

                            if !is_selected_by_other_agent {
                                ui.separator();
                                ui.horizontal(|ui| {
                                    let button_text = if worker_data
                                        .conversation_active
                                    {
                                        "Stop Conversation"
                                    } else {
                                        "Start Conversation"
                                    };
                                    let button =
                                        egui::Button::new(button_text);

                                    if ui.add(button).clicked() {
                                        if worker_data.conversation_active {
                                            worker_data.conversation_active = false;
                                            worker_data.in_conversation = false;
                                            worker_data.stop_requested = true;
                                        } else if !worker_data.conversation_topic.is_empty() {
                                            worker_data.conversation_active = true;
                                            worker_data.in_conversation = true;
                                            worker_data.start_requested = true;
                                        }
                                    }

                                    if ui.button("Status").clicked() {
                                        println!(
                                            "=== Worker Node {:?} Status ===",
                                            node
                                        );
                                        println!(
                                            "Global ID: {}",
                                            worker_data.global_id
                                        );
                                        println!(
                                            "Manager node: {:?}",
                                            worker_data.manager_node
                                        );
                                        println!(
                                            "Name: {}",
                                            worker_data.name
                                        );
                                    }

                                    if ui.button("Erase").clicked() {
                                        erase = true;
                                    }
                                });
                            }
                            ui.separator();
                            ui.small(format!("Global ID: {}", worker_data.global_id));
                    });
                }

                if erase {
                    snarl.remove_node(node);
                }
            }
            AgentNodeKind::Evaluator => {
                let my_worker_node = inputs
                    .get(0)
                    .and_then(|pin| pin.remotes.first())
                    .map(|out_pin_id| out_pin_id.node);

                let inferred_manager_node = my_worker_node
                    .and_then(|wn| snarl.get_node(wn))
                    .and_then(|nd| match &nd.payload {
                        NodePayload::Worker(w) => w.manager_node,
                        _ => None,
                    });

                let manager_name = inferred_manager_node
                    .and_then(|mid| snarl.get_node(mid))
                    .and_then(|nd| match &nd.payload {
                        NodePayload::Manager(m) => Some(m.name.clone()),
                        _ => None,
                    })
                    .unwrap_or_else(|| "Unassigned".to_string());

                // Update inferred wire relationships.
                if let Some(node_mut) = snarl.get_node_mut(node) {
                    if let NodePayload::Evaluator(e) = &mut node_mut.payload {
                        e.worker_node = my_worker_node;
                        e.manager_node = inferred_manager_node;
                    }
                }

                let mut erase = false;
                {
                    let node_mut = snarl.get_node_mut(node).unwrap();
                    let evaluator_data = match &mut node_mut.payload {
                        NodePayload::Evaluator(e) => e,
                        _ => unreachable!("kind mismatch"),
                    };

                    ui.vertical(|ui| {
                            ui.small(format!("Node {:?}", node));
                            ui.separator();
                            AMSAgents::render_agent_evaluator_header(
                                ui,
                                &manager_name,
                            );
                            ui.separator();

                            ui.horizontal(|ui| {
                                ui.label("Name:");
                                ui.add(egui::TextEdit::singleline(
                                    &mut evaluator_data.name,
                                ));
                            });

                            ui.horizontal(|ui| {
                                ui.label("Analysis:");
                                egui::ComboBox::from_id_salt(
                                    ui.id().with(node.0).with(
                                        "eval_analysis_mode",
                                    ),
                                )
                                .selected_text(if evaluator_data
                                    .analysis_mode
                                    .is_empty()
                                {
                                    "Select".to_string()
                                } else {
                                    evaluator_data.analysis_mode.clone()
                                })
                                .show_ui(ui, |ui| {
                                    if ui.selectable_label(
                                        evaluator_data.analysis_mode
                                            == "Topic Extraction",
                                        "Topic Extraction",
                                    ).clicked() {
                                        evaluator_data.analysis_mode =
                                            "Topic Extraction".to_string();
                                        evaluator_data.instruction = "Topic Extraction: extract the topic in 1 or 2 words. Identify what is the topic of the sentence being analysed.".to_string();
                                    }
                                    if ui.selectable_label(
                                        evaluator_data.analysis_mode
                                            == "Decision Analysis",
                                        "Decision Analysis",
                                    ).clicked() {
                                        evaluator_data.analysis_mode =
                                            "Decision Analysis".to_string();
                                        evaluator_data.instruction = "Decision Analysis: extract a decision in 1 or 2 sentences about the agent in the message being analysed. Focus on the concrete decision and its intent.".to_string();
                                    }
                                    if ui.selectable_label(
                                        evaluator_data.analysis_mode
                                            == "Sentiment Classification",
                                        "Sentiment Classification",
                                    ).clicked() {
                                        evaluator_data.analysis_mode =
                                            "Sentiment Classification".to_string();
                                        evaluator_data.instruction = "Sentiment Classification: extract the sentiment of the message being analysed and return one word that is the sentiment.".to_string();
                                    }
                                });
                            });

                            ui.horizontal(|ui| {
                                ui.label("Instruction:");
                                ui.add(egui::TextEdit::singleline(
                                    &mut evaluator_data.instruction,
                                ));
                            });

                            ui.horizontal(|ui| {
                                if ui
                                    .checkbox(&mut evaluator_data.limit_token, "Limit Token")
                                    .changed()
                                {
                                    if !evaluator_data.limit_token {
                                        evaluator_data.num_predict.clear();
                                    }
                                }
                                if evaluator_data.limit_token {
                                    ui.label("num_predict:");
                                    ui.add(egui::TextEdit::singleline(
                                        &mut evaluator_data.num_predict,
                                    ).desired_width(80.0));
                                }
                            });

                            ui.separator();
                            let button_text = if evaluator_data.active {
                                "Stop Evaluating"
                            } else {
                                "Evaluate"
                            };
                            let button = egui::Button::new(button_text);

                            ui.horizontal(|ui| {
                                if ui.add(button).clicked() {
                                    evaluator_data.active =
                                        !evaluator_data.active;
                                }

                                if ui.button("Status").clicked() {
                                    println!(
                                        "=== Evaluator Node {:?} Status ===",
                                        node
                                    );
                                    println!(
                                        "Global ID: {}",
                                        evaluator_data.global_id
                                    );
                                    println!(
                                        "Name: {}",
                                        evaluator_data.name
                                    );
                                }

                                if ui.button("Erase").clicked() {
                                    erase = true;
                                }
                            });
                            ui.separator();
                            ui.small(format!("Global ID: {}", evaluator_data.global_id));
                    });
                }

                if erase {
                    snarl.remove_node(node);
                }
            }
            AgentNodeKind::Researcher => {
                let my_worker_node = inputs
                    .get(0)
                    .and_then(|pin| pin.remotes.first())
                    .map(|out_pin_id| out_pin_id.node);

                let inferred_manager_node = my_worker_node
                    .and_then(|wn| snarl.get_node(wn))
                    .and_then(|nd| match &nd.payload {
                        NodePayload::Worker(w) => w.manager_node,
                        _ => None,
                    });

                let manager_name = inferred_manager_node
                    .and_then(|mid| snarl.get_node(mid))
                    .and_then(|nd| match &nd.payload {
                        NodePayload::Manager(m) => Some(m.name.clone()),
                        _ => None,
                    })
                    .unwrap_or_else(|| "Unassigned".to_string());

                if let Some(node_mut) = snarl.get_node_mut(node) {
                    if let NodePayload::Researcher(r) = &mut node_mut.payload {
                        r.worker_node = my_worker_node;
                        r.manager_node = inferred_manager_node;
                    }
                }

                let mut erase = false;
                {
                    let node_mut = snarl.get_node_mut(node).unwrap();
                    let researcher_data = match &mut node_mut.payload {
                        NodePayload::Researcher(r) => r,
                        _ => unreachable!("kind mismatch"),
                    };

                    ui.vertical(|ui| {
                            ui.small(format!("Node {:?}", node));
                            ui.separator();
                            AMSAgents::render_agent_researcher_header(
                                ui,
                                &manager_name,
                            );
                            ui.separator();

                            ui.horizontal(|ui| {
                                ui.label("Name:");
                                ui.add(egui::TextEdit::singleline(
                                    &mut researcher_data.name,
                                ));
                            });

                            ui.horizontal(|ui| {
                                ui.label("Topics:");
                                egui::ComboBox::from_id_salt(
                                    ui.id().with(node.0).with(
                                        "research_topic_mode",
                                    ),
                                )
                                .selected_text(if researcher_data.topic_mode
                                    .is_empty()
                                {
                                    "Select".to_string()
                                } else {
                                    researcher_data.topic_mode.clone()
                                })
                                .show_ui(ui, |ui| {
                                    if ui.selectable_label(
                                        researcher_data.topic_mode == "Articles",
                                        "Articles",
                                    ).clicked() {
                                        researcher_data.topic_mode =
                                            "Articles".to_string();
                                        researcher_data.instruction = "Generate article references connected to the message context. Prefer a mix of classic and recent pieces.".to_string();
                                    }
                                    if ui.selectable_label(
                                        researcher_data.topic_mode == "Movies",
                                        "Movies",
                                    ).clicked() {
                                        researcher_data.topic_mode =
                                            "Movies".to_string();
                                        researcher_data.instruction = "Generate movie references connected to the message context. Prefer diverse genres and well-known titles.".to_string();
                                    }
                                    if ui.selectable_label(
                                        researcher_data.topic_mode == "Music",
                                        "Music",
                                    ).clicked() {
                                        researcher_data.topic_mode =
                                            "Music".to_string();
                                        researcher_data.instruction = "Generate music references connected to the message context. Include artist and track or album when possible.".to_string();
                                    }
                                });
                            });

                            ui.horizontal(|ui| {
                                ui.label("Instruction:");
                                ui.add(egui::TextEdit::singleline(
                                    &mut researcher_data.instruction,
                                ));
                            });

                            ui.horizontal(|ui| {
                                if ui
                                    .checkbox(&mut researcher_data.limit_token, "Token")
                                    .changed()
                                {
                                    if !researcher_data.limit_token {
                                        researcher_data.num_predict.clear();
                                    }
                                }
                                if researcher_data.limit_token {
                                    ui.label("num_predict:");
                                    ui.add(egui::TextEdit::singleline(
                                        &mut researcher_data.num_predict,
                                    ).desired_width(80.0));
                                }
                            });

                            ui.separator();
                            let button_text = if researcher_data.active {
                                "Stop Researching"
                            } else {
                                "Research"
                            };
                            let button = egui::Button::new(button_text);

                            ui.horizontal(|ui| {
                                if ui.add(button).clicked() {
                                    researcher_data.active =
                                        !researcher_data.active;
                                }

                                if ui.button("Status").clicked() {
                                    println!(
                                        "=== Researcher Node {:?} Status ===",
                                        node
                                    );
                                    println!(
                                        "Global ID: {}",
                                        researcher_data.global_id
                                    );
                                    println!(
                                        "Name: {}",
                                        researcher_data.name
                                    );
                                }

                                if ui.button("Erase").clicked() {
                                    erase = true;
                                }
                            });
                            ui.separator();
                            ui.small(format!("Global ID: {}", researcher_data.global_id));
                    });
                }

                if erase {
                    snarl.remove_node(node);
                }
            }
            AgentNodeKind::OutgoingHttp => {
                let panel_border_color = ui.visuals().widgets.noninteractive.bg_stroke.color;
                let mut erase = false;
                ui.vertical(|ui| {
                    let outgoing_panel = egui::Frame::default()
                        .fill(egui::Color32::from_rgb(40, 40, 40))
                        .stroke(egui::Stroke::new(1.0, panel_border_color))
                        .corner_radius(4.0)
                        .inner_margin(egui::Margin::same(6));
                    outgoing_panel.show(ui, |ui| {
                        ui.label(egui::RichText::new("Outgoing HTTP").strong().size(12.0));
                        ui.add_space(4.0);
                        let terminal_height = 100.0;
                        let terminal_frame = egui::Frame::default()
                            .fill(egui::Color32::from_rgb(0, 0, 0))
                            .stroke(egui::Stroke::new(1.0, panel_border_color))
                            .inner_margin(egui::Margin::same(6))
                            .corner_radius(4.0);
                        ui.allocate_ui_with_layout(
                            egui::vec2(ui.available_width(), terminal_height),
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                terminal_frame.show(ui, |ui| {
                                    ui.set_min_width(ui.available_width());
                                    ui.set_min_height(terminal_height);
                                    ui.set_max_height(terminal_height);
                                    let lines = crate::http_client::get_outgoing_http_log_lines();
                                    egui::ScrollArea::vertical()
                                        .id_salt(ui.id().with(node.0).with("outgoing_http_node_scroll"))
                                        .auto_shrink([false, false])
                                        .stick_to_bottom(true)
                                        .show(ui, |ui| {
                                            for line in lines {
                                                ui.label(
                                                    egui::RichText::new(line)
                                                        .monospace()
                                                        .size(10.0)
                                                        .color(egui::Color32::WHITE),
                                                );
                                            }
                                        });
                                });
                            },
                        );
                        if ui.button("Erase").clicked() {
                            erase = true;
                        }
                    });
                });
                if erase {
                    snarl.remove_node(node);
                }
            }
        }
    }

    fn has_graph_menu(&mut self, _pos: egui::Pos2, _snarl: &mut Snarl<NodeData>) -> bool {
        true
    }

    fn show_graph_menu(
        &mut self,
        pos: egui::Pos2,
        ui: &mut egui::Ui,
        snarl: &mut Snarl<NodeData>,
    ) {
        ui.vertical(|ui| {
            ui.label("Spawn node");

            if ui.button(AgentNodeKind::Manager.label()).clicked() {
                let mut node = NodeData::new_manager();
                node.set_name(Self::numbered_name_for_kind(snarl, AgentNodeKind::Manager));
                snarl.insert_node(pos, node);
            }

            if ui.button(AgentNodeKind::Worker.label()).clicked() {
                let mut node = NodeData::new_worker();
                node.set_name(Self::numbered_name_for_kind(snarl, AgentNodeKind::Worker));
                snarl.insert_node(pos, node);
            }

            if ui.button(AgentNodeKind::Evaluator.label()).clicked() {
                let mut node = NodeData::new_evaluator();
                node.set_name(Self::numbered_name_for_kind(snarl, AgentNodeKind::Evaluator));
                snarl.insert_node(pos, node);
            }

            if ui.button(AgentNodeKind::Researcher.label()).clicked() {
                let mut node = NodeData::new_researcher();
                node.set_name(Self::numbered_name_for_kind(snarl, AgentNodeKind::Researcher));
                snarl.insert_node(pos, node);
            }

            if ui.button(AgentNodeKind::OutgoingHttp.label()).clicked() {
                snarl.insert_node(pos, NodeData::new_outgoing_http());
            }
        });
    }
}

#[derive(Clone)]
struct NodeWorkerDataSnapshot {
    name: String,
    conversation_partner_node: Option<NodeId>,
    conversation_mode: String,
    manager_node: Option<NodeId>,
}

impl AMSAgents {
    fn print_nodes_graph_snapshot(&self) {
        let mut nodes: Vec<(usize, String, &'static str)> = self
            .nodes_panel
            .snarl
            .nodes_ids_data()
            .map(|(id, node)| {
                let (name, kind) = match &node.value.payload {
                    NodePayload::Manager(m) => (m.name.clone(), "Manager"),
                    NodePayload::Worker(w) => (w.name.clone(), "Worker"),
                    NodePayload::Evaluator(e) => (e.name.clone(), "Evaluator"),
                    NodePayload::Researcher(r) => (r.name.clone(), "Researcher"),
                    NodePayload::OutgoingHttp => ("Outgoing HTTP".to_string(), "OutgoingHttp"),
                };
                (id.0, name, kind)
            })
            .collect();
        nodes.sort_by_key(|(id, _, _)| *id);

        let label_by_id: HashMap<usize, String> = nodes
            .iter()
            .map(|(id, name, kind)| (*id, format!("{} [{}]", name, kind)))
            .collect();

        let mut edges: Vec<String> = Vec::new();
        for (id, node) in self.nodes_panel.snarl.nodes_ids_data() {
            match &node.value.payload {
                NodePayload::Worker(w) => {
                    if let Some(mid) = w.manager_node {
                        edges.push(format!(
                            "{} -> {}  (manager_to_worker)",
                            label_by_id.get(&mid.0).cloned().unwrap_or_else(|| format!("Node {}", mid.0)),
                            label_by_id.get(&id.0).cloned().unwrap_or_else(|| format!("Node {}", id.0)),
                        ));
                    }
                    if w.conversation_mode == "Shared" {
                        if let Some(pid) = w.conversation_partner_node {
                            if id.0 < pid.0 {
                                edges.push(format!(
                                    "{} <-> {}  (worker_shared_partner)",
                                    label_by_id.get(&id.0).cloned().unwrap_or_else(|| format!("Node {}", id.0)),
                                    label_by_id.get(&pid.0).cloned().unwrap_or_else(|| format!("Node {}", pid.0)),
                                ));
                            }
                        }
                    }
                }
                NodePayload::Evaluator(e) => {
                    if let Some(wid) = e.worker_node {
                        edges.push(format!(
                            "{} -> {}  (worker_to_evaluator)",
                            label_by_id.get(&wid.0).cloned().unwrap_or_else(|| format!("Node {}", wid.0)),
                            label_by_id.get(&id.0).cloned().unwrap_or_else(|| format!("Node {}", id.0)),
                        ));
                    }
                    if let Some(mid) = e.manager_node {
                        edges.push(format!(
                            "{} -> {}  (evaluator_to_manager)",
                            label_by_id.get(&id.0).cloned().unwrap_or_else(|| format!("Node {}", id.0)),
                            label_by_id.get(&mid.0).cloned().unwrap_or_else(|| format!("Node {}", mid.0)),
                        ));
                    }
                }
                NodePayload::Researcher(r) => {
                    if let Some(wid) = r.worker_node {
                        edges.push(format!(
                            "{} -> {}  (worker_to_researcher)",
                            label_by_id.get(&wid.0).cloned().unwrap_or_else(|| format!("Node {}", wid.0)),
                            label_by_id.get(&id.0).cloned().unwrap_or_else(|| format!("Node {}", id.0)),
                        ));
                    }
                    if let Some(mid) = r.manager_node {
                        edges.push(format!(
                            "{} -> {}  (researcher_to_manager)",
                            label_by_id.get(&id.0).cloned().unwrap_or_else(|| format!("Node {}", id.0)),
                            label_by_id.get(&mid.0).cloned().unwrap_or_else(|| format!("Node {}", mid.0)),
                        ));
                    }
                }
                NodePayload::Manager(_) | NodePayload::OutgoingHttp => {}
            }
        }
        edges.sort();

        println!("=== Run Graph ===");
        println!("Nodes ({}):", nodes.len());
        for (id, name, kind) in &nodes {
            println!("  [{}] {} ({})", id, name, kind);
        }
        println!("Edges ({}):", edges.len());
        for e in &edges {
            println!("  {}", e);
        }
    }

    /// Stops the conversation loop for a worker node (by node id used as handle key).
    fn stop_conversation_for_node(&mut self, node_id: NodeId) {
        let key = node_id.0;
        self.conversation_loop_handles.retain(|(id, flag, _)| {
            if *id == key {
                *flag.lock().unwrap() = false;
                false
            } else {
                true
            }
        });
    }

    /// Starts the conversation loop for a worker node using the same ADK/HTTP flow as the old Workspace.
    /// All agent data must be resolved beforehand to avoid holding snarl while borrowing self.
    fn start_conversation_from_node_worker_resolved(
        &mut self,
        node_id: NodeId,
        agent_a_name: String,
        agent_a_instruction: String,
        agent_a_topic: String,
        agent_a_topic_source: String,
        agent_b_id: usize,
        agent_b_name: String,
        agent_b_instruction: String,
        agent_b_topic: String,
        agent_b_topic_source: String,
    ) {
        let agent_a_id = node_id.0;
        let active_flag = Arc::new(Mutex::new(true));
        let flag_clone = active_flag.clone();
        let endpoint = self.http_endpoint.clone();
        let last_msg = self.last_message_in_chat.clone();
        let selected_model = if self.selected_ollama_model.trim().is_empty() {
            None
        } else {
            Some(self.selected_ollama_model.clone())
        };
        let history_size = self.conversation_history_size;
        let turn_delay_secs = self.conversation_turn_delay_secs;
        let handle = self.rt_handle.clone();

        let loop_handle = handle.spawn(async move {
            crate::agent_conversation_loop::start_conversation_loop(
                agent_a_id,
                agent_a_name,
                agent_a_instruction,
                agent_a_topic,
                agent_a_topic_source,
                agent_b_id,
                agent_b_name,
                agent_b_instruction,
                agent_b_topic,
                agent_b_topic_source,
                endpoint,
                flag_clone,
                last_msg,
                selected_model,
                history_size,
                turn_delay_secs,
            )
            .await;
        });

        self.conversation_loop_handles.push((node_id.0, active_flag, loop_handle));
    }

    pub(super) fn render_nodes_panel(&mut self, ui: &mut egui::Ui) {
        let panel_border_color = ui.visuals().widgets.noninteractive.bg_stroke.color;
        let nodes_panel = egui::Frame::default()
            .fill(egui::Color32::from_rgb(40, 40, 40))
            .stroke(egui::Stroke::new(1.0, panel_border_color))
            .corner_radius(4.0)
            .inner_margin(egui::Margin::same(6));

        // Nodes panel extends to bottom of window (no fixed Outgoing HTTP panel).
        let panel_height = ui.available_height().max(120.0);
        let mut viewer = BasicNodeViewer;

        // Customize Snarl appearance:
        // - start zoom at ~1.0 (clamp initial scaling)
        // - place pins on the node edge (the "ball" on border)
        // - thin grey grid lines
        let mut style = SnarlStyle::new();
        style.min_scale = Some(0.25);
        style.max_scale = Some(2.0);
        style.pin_placement = Some(PinPlacement::Edge);
        style.wire_style = Some(self.nodes_panel.wire_style);
        style.wire_width = Some(3.0);
        style.wire_smoothness = Some(0.0);
        style.pin_stroke = Some(egui::Stroke::new(1.5, egui::Color32::from_gray(200)));
        // Vertical + horizontal grid (no rotation).
        style.bg_pattern = Some(BackgroundPattern::grid(egui::vec2(50.0, 50.0), 0.0));
        style.bg_pattern_stroke = Some(egui::Stroke::new(
            1.0,
            egui::Color32::from_gray(110).gamma_multiply(0.5),
        ));

        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), panel_height),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                nodes_panel.show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Nodes").strong().size(12.0));
                        ui.add_space(8.0);
                        let label = match self.nodes_panel.wire_style {
                            WireStyle::Bezier5 => "Wires: Bezier5",
                            WireStyle::Line => "Wires: Line",
                            _ => "Wires: (custom)",
                        };
                        if ui.button(label).clicked() {
                            self.nodes_panel.wire_style = match self.nodes_panel.wire_style {
                                WireStyle::Bezier5 => WireStyle::Line,
                                _ => WireStyle::Bezier5,
                            };
                        }
                        if ui.button("Run Graph").clicked() {
                            self.print_nodes_graph_snapshot();
                        }
                    });
                    ui.add_space(4.0);
                    SnarlWidget::new()
                        .id_salt("ams_nodes_panel_v2")
                        .style(style)
                        .show(&mut self.nodes_panel.snarl, &mut viewer, ui);
                });

                let ctx = ui.ctx().clone();
                let last_msg = self.last_message_in_chat.lock().unwrap().clone();

                // Process pending Start/Stop conversation from Worker nodes (same ADK/HTTP as old Workspace).
                let mut to_stop = Vec::new();
                let mut to_start = Vec::new();
                for (id, node) in self.nodes_panel.snarl.nodes_ids_data() {
                    if let NodePayload::Worker(w) = &node.value.payload {
                        if w.stop_requested {
                            to_stop.push(id);
                        }
                        if w.start_requested {
                            let (agent_b_id, agent_b_name, agent_b_instruction, agent_b_topic, agent_b_topic_source) =
                                if w.conversation_mode == "Unique" || w.conversation_partner_node.is_none() {
                                    (id.0, w.name.clone(), w.instruction.clone(), w.conversation_topic.clone(), w.conversation_topic_source.clone())
                                } else {
                                    let pid = w.conversation_partner_node.unwrap();
                                    match self.nodes_panel.snarl.get_node(pid).map(|n| &n.payload) {
                                        Some(NodePayload::Worker(p)) => (
                                            pid.0,
                                            p.name.clone(),
                                            p.instruction.clone(),
                                            p.conversation_topic.clone(),
                                            p.conversation_topic_source.clone(),
                                        ),
                                        _ => (id.0, w.name.clone(), w.instruction.clone(), w.conversation_topic.clone(), w.conversation_topic_source.clone()),
                                    }
                                };
                            to_start.push((
                                id,
                                w.name.clone(),
                                w.instruction.clone(),
                                w.conversation_topic.clone(),
                                w.conversation_topic_source.clone(),
                                agent_b_id,
                                agent_b_name,
                                agent_b_instruction,
                                agent_b_topic,
                                agent_b_topic_source,
                            ));
                        }
                    }
                }
                let start_ids: Vec<NodeId> = to_start.iter().map(|(id, ..)| *id).collect();
                for node_id in &to_stop {
                    self.stop_conversation_for_node(*node_id);
                }
                for (node_id, a_name, a_instr, a_topic, a_src, b_id, b_name, b_instr, b_topic, b_src) in to_start {
                    self.start_conversation_from_node_worker_resolved(
                        node_id, a_name, a_instr, a_topic, a_src, b_id, b_name, b_instr, b_topic, b_src,
                    );
                }
                let clear_flags: HashSet<NodeId> =
                    to_stop.iter().copied().chain(start_ids.into_iter()).collect();
                for (id, node) in self.nodes_panel.snarl.nodes_ids_data_mut() {
                    if clear_flags.contains(&id) {
                        if let NodePayload::Worker(w) = &mut node.value.payload {
                            w.start_requested = false;
                            w.stop_requested = false;
                        }
                    }
                }

                // Run Evaluator nodes: if active and last_message_in_chat is new for this node, run Ollama + send_evaluator_result.
                for (id, node) in self.nodes_panel.snarl.nodes_ids_data() {
                    if let NodePayload::Evaluator(e) = &node.value.payload {
                        if !e.active {
                            continue;
                        }
                        let last_eval = self.last_evaluated_message_by_evaluator.lock().unwrap().get(&id.0).cloned();
                        if last_msg.as_ref().map_or(false, |s| !s.is_empty())
                            && last_eval.as_ref() != last_msg.as_ref()
                        {
                            let message = last_msg.clone().unwrap_or_default();
                            self.last_evaluated_message_by_evaluator
                                .lock()
                                .unwrap()
                                .insert(id.0, message.clone());
                            let instruction = e.instruction.clone();
                            let analysis_mode = e.analysis_mode.clone();
                            let limit_token = e.limit_token;
                            let num_predict = e.num_predict.clone();
                            let endpoint = self.http_endpoint.clone();
                            let ctx = ctx.clone();
                            let handle = self.rt_handle.clone();
                            let selected_model = if self.selected_ollama_model.trim().is_empty() {
                                None
                            } else {
                                Some(self.selected_ollama_model.clone())
                            };
                            handle.spawn(async move {
                                match crate::adk_integration::send_to_ollama(
                                    &instruction,
                                    &message,
                                    limit_token,
                                    &num_predict,
                                    selected_model.as_deref(),
                                )
                                .await
                                {
                                    Ok(response) => {
                                        let response_lower = response.to_lowercase();
                                        let sentiment = match analysis_mode.as_str() {
                                            "Topic Extraction" => "topic",
                                            "Decision Analysis" => "decision",
                                            "Sentiment Classification" => {
                                                if response_lower.contains("positive")
                                                    || response_lower.contains("happy")
                                                {
                                                    "sentiment"
                                                } else if response_lower.contains("negative")
                                                    || response_lower.contains("sad")
                                                    || response_lower.contains("angry")
                                                    || response_lower.contains("frustrated")
                                                {
                                                    "sentiment"
                                                } else if response_lower.contains("neutral") {
                                                    "sentiment"
                                                } else {
                                                    "unknown"
                                                }
                                            }
                                            _ => {
                                                if response_lower.contains("happy") {
                                                    "happy"
                                                } else if response_lower.contains("sad") {
                                                    "sad"
                                                } else {
                                                    "analysis"
                                                }
                                            }
                                        };
                                        if let Err(e) = crate::http_client::send_evaluator_result(
                                            &endpoint,
                                            "Agent Evaluator",
                                            sentiment,
                                            &response,
                                        )
                                        .await
                                        {
                                            eprintln!("[Evaluator] Failed to send to ams-chat: {}", e);
                                        }
                                    }
                                    Err(e) => eprintln!("[Evaluator] Ollama error: {}", e),
                                }
                                ctx.request_repaint();
                            });
                        }
                    }
                }

                // Run Researcher nodes: if active and last_message_in_chat is new for this node, run Ollama + send_researcher_result.
                for (id, node) in self.nodes_panel.snarl.nodes_ids_data() {
                    if let NodePayload::Researcher(r) = &node.value.payload {
                        if !r.active {
                            continue;
                        }
                        let last_research = self
                            .last_researched_message_by_researcher
                            .lock()
                            .unwrap()
                            .get(&id.0)
                            .cloned();
                        if last_msg.as_ref().map_or(false, |s| !s.is_empty())
                            && last_research.as_ref() != last_msg.as_ref()
                        {
                            let message = last_msg.clone().unwrap_or_default();
                            self.last_researched_message_by_researcher
                                .lock()
                                .unwrap()
                                .insert(id.0, message.clone());
                            let topic = if r.topic_mode.trim().is_empty() {
                                "Articles".to_string()
                            } else {
                                r.topic_mode.clone()
                            };
                            let instruction = format!(
                                "{}\n\nUsing the latest chat message, suggest 3 {} references related to what was said. Keep it concise with bullet points: title and one-line why it matches.",
                                r.instruction,
                                topic.to_lowercase()
                            );
                            let limit_token = r.limit_token;
                            let num_predict = r.num_predict.clone();
                            let endpoint = self.http_endpoint.clone();
                            let ctx = ctx.clone();
                            let handle = self.rt_handle.clone();
                            let selected_model = if self.selected_ollama_model.trim().is_empty() {
                                None
                            } else {
                                Some(self.selected_ollama_model.clone())
                            };
                            handle.spawn(async move {
                                match crate::adk_integration::send_to_ollama(
                                    &instruction,
                                    &message,
                                    limit_token,
                                    &num_predict,
                                    selected_model.as_deref(),
                                )
                                .await
                                {
                                    Ok(response) => {
                                        if let Err(e) = crate::http_client::send_researcher_result(
                                            &endpoint,
                                            "Agent Researcher",
                                            &topic,
                                            &response,
                                        )
                                        .await
                                        {
                                            eprintln!(
                                                "[Researcher] Failed to send to ams-chat: {}",
                                                e
                                            );
                                        }
                                    }
                                    Err(e) => eprintln!("[Researcher] Ollama error: {}", e),
                                }
                                ctx.request_repaint();
                            });
                        }
                    }
                }
            },
        );
    }
}
