use chrono::{DateTime, Utc};
use eframe::egui;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::{fs, path::Path};
use rfd::FileDialog;

const DEFAULT_MOLECULES_FILE: &str = "molecules.json";
const DEFAULT_SAMPLES_FILE: &str = "samples.json";
const DEFAULT_OBSERVATIONS_FILE: &str = "observations.json";
const DEFAULT_CSV_FILE: &str = "experiment_data.csv";
const STATUS_SECONDS: i64 = 4;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
enum Role {
    HBA,
    HBD,
}

impl Default for Role {
    fn default() -> Self {
        Self::HBA
    }
}

impl Role {
    fn label(&self) -> &'static str {
        match self {
            Role::HBA => "Acceptor (HBA)",
            Role::HBD => "Donor (HBD)",
        }
    }

    fn short(&self) -> &'static str {
        match self {
            Role::HBA => "HBA",
            Role::HBD => "HBD",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
enum Phase {
    Liquid,
    Solid,
}

impl Default for Phase {
    fn default() -> Self {
        Self::Liquid
    }
}

impl Phase {
    fn label(&self) -> &'static str {
        match self {
            Phase::Liquid => "Liquid",
            Phase::Solid => "Solid",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
enum SampleType {
    Binary,
    Pure,
}

impl Default for SampleType {
    fn default() -> Self {
        Self::Binary
    }
}

impl SampleType {
    fn label(&self) -> &'static str {
        match self {
            SampleType::Binary => "Binary",
            SampleType::Pure => "Pure",
        }
    }
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

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Sample {
    id: u32,
    #[serde(rename = "type")]
    type_: SampleType,

    c1_id: u32,
    c1_name: String,
    c1_mass: f64,

    c2_id: Option<u32>,
    c2_name: Option<String>,
    c2_mass: Option<f64>,

    ratio_label: String,
    real_molar_ratio: Option<f64>,

    created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Observation {
    id: u32,
    sample_id: u32,
    temperature: f64,
    result: Phase,
    timestamp: DateTime<Utc>,
    #[serde(default)]
    note: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MoleculesData {
    molecules: Vec<Molecule>,
    next_mol_id: u32,
}

impl Default for MoleculesData {
    fn default() -> Self {
        Self {
            molecules: Vec::new(),
            next_mol_id: 1,
        }
    }
}

impl MoleculesData {
    fn normalise_ids(&mut self) {
        let max_mol = self.molecules.iter().map(|m| m.id).max().unwrap_or(0);
        if self.next_mol_id <= max_mol {
            self.next_mol_id = max_mol + 1;
        }
    }

    fn alloc_mol_id(&mut self) -> u32 {
        let id = self.next_mol_id;
        self.next_mol_id += 1;
        id
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct SamplesData {
    samples: Vec<Sample>,
    next_sample_id: u32,
}

impl Default for SamplesData {
    fn default() -> Self {
        Self {
            samples: Vec::new(),
            next_sample_id: 1,
        }
    }
}

impl SamplesData {
    fn normalise_ids(&mut self) {
        let max_sample = self.samples.iter().map(|s| s.id).max().unwrap_or(0);
        if self.next_sample_id <= max_sample {
            self.next_sample_id = max_sample + 1;
        }
    }

    fn alloc_sample_id(&mut self) -> u32 {
        let id = self.next_sample_id;
        self.next_sample_id += 1;
        id
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ObservationsData {
    observations: Vec<Observation>,
    next_obs_id: u32,
}

impl Default for ObservationsData {
    fn default() -> Self {
        Self {
            observations: Vec::new(),
            next_obs_id: 1,
        }
    }
}

impl ObservationsData {
    fn normalise_ids(&mut self) {
        let max_obs = self.observations.iter().map(|o| o.id).max().unwrap_or(0);
        if self.next_obs_id <= max_obs {
            self.next_obs_id = max_obs + 1;
        }
    }

    fn alloc_obs_id(&mut self) -> u32 {
        let id = self.next_obs_id;
        self.next_obs_id += 1;
        id
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Molecules,
    Weighing,
    Testing,
    History,
}

impl Default for Tab {
    fn default() -> Self {
        Self::Molecules
    }
}

impl Tab {
    fn label(&self) -> &'static str {
        match self {
            Tab::Molecules => "1. Molecules",
            Tab::Weighing => "2. Weighting",
            Tab::Testing => "3. Testing",
            Tab::History => "4. History & Export",
        }
    }
}

#[derive(Clone, Copy)]
enum StatusKind {
    Ok,
    Error,
}

struct StatusMessage {
    text: String,
    kind: StatusKind,
    at: DateTime<Utc>,
}

struct UiState {
    current_tab: Tab,
    search_query: String,

    molecules_file: String,
    samples_file: String,
    observations_file: String,

    new_mol_name: String,
    new_mol_short: String,
    new_mol_cas: String,
    new_mol_mw: String,
    new_mol_role: Role,
    new_mol_supplier: String,
    new_mol_sku: String,
    new_mol_link: String,

    prep_mode: SampleType,
    prep_c1_id: Option<u32>,
    prep_c2_id: Option<u32>,
    prep_parts_c1: String,
    prep_parts_c2: String,
    prep_mass_c1: String,
    prep_mass_c2_actual: String,
    prep_theoretical_text: String,

    test_selected_sample_id: Option<u32>,
    test_temp: String,
    test_result: Phase,
    test_note: String,
    export_selected_obs_ids: Vec<u32>,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            current_tab: Tab::Molecules,
            search_query: String::new(),

            molecules_file: DEFAULT_MOLECULES_FILE.to_string(),
            samples_file: DEFAULT_SAMPLES_FILE.to_string(),
            observations_file: DEFAULT_OBSERVATIONS_FILE.to_string(),

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
            prep_parts_c2: "1".to_string(),
            prep_mass_c1: String::new(),
            prep_mass_c2_actual: String::new(),
            prep_theoretical_text: String::new(),

            test_selected_sample_id: None,
            test_temp: "25.0".to_string(),
            test_result: Phase::Liquid,
            test_note: String::new(),
            export_selected_obs_ids: Vec::new(),
        }
    }
}

struct LabApp {
    molecules_data: MoleculesData,
    samples_data: SamplesData,
    observations_data: ObservationsData,
    ui: UiState,
    status: Option<StatusMessage>,
}

impl LabApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::light());
        Self::load()
    }

    fn load() -> Self {
        let ui = UiState::default();
        let mut molecules_data = match fs::read_to_string(&ui.molecules_file) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => MoleculesData::default(),
        };
        molecules_data.normalise_ids();

        let mut samples_data = match fs::read_to_string(&ui.samples_file) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => SamplesData::default(),
        };
        samples_data.normalise_ids();

        let mut observations_data = match fs::read_to_string(&ui.observations_file) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => ObservationsData::default(),
        };
        observations_data.normalise_ids();

        let app = Self {
            molecules_data,
            samples_data,
            observations_data,
            ui,
            status: None,
        };
        app.save_all_data();
        app
    }

    fn save_molecules(&self) {
        if let Ok(json) = serde_json::to_string_pretty(&self.molecules_data) {
            let _ = fs::write(&self.ui.molecules_file, json);
        }
    }

    fn save_samples(&self) {
        if let Ok(json) = serde_json::to_string_pretty(&self.samples_data) {
            let _ = fs::write(&self.ui.samples_file, json);
        }
    }

    fn save_observations(&self) {
        if let Ok(json) = serde_json::to_string_pretty(&self.observations_data) {
            let _ = fs::write(&self.ui.observations_file, json);
        }
    }

    fn save_all_data(&self) {
        self.save_molecules();
        self.save_samples();
        self.save_observations();
    }

    fn load_molecules(&mut self) {
        self.molecules_data = match fs::read_to_string(&self.ui.molecules_file) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => MoleculesData::default(),
        };
        self.molecules_data.normalise_ids();
    }

    fn load_samples(&mut self) {
        self.samples_data = match fs::read_to_string(&self.ui.samples_file) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => SamplesData::default(),
        };
        self.samples_data.normalise_ids();
    }

    fn load_observations(&mut self) {
        self.observations_data = match fs::read_to_string(&self.ui.observations_file) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => ObservationsData::default(),
        };
        self.observations_data.normalise_ids();
    }

    fn show_status(&mut self, msg: impl Into<String>, kind: StatusKind) {
        self.status = Some(StatusMessage {
            text: msg.into(),
            kind,
            at: Utc::now(),
        });
    }

    fn status_ui(&mut self, ui: &mut egui::Ui) {
        let mut clear_status = false;
        if let Some(status) = &self.status {
            if (Utc::now() - status.at).num_seconds() <= STATUS_SECONDS {
                let color = match status.kind {
                    StatusKind::Ok => egui::Color32::from_rgb(40, 140, 70),
                    StatusKind::Error => egui::Color32::from_rgb(180, 60, 60),
                };
                ui.colored_label(color, status.text.clone());
            } else {
                clear_status = true;
            }
        }
        if clear_status {
            self.status = None;
        }
    }

    fn parse_f64(input: &str) -> Option<f64> {
        input.trim().parse::<f64>().ok()
    }

    fn molecules_by_role(&self, role: Role) -> Vec<(u32, String)> {
        self.molecules_data
            .molecules
            .iter()
            .filter(|m| m.role == role)
            .map(|m| (m.id, m.short_name.clone()))
            .collect()
    }

    fn sample_label(sample: &Sample) -> String {
        match sample.type_ {
            SampleType::Binary => format!(
                "#{} {} + {} ({})",
                sample.id,
                sample.c1_name,
                sample.c2_name.as_deref().unwrap_or("?"),
                sample.ratio_label
            ),
            SampleType::Pure => format!("#{} Pure {}", sample.id, sample.c1_name),
        }
    }

    fn real_ratio_display(sample: &Sample) -> String {
        if let SampleType::Binary = sample.type_ {
            if let (Some(real), Some((first, _second))) = (
                sample.real_molar_ratio,
                sample
                    .ratio_label
                    .split_once(':')
                    .map(|(a, b)| (a.trim().to_string(), b.trim().to_string())),
            ) {
                if let Ok(first_val) = first.parse::<f64>() {
                    let real_second = real * first_val;
                    return format!("{first}:{real_second:.3}");
                }
            }
        }
        "N/A".to_string()
    }

    fn file_selector_row(ui: &mut egui::Ui, current_path: &mut String) -> bool {
        let mut changed = false;
        if ui.button("Change file…").clicked() {
            if let Some(path) = FileDialog::new()
                .add_filter("JSON", &["json"])
                .set_file_name(current_path)
                .pick_file()
            {
                *current_path = path.to_string_lossy().to_string();
                changed = true;
            }
        }
        ui.label("File:");
        ui.text_edit_singleline(current_path);
        changed
    }

    fn add_molecule(&mut self) {
        let name = self.ui.new_mol_name.trim();
        let short = self.ui.new_mol_short.trim();
        let cas = self.ui.new_mol_cas.trim();
        let mw = Self::parse_f64(&self.ui.new_mol_mw);

        if name.is_empty() {
            self.show_status("Name is required.", StatusKind::Error);
            return;
        }
        if short.is_empty() {
            self.show_status("Short name is required.", StatusKind::Error);
            return;
        }
        let mw = match mw {
            Some(value) if value > 0.0 => value,
            _ => {
                self.show_status("MW must be a positive number.", StatusKind::Error);
                return;
            }
        };

        let molecule = Molecule {
            id: self.molecules_data.alloc_mol_id(),
            name: name.to_string(),
            short_name: short.to_string(),
            cas_number: cas.to_string(),
            mw,
            role: self.ui.new_mol_role,
            supplier: self.ui.new_mol_supplier.trim().to_string(),
            sku: self.ui.new_mol_sku.trim().to_string(),
            webpage: self.ui.new_mol_link.trim().to_string(),
        };

        self.molecules_data.molecules.push(molecule);
        self.save_molecules();
        self.show_status("Molecule added.", StatusKind::Ok);

        self.ui.new_mol_name.clear();
        self.ui.new_mol_short.clear();
        self.ui.new_mol_cas.clear();
        self.ui.new_mol_mw.clear();
        self.ui.new_mol_supplier.clear();
        self.ui.new_mol_sku.clear();
        self.ui.new_mol_link.clear();
    }

    fn add_binary_sample(&mut self) {
        let c1_id = match self.ui.prep_c1_id {
            Some(id) => id,
            None => {
                self.show_status("Select an acceptor.", StatusKind::Error);
                return;
            }
        };
        let c2_id = match self.ui.prep_c2_id {
            Some(id) => id,
            None => {
                self.show_status("Select a donor.", StatusKind::Error);
                return;
            }
        };
        if c1_id == c2_id {
            self.show_status("Select two different molecules.", StatusKind::Error);
            return;
        }

        let (c1_id, c1_name, c1_mw) = match self.molecules_data.molecules.iter().find(|m| m.id == c1_id)
        {
            Some(m) => (m.id, m.short_name.clone(), m.mw),
            None => {
                self.show_status("Acceptor not found.", StatusKind::Error);
                return;
            }
        };
        let (c2_id, c2_name, c2_mw) = match self.molecules_data.molecules.iter().find(|m| m.id == c2_id)
        {
            Some(m) => (m.id, m.short_name.clone(), m.mw),
            None => {
                self.show_status("Donor not found.", StatusKind::Error);
                return;
            }
        };

        let mass1 = match Self::parse_f64(&self.ui.prep_mass_c1) {
            Some(value) if value > 0.0 => value,
            _ => {
                self.show_status("Mass 1 must be a positive number.", StatusKind::Error);
                return;
            }
        };
        let mass2 = match Self::parse_f64(&self.ui.prep_mass_c2_actual) {
            Some(value) if value > 0.0 => value,
            _ => {
                self.show_status("Mass 2 must be a positive number.", StatusKind::Error);
                return;
            }
        };

        let ratio_label = format!(
            "{}:{}",
            self.ui.prep_parts_c1.trim(),
            self.ui.prep_parts_c2.trim()
        );

        let moles1 = mass1 / c1_mw;
        let moles2 = mass2 / c2_mw;
        let real_ratio = if moles1 > 0.0 { Some(moles2 / moles1) } else { None };

        let sample_id = self.samples_data.alloc_sample_id();
        let sample = Sample {
            id: sample_id,
            type_: SampleType::Binary,
            c1_id,
            c1_name,
            c1_mass: mass1,
            c2_id: Some(c2_id),
            c2_name: Some(c2_name),
            c2_mass: Some(mass2),
            ratio_label,
            real_molar_ratio: real_ratio,
            created_at: Utc::now(),
        };

        self.samples_data.samples.push(sample);
        self.save_samples();
        self.show_status("Binary sample saved.", StatusKind::Ok);
        self.ui.prep_mass_c2_actual.clear();
    }

    fn add_pure_sample(&mut self) {
        let c1_id = match self.ui.prep_c1_id {
            Some(id) => id,
            None => {
                self.show_status("Select a molecule.", StatusKind::Error);
                return;
            }
        };
        let (c1_id, c1_name) = match self.molecules_data.molecules.iter().find(|m| m.id == c1_id) {
            Some(m) => (m.id, m.short_name.clone()),
            None => {
                self.show_status("Selected molecule not found.", StatusKind::Error);
                return;
            }
        };
        let mass = match Self::parse_f64(&self.ui.prep_mass_c1) {
            Some(value) if value > 0.0 => value,
            _ => {
                self.show_status("Mass must be a positive number.", StatusKind::Error);
                return;
            }
        };

        let sample_id = self.samples_data.alloc_sample_id();
        let sample = Sample {
            id: sample_id,
            type_: SampleType::Pure,
            c1_id,
            c1_name,
            c1_mass: mass,
            c2_id: None,
            c2_name: None,
            c2_mass: None,
            ratio_label: "Pure".to_string(),
            real_molar_ratio: None,
            created_at: Utc::now(),
        };

        self.samples_data.samples.push(sample);
        self.save_samples();
        self.show_status("Pure sample saved.", StatusKind::Ok);
    }

    fn add_observation(&mut self, sample_id: u32) {
        let temp = match Self::parse_f64(&self.ui.test_temp) {
            Some(value) => value,
            None => {
                self.show_status("Temperature must be a number.", StatusKind::Error);
                return;
            }
        };

        let obs_id = self.observations_data.alloc_obs_id();
        let note = self.ui.test_note.trim();
        self.observations_data.observations.push(Observation {
            id: obs_id,
            sample_id,
            temperature: temp,
            result: self.ui.test_result,
            timestamp: Utc::now(),
            note: if note.is_empty() {
                None
            } else {
                Some(note.to_string())
            },
        });
        self.save_observations();
        self.show_status("Observation recorded.", StatusKind::Ok);
    }

    fn export_to_csv(
        &self,
        only_obs_ids: Option<&[u32]>,
        path: &Path,
    ) -> Result<(), Box<dyn Error>> {
        let mut wtr = csv::Writer::from_path(path)?;
        wtr.write_record(&[
            "Date",
            "Obs ID",
            "Sample ID",
            "Type",
            "Component 1",
            "Mass 1 (mg)",
            "Component 2",
            "Mass 2 (mg)",
            "Target Ratio",
            "Real Ratio",
            "Temp (C)",
            "Result",
            "Note",
        ])?;

        for obs in &self.observations_data.observations {
            if let Some(filter_ids) = only_obs_ids {
                if !filter_ids.contains(&obs.id) {
                    continue;
                }
            }
            let date_str = obs.timestamp.format("%Y-%m-%d %H:%M").to_string();
            let temp_str = format!("{:.2}", obs.temperature);
            let result_str = obs.result.label().to_string();

            let row = if let Some(sample) = self.samples_data.samples.iter().find(|s| s.id == obs.sample_id)
            {
                let type_str = sample.type_.label().to_string();
                let c2_name = sample.c2_name.clone().unwrap_or_default();
                let c2_mass = sample
                    .c2_mass
                    .map(|m| format!("{:.3}", m))
                    .unwrap_or_default();
                let target_ratio = sample.ratio_label.clone();
                let real_ratio = Self::real_ratio_display(sample);

                vec![
                    date_str,
                    obs.id.to_string(),
                    sample.id.to_string(),
                    type_str,
                    sample.c1_name.clone(),
                    format!("{:.3}", sample.c1_mass),
                    c2_name,
                    c2_mass,
                    target_ratio,
                    real_ratio,
                    temp_str,
                    result_str,
                    obs.note.clone().unwrap_or_default(),
                ]
            } else {
                vec![
                    date_str,
                    obs.id.to_string(),
                    obs.sample_id.to_string(),
                    "Unknown".to_string(),
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                    temp_str,
                    result_str,
                    obs.note.clone().unwrap_or_default(),
                ]
            };

            wtr.write_record(row)?;
        }

        wtr.flush()?;
        Ok(())
    }

    fn tab_header(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("Eutectic Lab Notebook");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                self.status_ui(ui);
            });
        });

        ui.add_space(4.0);
        ui.horizontal(|ui| {
            for tab in [Tab::Molecules, Tab::Weighing, Tab::Testing, Tab::History] {
                if ui
                    .selectable_label(self.ui.current_tab == tab, tab.label())
                    .clicked()
                {
                    self.ui.current_tab = tab;
                }
            }
        });
        ui.separator();
    }

    fn tab_molecules(&mut self, ui: &mut egui::Ui) {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Molecules");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if Self::file_selector_row(ui, &mut self.ui.molecules_file) {
                        self.load_molecules();
                        self.ui.prep_c1_id = None;
                        self.ui.prep_c2_id = None;
                        self.ui.prep_theoretical_text.clear();
                        self.show_status("Molecules file loaded.", StatusKind::Ok);
                    }
                });
            });
            ui.separator();
            ui.heading("Register New Molecule");
            ui.add_space(6.0);

            egui::Grid::new("mol_form")
                .num_columns(2)
                .spacing([12.0, 6.0])
                .show(ui, |ui| {
                    ui.label("Name");
                    ui.text_edit_singleline(&mut self.ui.new_mol_name);
                    ui.end_row();

                    ui.label("Short name");
                    ui.text_edit_singleline(&mut self.ui.new_mol_short);
                    ui.end_row();

                    ui.label("CAS");
                    ui.text_edit_singleline(&mut self.ui.new_mol_cas);
                    ui.end_row();

                    ui.label("Molecular weight");
                    ui.text_edit_singleline(&mut self.ui.new_mol_mw);
                    ui.end_row();

                    ui.label("Role");
                    ui.horizontal(|ui| {
                        ui.radio_value(&mut self.ui.new_mol_role, Role::HBA, Role::HBA.label());
                        ui.radio_value(&mut self.ui.new_mol_role, Role::HBD, Role::HBD.label());
                    });
                    ui.end_row();

                    ui.label("Supplier");
                    ui.text_edit_singleline(&mut self.ui.new_mol_supplier);
                    ui.end_row();

                    ui.label("SKU");
                    ui.text_edit_singleline(&mut self.ui.new_mol_sku);
                    ui.end_row();

                    ui.label("Webpage");
                    ui.text_edit_singleline(&mut self.ui.new_mol_link);
                    ui.end_row();
                });

            ui.add_space(6.0);
            if ui.button("Add molecule").clicked() {
                self.add_molecule();
            }
        });

        ui.add_space(12.0);
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Registered Molecules");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.ui.search_query)
                            .hint_text("Search name, short name, or CAS"),
                    );
                });
            });
            ui.add_space(6.0);

            let query = self.ui.search_query.trim().to_lowercase();
            egui::ScrollArea::vertical().show(ui, |ui| {
                egui::Grid::new("mol_list")
                    .striped(true)
                    .spacing([12.0, 6.0])
                    .show(ui, |ui| {
                        ui.strong("Short");
                        ui.strong("CAS");
                        ui.strong("MW");
                        ui.strong("Role");
                        ui.strong("Supplier");
                        ui.strong("Webpage");
                        ui.strong("Action");
                        ui.end_row();

                        let mut delete_id = None;
                        for mol in self.molecules_data.molecules.iter().filter(|m| {
                            query.is_empty()
                                || m.name.to_lowercase().contains(&query)
                                || m.short_name.to_lowercase().contains(&query)
                                || m.cas_number.to_lowercase().contains(&query)
                        }) {
                            ui.label(&mol.short_name);
                            ui.label(&mol.cas_number);
                            ui.label(format!("{:.3}", mol.mw));
                            ui.label(mol.role.short());
                            ui.label(&mol.supplier);
                            if !mol.webpage.is_empty() {
                                ui.hyperlink_to(&mol.webpage, &mol.webpage);
                            } else {
                                ui.label("-");
                            }
                            if ui.button("Delete").clicked() {
                                delete_id = Some(mol.id);
                            }
                            ui.end_row();
                        }

                        if let Some(id) = delete_id {
                            self.molecules_data.molecules.retain(|m| m.id != id);
                            self.save_molecules();
                            self.show_status("Molecule deleted.", StatusKind::Ok);
                        }
                    });
            });
        });
    }

    fn tab_weighing(&mut self, ui: &mut egui::Ui) {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Sample Preparation / Weighting");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if Self::file_selector_row(ui, &mut self.ui.samples_file) {
                        self.load_samples();
                        self.ui.test_selected_sample_id = None;
                        self.show_status("Samples file loaded.", StatusKind::Ok);
                    }
                });
            });
            ui.separator();
            ui.add_space(6.0);

            ui.horizontal(|ui| {
                ui.label("Mode");
                ui.radio_value(&mut self.ui.prep_mode, SampleType::Binary, "Binary mixture");
                ui.radio_value(&mut self.ui.prep_mode, SampleType::Pure, "Pure component");
            });

            ui.add_space(10.0);

            match self.ui.prep_mode {
                SampleType::Binary => {
                    let acc_opts = self.molecules_by_role(Role::HBA);
                    let don_opts = self.molecules_by_role(Role::HBD);

                    let acc_label = self
                        .molecules_data
                        .molecules
                        .iter()
                        .find(|m| Some(m.id) == self.ui.prep_c1_id)
                        .map(|m| m.short_name.clone())
                        .unwrap_or_else(|| "Select acceptor".to_string());
                    let don_label = self
                        .molecules_data
                        .molecules
                        .iter()
                        .find(|m| Some(m.id) == self.ui.prep_c2_id)
                        .map(|m| m.short_name.clone())
                        .unwrap_or_else(|| "Select donor".to_string());

                    egui::Grid::new("binary_form")
                        .spacing([12.0, 6.0])
                        .show(ui, |ui| {
                            ui.label("Acceptor");
                            egui::ComboBox::from_id_salt("prep_c1")
                                .selected_text(acc_label)
                                .show_ui(ui, |ui| {
                                    for (id, label) in acc_opts {
                                        ui.selectable_value(&mut self.ui.prep_c1_id, Some(id), label);
                                    }
                                });
                            ui.end_row();

                            ui.label("Donor");
                            egui::ComboBox::from_id_salt("prep_c2")
                                .selected_text(don_label)
                                .show_ui(ui, |ui| {
                                    for (id, label) in don_opts {
                                        ui.selectable_value(&mut self.ui.prep_c2_id, Some(id), label);
                                    }
                                });
                            ui.end_row();

                            ui.label("Molar ratio");
                            ui.horizontal(|ui| {
                                ui.label("Acceptor parts");
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.ui.prep_parts_c1)
                                        .desired_width(40.0),
                                );
                                ui.label(": Donor parts");
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.ui.prep_parts_c2)
                                        .desired_width(40.0),
                                );
                            });
                            ui.end_row();

                            ui.label("Mass component 1 (mg)");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.ui.prep_mass_c1)
                                    .desired_width(140.0),
                            );
                            ui.end_row();
                        });

                    let mut new_theoretical: Option<String> = None;
                    if let (Some(c1_id), Some(c2_id)) = (self.ui.prep_c1_id, self.ui.prep_c2_id)
                    {
                        if let (Some(c1), Some(c2)) = (
                            self.molecules_data.molecules.iter().find(|m| m.id == c1_id),
                            self.molecules_data.molecules.iter().find(|m| m.id == c2_id),
                        ) {
                            let p1 = Self::parse_f64(&self.ui.prep_parts_c1);
                            let p2 = Self::parse_f64(&self.ui.prep_parts_c2);
                            let m1 = Self::parse_f64(&self.ui.prep_mass_c1);
                            let m2 = Self::parse_f64(&self.ui.prep_mass_c2_actual);

                            if let (Some(p1), Some(p2)) = (p1, p2) {
                                if p1 > 0.0 && p2 > 0.0 {
                                    let m1_pos = m1.unwrap_or(0.0) > 0.0;
                                    let m2_pos = m2.unwrap_or(0.0) > 0.0;

                                    // Only suggest when exactly one mass is set.
                                    if m1_pos ^ m2_pos {
                                        if m1_pos {
                                            let mass1 = m1.unwrap();
                                            let ratio = p2 / p1;
                                            let mass2 = (mass1 / c1.mw) * ratio * c2.mw;
                                            new_theoretical = Some(format!(
                                                "Theoretical mass for {} (component 2) is {:.2} mg.",
                                                c2.short_name, mass2
                                            ));
                                        } else if m2_pos {
                                            let mass2 = m2.unwrap();
                                            let inv_ratio = p1 / p2;
                                            let mass1 = (mass2 / c2.mw) * inv_ratio * c1.mw;
                                            new_theoretical = Some(format!(
                                                "Theoretical mass for {} (component 1) is {:.2} mg.",
                                                c1.short_name, mass1
                                            ));
                                        }
                                    } else if !m1_pos && !m2_pos {
                                        // Neither mass entered – clear any old hint.
                                        new_theoretical = Some(String::new());
                                    }
                                }
                            }
                        }
                    }

                    if let Some(text) = new_theoretical {
                        self.ui.prep_theoretical_text = text;
                    }

                    ui.add_space(8.0);
                    ui.label(&self.ui.prep_theoretical_text);

                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label("Actual mass component 2 (mg)");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.ui.prep_mass_c2_actual)
                                .desired_width(140.0),
                        );
                    });

                    ui.add_space(6.0);
                    if ui.button("Save binary sample").clicked() {
                        self.add_binary_sample();
                    }
                }
                SampleType::Pure => {
                    let all_opts: Vec<(u32, String)> = self
                        .molecules_data
                        .molecules
                        .iter()
                        .map(|m| (m.id, m.short_name.clone()))
                        .collect();

                    let cur_label = self
                        .molecules_data
                        .molecules
                        .iter()
                        .find(|m| Some(m.id) == self.ui.prep_c1_id)
                        .map(|m| m.short_name.clone())
                        .unwrap_or_else(|| "Select molecule".to_string());

                    egui::Grid::new("pure_form")
                        .spacing([12.0, 6.0])
                        .show(ui, |ui| {
                            ui.label("Molecule");
                            egui::ComboBox::from_id_salt("prep_pure_c1")
                                .selected_text(cur_label)
                                .show_ui(ui, |ui| {
                                    for (id, label) in all_opts {
                                        ui.selectable_value(&mut self.ui.prep_c1_id, Some(id), label);
                                    }
                                });
                            ui.end_row();

                            ui.label("Mass (mg)");
                            ui.text_edit_singleline(&mut self.ui.prep_mass_c1);
                            ui.end_row();
                        });

                    ui.add_space(6.0);
                    if ui.button("Save pure sample").clicked() {
                        self.add_pure_sample();
                    }
                }
            }
        });

        ui.add_space(12.0);
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.heading("Prepared Samples");
            ui.add_space(6.0);

            egui::ScrollArea::vertical().show(ui, |ui| {
                egui::Grid::new("sample_list")
                    .striped(true)
                    .spacing([12.0, 6.0])
                    .show(ui, |ui| {
                        ui.strong("ID");
                        ui.strong("Composition");
                        ui.strong("Ratio");
                        ui.strong("Masses (mg)");
                        ui.strong("Action");
                        ui.end_row();

                        let mut delete_id = None;
                        for sample in self.samples_data.samples.iter().rev() {
                            ui.label(format!("#{}", sample.id));
                            match sample.type_ {
                                SampleType::Binary => {
                                    ui.label(format!(
                                        "{} + {}",
                                        sample.c1_name,
                                        sample.c2_name.as_deref().unwrap_or("?")
                                    ));
                                    let ratio_display = Self::real_ratio_display(sample);
                                    ui.label(format!(
                                        "Target {} | Real {}",
                                        sample.ratio_label, ratio_display
                                    ));
                                    ui.label(format!(
                                        "{:.2} / {:.2}",
                                        sample.c1_mass,
                                        sample.c2_mass.unwrap_or(0.0)
                                    ));
                                }
                                SampleType::Pure => {
                                    ui.label(format!("Pure {}", sample.c1_name));
                                    ui.label("Pure");
                                    ui.label(format!("{:.2}", sample.c1_mass));
                                }
                            };
                            if ui.button("Delete").clicked() {
                                delete_id = Some(sample.id);
                            }
                            ui.end_row();
                        }

                        if let Some(id) = delete_id {
                            self.samples_data.samples.retain(|s| s.id != id);
                            self.observations_data.observations.retain(|o| o.sample_id != id);
                            self.save_samples();
                            self.save_observations();
                            self.show_status("Sample deleted.", StatusKind::Ok);
                        }
                    });
            });
        });
    }

    fn tab_testing(&mut self, ui: &mut egui::Ui) {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Phase Observation");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if Self::file_selector_row(ui, &mut self.ui.observations_file) {
                        self.load_observations();
                        self.ui.export_selected_obs_ids.clear();
                        self.show_status("Observations file loaded.", StatusKind::Ok);
                    }
                });
            });
            ui.separator();
            ui.add_space(6.0);

            let selected_label = self
                .ui
                .test_selected_sample_id
                .and_then(|id| self.samples_data.samples.iter().find(|s| s.id == id))
                .map(|s| Self::sample_label(s))
                .unwrap_or_else(|| "Select sample".to_string());

            let sample_options: Vec<(u32, String)> = self
                .samples_data
                .samples
                .iter()
                .rev()
                .map(|sample| (sample.id, Self::sample_label(sample)))
                .collect();

            egui::ComboBox::from_id_salt("test_sample")
                .selected_text(selected_label)
                .show_ui(ui, |ui| {
                    for (id, label) in sample_options {
                        ui.selectable_value(&mut self.ui.test_selected_sample_id, Some(id), label);
                    }
                });

            ui.add_space(8.0);

            if let Some(sample_id) = self.ui.test_selected_sample_id {
                let summary = self
                    .samples_data
                    .samples
                    .iter()
                    .find(|s| s.id == sample_id)
                    .map(|s| Self::sample_label(s))
                    .unwrap_or_else(|| "Sample not found".to_string());
                ui.label(summary);

                ui.add_space(6.0);
                egui::Grid::new("test_form")
                    .spacing([12.0, 6.0])
                    .show(ui, |ui| {
                        ui.label("Temperature (C)");
                        ui.text_edit_singleline(&mut self.ui.test_temp);
                        ui.end_row();

                        ui.label("Result");
                        ui.horizontal(|ui| {
                            ui.radio_value(&mut self.ui.test_result, Phase::Liquid, "Liquid");
                            ui.radio_value(&mut self.ui.test_result, Phase::Solid, "Solid");
                        });
                        ui.end_row();

                        ui.label("Note");
                        ui.text_edit_multiline(&mut self.ui.test_note);
                        ui.end_row();
                    });

                ui.add_space(6.0);
                if ui.button("Record result").clicked() {
                    self.add_observation(sample_id);
                }
            } else {
                ui.label("Select a sample to record observations.");
            }
        });
    }

    fn tab_history(&mut self, ui: &mut egui::Ui) {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Experiment History");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if Self::file_selector_row(ui, &mut self.ui.observations_file) {
                        self.load_observations();
                        self.ui.export_selected_obs_ids.clear();
                        self.show_status("Observations file loaded.", StatusKind::Ok);
                    }
                    ui.separator();
                    if ui.button("Export selected…").clicked() {
                        if !self.ui.export_selected_obs_ids.is_empty() {
                            if let Some(path) = FileDialog::new()
                                .set_file_name(DEFAULT_CSV_FILE)
                                .add_filter("CSV", &["csv"])
                                .save_file()
                            {
                                match self.export_to_csv(
                                    Some(&self.ui.export_selected_obs_ids),
                                    path.as_path(),
                                ) {
                                    Ok(()) => self.show_status(
                                        "Selected histories exported to CSV.",
                                        StatusKind::Ok,
                                    ),
                                    Err(err) => self.show_status(
                                        format!("Export failed: {}", err),
                                        StatusKind::Error,
                                    ),
                                }
                            }
                        } else {
                            self.show_status(
                                "No histories selected for export.",
                                StatusKind::Error,
                            );
                        }
                    }

                    if ui.button("Export all…").clicked() {
                        if let Some(path) = FileDialog::new()
                            .set_file_name(DEFAULT_CSV_FILE)
                            .add_filter("CSV", &["csv"])
                            .save_file()
                        {
                            match self.export_to_csv(None, path.as_path()) {
                                Ok(()) => self.show_status("Exported all to CSV.", StatusKind::Ok),
                                Err(err) => self.show_status(
                                    format!("Export failed: {}", err),
                                    StatusKind::Error,
                                ),
                            }
                        }
                    }
                });
            });

            ui.add_space(6.0);
            egui::ScrollArea::vertical().show(ui, |ui| {
                egui::Grid::new("history_list")
                    .striped(true)
                    .spacing([12.0, 6.0])
                    .show(ui, |ui| {
                        ui.strong("Sample");
                        ui.strong("Components");
                        ui.strong("Target ratio");
                        ui.strong("Real ratio");
                        ui.strong("Temp (C)");
                        ui.strong("Result");
                        ui.strong("Note");
                        ui.strong("Action");
                        ui.end_row();

                        let mut delete_id = None;
                        for obs in self.observations_data.observations.iter().rev() {
                            let sample = self.samples_data.samples.iter().find(|s| s.id == obs.sample_id);
                            if let Some(sample) = sample {
                                ui.label(format!("#{}", sample.id));
                                match sample.type_ {
                                    SampleType::Binary => ui.label(format!(
                                        "{} + {}",
                                        sample.c1_name,
                                        sample.c2_name.as_deref().unwrap_or("?")
                                    )),
                                    SampleType::Pure => ui.label(format!("Pure {}", sample.c1_name)),
                                };
                                ui.label(sample.ratio_label.clone());
                                ui.label(Self::real_ratio_display(sample));
                            } else {
                                ui.label(format!("#{}", obs.sample_id));
                                ui.label("Unknown sample");
                                ui.label("-");
                                ui.label("-");
                            }

                            ui.label(format!("{:.2}", obs.temperature));
                            ui.colored_label(
                                match obs.result {
                                    Phase::Liquid => egui::Color32::from_rgb(40, 140, 70),
                                    Phase::Solid => egui::Color32::from_rgb(180, 60, 60),
                                },
                                obs.result.label(),
                            );
                            ui.label(obs.note.clone().unwrap_or_default());
                            ui.horizontal(|ui| {
                                let is_selected =
                                    self.ui.export_selected_obs_ids.contains(&obs.id);
                                if ui
                                    .selectable_label(is_selected, "Select")
                                    .clicked()
                                {
                                    if is_selected {
                                        self.ui
                                            .export_selected_obs_ids
                                            .retain(|id| *id != obs.id);
                                    } else {
                                        self.ui.export_selected_obs_ids.push(obs.id);
                                    }
                                }
                                if ui.button("Delete").clicked() {
                                    delete_id = Some(obs.id);
                                }
                            });
                            ui.end_row();
                        }

                        if let Some(id) = delete_id {
                            self.observations_data.observations.retain(|o| o.id != id);
                            self.save_observations();
                            self.show_status("Observation deleted.", StatusKind::Ok);
                        }
                    });
            });
        });
    }
}

impl eframe::App for LabApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            self.tab_header(ui);
        });

        egui::CentralPanel::default().show(ctx, |ui| match self.ui.current_tab {
            Tab::Molecules => self.tab_molecules(ui),
            Tab::Weighing => self.tab_weighing(ui),
            Tab::Testing => self.tab_testing(ui),
            Tab::History => self.tab_history(ui),
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Eutectic Lab Notebook",
        options,
        Box::new(|cc| Ok(Box::new(LabApp::new(cc)))),
    )
}
