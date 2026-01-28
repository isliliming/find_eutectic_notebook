# Eutectic Lab Notebook (Rust + egui)

A beginner‑friendly desktop laboratory notebook for **binary eutectic systems**, written in Rust using `eframe` / `egui`.

It helps you:

- Manage **molecule metadata** (HBA/HBD, CAS, MW, supplier, etc.).
- Prepare **binary mixtures** and **pure samples** with real molar ratios.
- Record **phase behaviour** (liquid/solid vs temperature) with notes.
- Export **selected or all** experiments to CSV for analysis.

---

## What this project does

This app organises your experimental workflow into four tabs:

1. **Molecules** – register and search chemicals.
2. **Weighing** – prepare samples (binary or pure) with real ratios.
3. **Testing** – log observations with temperature, phase, and notes.
4. **History & Export** – review results and export to CSV.

All data is stored locally as JSON files in `lab_data/` that you can swap in the UI.

---

## Quick start (step‑by‑step)

### 1) Install Rust

- Go to `https://rustup.rs` and install Rust (stable).
- Restart your terminal after installation.

### 2) Run the app (macOS / Linux)

```bash
git clone <your-repo-url>
cd find_eutectic_notebook
cargo run
```

### 3) Run the app (Windows)

```powershell
git clone <your-repo-url>
cd find_eutectic_notebook
cargo run
```

The first build may take a few minutes. Once complete, the GUI opens.

---

## Data files (important)

The app now uses **three separate JSON files** by default (stored in `lab_data/`):

- `lab_data/molecules.json` – all molecule records and next molecule ID
- `lab_data/samples.json` – prepared samples and next sample ID
- `lab_data/observations.json` – phase observations and next observation ID

### Switching files

Each tab has a **“Change file…”** button and a file path field:

- **Molecules tab** → selects the `lab_data/molecules.json` file
- **Weighing tab** → selects the `lab_data/samples.json` file
- **Testing tab** → selects the `lab_data/observations.json` file
- **History tab** → also controls the `lab_data/observations.json` file

This lets you keep multiple datasets and switch between them without copying files.

---

## Exporting to CSV

In **History & Export**:

- **Export selected…** → exports only the selected rows
- **Export all…** → exports everything

The save dialog lets you choose the filename and path.

CSV columns:

```
Date, Obs ID, Sample ID, Type, Component 1, Mass 1 (mg), Component 2, Mass 2 (mg), Target Ratio, Real Ratio, Temp (C), Result, Note
```

---

## Building for test

For testing, you can make a fast compile. Run the following command in the root directory:

```bash
cargo run
```

## Building a release version

For a fast, optimised build. Run the following command in the root directory:

```bash
cargo build --release
```

The executable will be:

- macOS/Linux: `target/release/eutectic_lab_notebook`
- Windows: `target\release\eutectic_lab_notebook.exe`

You can run that binary directly without installing Rust on the target machine.

---

## Packaging as a macOS app (optional)

You can wrap the release binary into a `.app` bundle for easier use:

1. Build a release binary:

   ```bash
   cargo build --release
   ```

2. Create a bundle structure:

   ```text
   Eutectic Lab Notebook.app/
     Contents/
       Info.plist
       MacOS/
         eutectic_lab_notebook
       Resources/
         AppIcon.icns  (optional)
   ```

3. Copy the binary:

   ```bash
   cp target/release/eutectic_lab_notebook "Eutectic Lab Notebook.app/Contents/MacOS/eutectic_lab_notebook"
   chmod +x "Eutectic Lab Notebook.app/Contents/MacOS/eutectic_lab_notebook"
   ```

For distribution beyond your own machine, you will need code‑signing and notarisation.

---

## Files to commit vs ignore

Recommended:

- **Commit**:
  - `src/`
  - `Cargo.toml`
  - `Cargo.lock`
  - `README.md`
- **Ignore**:
  - `target/` (build artefacts)
  - `.DS_Store` (macOS Finder metadata)
  - `lab_data/` (personal data)


