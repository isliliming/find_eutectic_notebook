use eframe::egui;
use serde::{Deserialize, Serialize};
use std::fs;
use chrono::{DateTime, Utc};
use std::error::Error;

// --- Data Structures ---

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
enum Role {
    #[default]
    HBA, // Acceptor
    HBD, // Donor
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
enum Phase {
    #[default]
    Liquid,
    Solid,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Molecule {
    id: u32,
    name: String,
    short_name: String,
    cas_number: String,
    mw: f64,
    role: Role,
    supplier: String,
    sku: String,
    webpage: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
enum SampleType {
    #[default]
    Binary,
    Pure,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Sample {
    id: u32,
    type_: SampleType,

    // Component 1 (Always exists)
    c1_id: u32,
    c1_name: String,
    c1_mass: f64,

    // Component 2 (Optional, only for Binary)
    c2_id: Option<u32>,
    c2_name: Option<String>,
    c2_mass: Option<f64>,

    // Ratio info
    ratio_label: String, // e.g. "1:2" or "Pure"
    real_molar_ratio: Option<f64>, // Only for Binary

    created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Observation {
    id: u32,
    sample_id: u32,
    temperature: f64,
    result: Phase,
    timestamp: DateTime<Utc>,
}

// --- The Application State ---

#[derive(Serialize, Deserialize)]
struct LabApp {
    molecules: Vec<Molecule>,
    samples: Vec<Sample>,
    observations: Vec<Observation>,

    next_mol_id: u32,
    next_sample_id: u32,
    next_obs_id: u32,

    // UI State (Not saved)
    #[serde(skip)]
    current_tab: Tab,
    #[serde(skip)]
    status_msg: Option<(String, DateTime<Utc>)>,

    #[serde(skip)]
    search_query: String,

    // Tab 1: Molecule Input
    #[serde(skip)]
    new_mol_name: String,
    #[serde(skip)]
    new_mol_short: String,
    #[serde(skip)]
    new_mol_cas: String,
    #[serde(skip)]
    new_mol_mw: String,
    #[serde(skip)]
    new_mol_role: Role,
    #[serde(skip)]
    new_mol_supplier: String,
    #[serde(skip)]
    new_mol_sku: String,
    #[serde(skip)]
    new_mol_link: String,

    // Tab 2: Sample Prep Input
    #[serde(skip)]
    prep_mode: SampleType,
    #[serde(skip)]
    prep_c1_id: Option<u32>,
    #[serde(skip)]
    prep_c2_id: Option<u32>,
    #[serde(skip)]
    prep_parts_c1: String,
    #[serde(skip)]
    prep_parts_c2: String,
    #[serde(skip)]
    prep_mass_c1: String,
    #[serde(skip)]
    prep_mass_c2_actual: String,

    // Tab 3: Testing Input
    #[serde(skip)]
    test_selected_sample_id: Option<u32>,
    #[serde(skip)]
    test_temp: String,
    #[serde(skip)]
    test_result: Phase,
}

#[derive(PartialEq, Default)]
enum Tab {
    #[default]
    Molecules,
    SamplePrep,
    Testing,
    History,
}

impl Default for LabApp {
    fn default() -> Self {
        Self {
            molecules: Vec::new(),
            samples: Vec::new(),
            observations: Vec::new(),
            next_mol_id: 1,
            next_sample_id: 1,
            next_obs_id: 1,
            current_tab: Tab::Molecules,
            status_msg: None,
            search_query: String::new(),

            new_mol_name: String::new(),
            new_mol_short: String::new(),
            new_mol_cas: String::new(),
            new_mol_mw: String::new(),
            new_mol_role: Role::HBA,
            new_mol_supplier: String::new(),
            new_mol_sku: String::new(),
            new_mol_link: String::new(),

            prep_mode: SampleType::Binary,
            prep_c1_id: None,
            prep_c2_id: None,
            prep_parts_c1: "1".to_string(),
            prep_parts_c2: "2".to_string(),
            prep_mass_c1: "100.0".to_string(),
            prep_mass_c2_actual: String::new(),

            test_selected_sample_id: None,
            test_temp: "25.0".to_string(),
            test_result: Phase::Liquid,
        }
    }
}

impl LabApp {
    fn save_to_disk(&self) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write("lab_notebook.json", json);
        }
    }

    fn load_from_disk() -> Self {
        if let Ok(content) = fs::read_to_string("lab_notebook.json") {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    fn show_status(&mut self, msg: &str) {
        self.status_msg = Some((msg.to_string(), Utc::now()));
    }

    // --- UPDATED Export Function ---
    fn export_to_csv(&self) -> Result<(), Box<dyn Error>> {
        let mut wtr = csv::Writer::from_path("experiment_data.csv")?;

        // Write Header with Mass and Real Ratio
        wtr.write_record(&[
            "Date",
            "Obs ID",
            "Sample ID",
            "Type",
            "Component 1",
            "Mass 1 (mg)", // <--- Added
            "Component 2",
            "Mass 2 (mg)", // <--- Added
            "Real Ratio (1:x)", // <--- Added
            "Temp (C)",
            "Result"
        ])?;

        for obs in &self.observations {
            if let Some(sample) = self.samples.iter().find(|s| s.id == obs.sample_id) {

                // Prepare fields
                let date_str = obs.timestamp.format("%Y-%m-%d %H:%M").to_string();
                let type_str = match sample.type_ { SampleType::Binary => "Binary", SampleType::Pure => "Pure" };

                // Handle optional Component 2 data
                let c2_str = sample.c2_name.as_deref().unwrap_or("N/A");
                let c2_mass_str = sample.c2_mass.map(|m| m.to_string()).unwrap_or("0.0".to_string());

                // Handle Real Ratio
                let ratio_str = sample.real_molar_ratio.map(|r| format!("{:.4}", r)).unwrap_or("N/A".to_string());

                wtr.write_record(&[
                    date_str,
                    obs.id.to_string(),
                    sample.id.to_string(),
                    type_str.to_string(),
                    sample.c1_name.clone(),
                    sample.c1_mass.to_string(), // Mass 1
                    c2_str.to_string(),
                    c2_mass_str,                // Mass 2
                    ratio_str,                  // Real Ratio
                    obs.temperature.to_string(),
                    format!("{:?}", obs.result)
                ])?;
            }
        }
        wtr.flush()?;
        Ok(())
    }
}

impl eframe::App for LabApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Header
            ui.horizontal(|ui| {
                ui.heading("🧪 Eutectic Lab Manager");
                if let Some((msg, time)) = &self.status_msg {
                    if (Utc::now() - *time).num_seconds() < 3 {
                        ui.colored_label(egui::Color32::GREEN, format!("  ✅ {}", msg));
                    }
                }
            });
            ui.separator();

            // Tabs
            ui.horizontal(|ui| {
                if ui.selectable_label(self.current_tab == Tab::Molecules, "1. Molecules").clicked() { self.current_tab = Tab::Molecules; }
                if ui.selectable_label(self.current_tab == Tab::SamplePrep, "2. Weighing").clicked() { self.current_tab = Tab::SamplePrep; }
                if ui.selectable_label(self.current_tab == Tab::Testing, "3. Testing").clicked() { self.current_tab = Tab::Testing; }
                if ui.selectable_label(self.current_tab == Tab::History, "4. History").clicked() { self.current_tab = Tab::History; }
            });
            ui.separator();

            match self.current_tab {
                // --- TAB 1: MOLECULES ---
                Tab::Molecules => {
                    ui.heading("Register New Chemical");
                    egui::Grid::new("mol_grid").show(ui, |ui| {
                        ui.label("Name:"); ui.text_edit_singleline(&mut self.new_mol_name); ui.end_row();
                        ui.label("Short:"); ui.text_edit_singleline(&mut self.new_mol_short); ui.end_row();
                        ui.label("CAS:"); ui.text_edit_singleline(&mut self.new_mol_cas); ui.end_row();
                        ui.label("MW:"); ui.text_edit_singleline(&mut self.new_mol_mw); ui.end_row();

                        ui.label("Supplier:"); ui.text_edit_singleline(&mut self.new_mol_supplier); ui.end_row();
                        ui.label("SKU:"); ui.text_edit_singleline(&mut self.new_mol_sku); ui.end_row();
                        ui.label("Webpage:"); ui.text_edit_singleline(&mut self.new_mol_link); ui.end_row();

                        ui.label("Role:");
                        ui.horizontal(|ui| {
                            ui.radio_value(&mut self.new_mol_role, Role::HBA, "Acceptor");
                            ui.radio_value(&mut self.new_mol_role, Role::HBD, "Donor");
                        }); ui.end_row();
                    });

                    if ui.button("Add Molecule").clicked() {
                        if let Ok(mw) = self.new_mol_mw.parse::<f64>() {
                            self.molecules.push(Molecule {
                                id: self.next_mol_id,
                                name: self.new_mol_name.clone(),
                                short_name: self.new_mol_short.clone(),
                                cas_number: self.new_mol_cas.clone(),
                                mw,
                                role: self.new_mol_role.clone(),
                                supplier: self.new_mol_supplier.clone(),
                                sku: self.new_mol_sku.clone(),
                                webpage: self.new_mol_link.clone(),
                            });
                            self.next_mol_id += 1;

                            self.new_mol_name.clear(); self.new_mol_short.clear(); self.new_mol_mw.clear();
                            self.new_mol_cas.clear(); self.new_mol_supplier.clear(); self.new_mol_sku.clear(); self.new_mol_link.clear();

                            self.save_to_disk();
                            self.show_status("Molecule Added");
                        }
                    }

                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.heading("Registered Molecules");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.text_edit_singleline(&mut self.search_query);
                            ui.label("🔍 Search:");
                        });
                    });

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        egui::Grid::new("mol_list_grid").striped(true).show(ui, |ui| {
                            ui.strong("Short");
                            ui.strong("CAS");
                            ui.strong("MW");
                            ui.strong("Role");
                            ui.strong("Supplier");
                            ui.strong("Action");
                            ui.end_row();

                            let mut delete_id = None;

                            let query = self.search_query.to_lowercase();
                            let filtered_mols = self.molecules.iter().filter(|m| {
                                query.is_empty()
                                    || m.name.to_lowercase().contains(&query)
                                    || m.short_name.to_lowercase().contains(&query)
                                    || m.cas_number.contains(&query)
                            });

                            for m in filtered_mols {
                                ui.label(&m.short_name);
                                ui.label(&m.cas_number);
                                ui.label(m.mw.to_string());
                                ui.label(match m.role { Role::HBA => "Acc", Role::HBD => "Don" });
                                ui.label(&m.supplier);

                                if ui.button("🗑").clicked() { delete_id = Some(m.id); }
                                ui.end_row();
                            }
                            if let Some(id) = delete_id {
                                self.molecules.retain(|m| m.id != id);
                                self.save_to_disk();
                            }
                        });
                    });
                }

                // --- TAB 2: SAMPLE PREPARATION (Weighing) ---
                Tab::SamplePrep => {
                    ui.heading("1. Weighing & Mixing");

                    ui.horizontal(|ui| {
                        ui.label("Sample Type:");
                        ui.radio_value(&mut self.prep_mode, SampleType::Binary, "Binary Mixture");
                        ui.radio_value(&mut self.prep_mode, SampleType::Pure, "Pure Component");
                    });
                    ui.separator();

                    if self.prep_mode == SampleType::Binary {
                        // --- BINARY MODE ---
                        let acc_opts: Vec<(u32, String)> = self.molecules.iter().filter(|m| m.role == Role::HBA).map(|m| (m.id, m.short_name.clone())).collect();
                        let don_opts: Vec<(u32, String)> = self.molecules.iter().filter(|m| m.role == Role::HBD).map(|m| (m.id, m.short_name.clone())).collect();

                        let cur_c1 = self.molecules.iter().find(|m| Some(m.id) == self.prep_c1_id).map(|m| m.short_name.clone()).unwrap_or("Select Acceptor".into());
                        let cur_c2 = self.molecules.iter().find(|m| Some(m.id) == self.prep_c2_id).map(|m| m.short_name.clone()).unwrap_or("Select Donor".into());

                        egui::Grid::new("bin_grid").show(ui, |ui| {
                            ui.label("Acceptor:");
                            egui::ComboBox::from_id_source("pc1").selected_text(cur_c1).show_ui(ui, |ui| {
                                for (id, n) in acc_opts { ui.selectable_value(&mut self.prep_c1_id, Some(id), n); }
                            });
                            ui.end_row();

                            ui.label("Donor:");
                            egui::ComboBox::from_id_source("pc2").selected_text(cur_c2).show_ui(ui, |ui| {
                                for (id, n) in don_opts { ui.selectable_value(&mut self.prep_c2_id, Some(id), n); }
                            });
                            ui.end_row();

                            ui.label("Molar Ratio:");
                            ui.horizontal(|ui| {
                                ui.label("Acc Parts:"); ui.add(egui::TextEdit::singleline(&mut self.prep_parts_c1).desired_width(40.0));
                                ui.label(" : Don Parts:"); ui.add(egui::TextEdit::singleline(&mut self.prep_parts_c2).desired_width(40.0));
                            });
                            ui.end_row();
                        });

                        // Calculation for Binary
                        let mut calc_done = false;
                        if let (Some(id1), Some(id2)) = (self.prep_c1_id, self.prep_c2_id) {
                            if let (Some(m1), Some(m2)) = (self.molecules.iter().find(|m| m.id == id1), self.molecules.iter().find(|m| m.id == id2)) {
                                let p1 = self.prep_parts_c1.parse::<f64>().unwrap_or(0.0);
                                let p2 = self.prep_parts_c2.parse::<f64>().unwrap_or(0.0);

                                if p1 > 0.0 {
                                    let target_ratio = p2 / p1;
                                    ui.horizontal(|ui| {
                                        ui.label("Mass Acceptor (mg):");
                                        ui.text_edit_singleline(&mut self.prep_mass_c1);
                                    });

                                    if let Ok(mass1) = self.prep_mass_c1.parse::<f64>() {
                                        let req_mass2 = (mass1 / m1.mw) * target_ratio * m2.mw;
                                        ui.colored_label(egui::Color32::LIGHT_BLUE, format!("👉 Add {:.2} mg of {}", req_mass2, m2.short_name));
                                        calc_done = true;
                                    }
                                }
                            }
                        }

                        if calc_done {
                            ui.separator();
                            ui.label("Record Actual Values:");
                            ui.horizontal(|ui| {
                                ui.label("Actual Mass Donor (mg):");
                                ui.text_edit_singleline(&mut self.prep_mass_c2_actual);
                            });

                            if ui.button("💾 Save Binary Sample").clicked() {
                                let m1 = self.molecules.iter().find(|m| Some(m.id) == self.prep_c1_id).unwrap();
                                let m2 = self.molecules.iter().find(|m| Some(m.id) == self.prep_c2_id).unwrap();
                                let mass1 = self.prep_mass_c1.parse::<f64>().unwrap_or(0.0);
                                let mass2 = self.prep_mass_c2_actual.parse::<f64>().unwrap_or(0.0);

                                let moles1 = mass1 / m1.mw;
                                let moles2 = mass2 / m2.mw;
                                let actual_r = if moles1 > 0.0 { moles2 / moles1 } else { 0.0 };

                                let s = Sample {
                                    id: self.next_sample_id,
                                    type_: SampleType::Binary,
                                    c1_id: m1.id, c1_name: m1.short_name.clone(), c1_mass: mass1,
                                    c2_id: Some(m2.id), c2_name: Some(m2.short_name.clone()), c2_mass: Some(mass2),
                                    ratio_label: format!("{}:{}", self.prep_parts_c1, self.prep_parts_c2),
                                    real_molar_ratio: Some(actual_r),
                                    created_at: Utc::now(),
                                };
                                self.samples.push(s);
                                self.next_sample_id += 1;
                                self.save_to_disk();
                                self.show_status("Binary Sample Saved!");
                                self.prep_mass_c2_actual.clear();
                            }
                        }

                    } else {
                        // --- PURE MODE ---
                        let all_opts: Vec<(u32, String)> = self.molecules.iter().map(|m| (m.id, m.short_name.clone())).collect();
                        let cur_c1 = self.molecules.iter().find(|m| Some(m.id) == self.prep_c1_id).map(|m| m.short_name.clone()).unwrap_or("Select Chemical".into());

                        egui::Grid::new("pure_grid").show(ui, |ui| {
                            ui.label("Chemical:");
                            egui::ComboBox::from_id_source("pure_c1").selected_text(cur_c1).show_ui(ui, |ui| {
                                for (id, n) in all_opts { ui.selectable_value(&mut self.prep_c1_id, Some(id), n); }
                            });
                            ui.end_row();

                            ui.label("Mass (mg):");
                            ui.text_edit_singleline(&mut self.prep_mass_c1);
                            ui.end_row();
                        });

                        if ui.button("💾 Save Pure Sample").clicked() {
                            if let Some(id) = self.prep_c1_id {
                                if let Some(m) = self.molecules.iter().find(|m| m.id == id) {
                                    let mass = self.prep_mass_c1.parse::<f64>().unwrap_or(0.0);
                                    let s = Sample {
                                        id: self.next_sample_id,
                                        type_: SampleType::Pure,
                                        c1_id: m.id, c1_name: m.short_name.clone(), c1_mass: mass,
                                        c2_id: None, c2_name: None, c2_mass: None,
                                        ratio_label: "Pure".to_string(),
                                        real_molar_ratio: None,
                                        created_at: Utc::now(),
                                    };
                                    self.samples.push(s);
                                    self.next_sample_id += 1;
                                    self.save_to_disk();
                                    self.show_status("Pure Sample Saved!");
                                }
                            }
                        }
                    }

                    ui.separator();
                    ui.heading("2. Prepared Samples List");
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        egui::Grid::new("sample_list_grid").striped(true).show(ui, |ui| {
                            ui.strong("ID");
                            ui.strong("Composition");
                            ui.strong("Ratio info");
                            ui.strong("Masses");
                            ui.strong("Action");
                            ui.end_row();

                            let mut del_samp = None;
                            for s in self.samples.iter().rev() {
                                ui.label(format!("#{}", s.id));
                                match s.type_ {
                                    SampleType::Binary => {
                                        ui.label(format!("{} + {}", s.c1_name, s.c2_name.as_deref().unwrap_or("?")));
                                        ui.label(format!("(Target {}) Real 1:{:.3}", s.ratio_label, s.real_molar_ratio.unwrap_or(0.0)));
                                        ui.label(format!("{:.0} / {:.0} mg", s.c1_mass, s.c2_mass.unwrap_or(0.0)));
                                    },
                                    SampleType::Pure => {
                                        ui.label(format!("Pure {}", s.c1_name));
                                        ui.label("Pure");
                                        ui.label(format!("{:.0} mg", s.c1_mass));
                                    }
                                }
                                if ui.button("🗑").clicked() { del_samp = Some(s.id); }
                                ui.end_row();
                            }
                            if let Some(id) = del_samp {
                                self.samples.retain(|s| s.id != id);
                                self.save_to_disk();
                            }
                        });
                    });
                }

                // --- TAB 3: TESTING ---
                Tab::Testing => {
                    ui.heading("Test a Sample");

                    let current_sample_txt = self.samples.iter()
                        .find(|s| Some(s.id) == self.test_selected_sample_id)
                        .map(|s| match s.type_ {
                            SampleType::Binary => format!("#{} {} + {}", s.id, s.c1_name, s.c2_name.as_deref().unwrap_or("")),
                            SampleType::Pure => format!("#{} Pure {}", s.id, s.c1_name),
                        })
                        .unwrap_or("Select Sample".to_string());

                    egui::ComboBox::from_id_source("samp_sel").selected_text(current_sample_txt).show_ui(ui, |ui| {
                        for s in self.samples.iter().rev() {
                            let label = match s.type_ {
                                SampleType::Binary => format!("#{} {}+{}", s.id, s.c1_name, s.c2_name.as_deref().unwrap_or("")),
                                SampleType::Pure => format!("#{} Pure {}", s.id, s.c1_name),
                            };
                            ui.selectable_value(&mut self.test_selected_sample_id, Some(s.id), label);
                        }
                    });

                    if let Some(sid) = self.test_selected_sample_id {
                        if let Some(sample) = self.samples.iter().find(|s| s.id == sid) {
                            ui.separator();
                            match sample.type_ {
                                SampleType::Binary => ui.label(format!("Testing: {} + {} (Real Ratio 1:{:.3})", sample.c1_name, sample.c2_name.as_deref().unwrap_or(""), sample.real_molar_ratio.unwrap_or(0.0))),
                                SampleType::Pure => ui.label(format!("Testing: Pure {}", sample.c1_name)),
                            };

                            egui::Grid::new("test_grid").show(ui, |ui| {
                                ui.label("Temperature (°C):");
                                ui.text_edit_singleline(&mut self.test_temp);
                                ui.end_row();

                                ui.label("Result:");
                                ui.horizontal(|ui| {
                                    ui.radio_value(&mut self.test_result, Phase::Liquid, "Liquid");
                                    ui.radio_value(&mut self.test_result, Phase::Solid, "Solid");
                                });
                                ui.end_row();
                            });

                            if ui.button("Record Result").clicked() {
                                if let Ok(t) = self.test_temp.parse::<f64>() {
                                    self.observations.push(Observation {
                                        id: self.next_obs_id,
                                        sample_id: sid,
                                        temperature: t,
                                        result: self.test_result.clone(),
                                        timestamp: Utc::now(),
                                    });
                                    self.next_obs_id += 1;
                                    self.save_to_disk();
                                    self.show_status("Result Recorded!");
                                }
                            }
                        }
                    }
                }

                // --- TAB 4: HISTORY ---
                Tab::History => {
                    ui.horizontal(|ui| {
                        ui.heading("Experiment Log");
                        if ui.button("📂 Export to CSV").clicked() {
                            if let Err(e) = self.export_to_csv() {
                                self.show_status(&format!("Export Error: {}", e));
                            } else {
                                self.show_status("Exported to experiment_data.csv");
                            }
                        }
                    });

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        egui::Grid::new("hist_grid").striped(true).show(ui, |ui| {
                            ui.strong("Sample");
                            ui.strong("Composition");
                            ui.strong("Temp");
                            ui.strong("Result");
                            ui.strong("Action");
                            ui.end_row();

                            let mut del_obs = None;
                            for obs in self.observations.iter().rev() {
                                if let Some(samp) = self.samples.iter().find(|s| s.id == obs.sample_id) {
                                    ui.label(format!("#{}", samp.id));
                                    match samp.type_ {
                                        SampleType::Binary => ui.label(format!("{}+{} ({})", samp.c1_name, samp.c2_name.as_deref().unwrap_or(""), samp.ratio_label)),
                                        SampleType::Pure => ui.label(format!("Pure {}", samp.c1_name)),
                                    };
                                    ui.label(format!("{}°C", obs.temperature));
                                    ui.colored_label(
                                        if obs.result == Phase::Liquid { egui::Color32::GREEN } else { egui::Color32::RED },
                                        format!("{:?}", obs.result)
                                    );
                                    if ui.button("🗑").clicked() { del_obs = Some(obs.id); }
                                    ui.end_row();
                                }
                            }
                            if let Some(id) = del_obs {
                                self.observations.retain(|o| o.id != id);
                                self.save_to_disk();
                            }
                        });
                    });
                }
            }
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Eutectic Lab Notebook",
        options,
        Box::new(|_cc| Box::new(LabApp::load_from_disk())),
    )
}