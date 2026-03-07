use eframe::egui;
use tokio::runtime::Handle;

#[derive(Clone)]
struct Agent {
    id: usize,
    name: String,
    selected: bool,
    instruction: String,
    limit_token: bool,
    num_predict: String,
}

pub struct MyApp {
    rt_handle: Handle,
    agents: Vec<Agent>,
    next_agent_id: usize,
}

impl MyApp {
    pub fn new(rt_handle: Handle) -> Self {
        Self { 
            rt_handle,
            agents: Vec::new(),
            next_agent_id: 1,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.add_space(20.0);
                    
                    if ui.button("Hello").clicked() {
                        println!("Hello");
                    }
                    
                    ui.add_space(10.0);
                    
                    if ui.button("Test Ollama").clicked() {
                        println!("Testing Ollama integration");
                        let ctx = ctx.clone();
                        let handle = self.rt_handle.clone();
                        handle.spawn(async move {
                            match crate::adk_integration::test_ollama().await {
                                Ok(_response) => {
                                    // Response is already printed during streaming in test_ollama()
                                }
                                Err(e) => {
                                    eprintln!("Ollama error: {}", e);
                                }
                            }
                            ctx.request_repaint();
                        });
                    }

                    ui.separator();

                    if ui.button("Create Agent").clicked() {
                        // Find the lowest available ID
                        let used_ids: std::collections::HashSet<usize> = 
                            self.agents.iter().map(|a| a.id).collect();
                        let mut new_id = 1;
                        while used_ids.contains(&new_id) {
                            new_id += 1;
                        }
                        
                        self.agents.push(Agent {
                            id: new_id,
                            name: format!("Agent {}", new_id),
                            selected: false,
                            instruction: "You are an assistant".to_string(),
                            limit_token: false,
                            num_predict: String::new(),
                        });
                        
                        // Update next_agent_id to be at least one more than the highest used ID
                        if new_id >= self.next_agent_id {
                            self.next_agent_id = new_id + 1;
                        }
                    }
                });
                
                ui.separator();
                
                // Scrollable area for agents with green border - full width
                let available_width = ui.available_width();
                egui::Frame::default()
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(0, 255, 0)))
                    .show(ui, |ui| {
                        ui.set_width(available_width);
                        egui::ScrollArea::vertical().show(ui, |ui| {
                        
                // Collect IDs of agents to remove and select
                let mut agents_to_remove = Vec::new();
                let mut agent_to_select: Option<usize> = None;
                
                // Display agents in rows
                for agent in &mut self.agents {
                    let agent_id = agent.id;
                    
                    // Agent row - 100% width, 100px height, minimal spacing
                    ui.allocate_ui_with_layout(
                        egui::Vec2::new(ui.available_width(), 30.0),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            ui.set_width(ui.available_width());
                            
                            // Clickable area with background color change
                            let bg_color = if agent.selected {
                                egui::Color32::from_rgb(60, 60, 60)
                            } else {
                                egui::Color32::from_rgb(45, 45, 45)
                            };
                            
                            let frame = egui::Frame::default()
                                .fill(bg_color)
                                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(255, 192, 203))) // Pink border
                                .inner_margin(egui::Margin::same(5.0))
                                .outer_margin(egui::Margin::same(0.0));
                            
                            let frame_response = frame.show(ui, |ui| {
                                ui.vertical(|ui| {
                                    ui.spacing_mut().item_spacing = egui::Vec2::new(5.0, 2.0);
                                    
                                    // Agent Name row
                                    ui.horizontal(|ui| {
                                        ui.label("Agent Name:");
                                        ui.add(egui::TextEdit::singleline(&mut agent.name)
                                            .desired_width(100.0));
                                    });
                                    
                                    // Instruction row (system prompt)
                                    ui.horizontal(|ui| {
                                        ui.label("Instruction:");
                                        ui.add(egui::TextEdit::singleline(&mut agent.instruction)
                                            .desired_width(200.0));
                                    });
                                    
                                    // Limit token checkbox and num_predict row
                                    ui.horizontal(|ui| {
                                        if ui.checkbox(&mut agent.limit_token, "Limit Token").changed() {
                                            if !agent.limit_token {
                                                agent.num_predict.clear();
                                            }
                                        }
                                        
                                        if agent.limit_token {
                                            ui.label("num_predict:");
                                            ui.add(egui::TextEdit::singleline(&mut agent.num_predict)
                                                .desired_width(80.0));
                                        }
                                    });
                                    
                                    // Status and Erase Agent buttons row
                                    ui.horizontal(|ui| {
                                        if ui.button("Status").clicked() {
                                            println!("=== Agent {} Status ===", agent.id);
                                            println!("Name: {}", agent.name);
                                            println!("Instruction: {}", agent.instruction);
                                            println!("Limit Token: {}", agent.limit_token);
                                            if agent.limit_token {
                                                println!("num_predict: {}", agent.num_predict);
                                            }
                                            println!("Selected: {}", agent.selected);
                                            println!("======================");
                                        }
                                        
                                        if ui.button("Erase Agent").clicked() {
                                            agents_to_remove.push(agent_id);
                                        }
                                    });
                                });
                            });
                            
                            // Make the whole frame area clickable
                            if frame_response.response.clicked() {
                                agent_to_select = Some(agent_id);
                            }
                        }
                    );
                    
                    // No spacing between rows
                }
                
                // Apply selection changes
                if let Some(selected_id) = agent_to_select {
                    for a in &mut self.agents {
                        a.selected = a.id == selected_id;
                    }
                }
                
                // Remove agents that were marked for deletion
                for id in agents_to_remove {
                    self.agents.retain(|a| a.id != id);
                }
                    });
                });
            });
        });
    }
}

