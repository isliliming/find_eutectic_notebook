# Eutectic Lab Notebook (Rust + egui)

A small standalone desktop laboratory notebook for binary eutectic systems, written in Rust using `eframe` / `egui`.

It helps you:

- Manage **molecule** metadata (HBA/HBD, CAS, MW, supplier, etc.).
- Prepare and record **binary mixes** and **pure samples** with real molar ratios.
- Log **phase behaviour** (liquid/solid vs temperature) with notes.
- Export selected or all **experiment histories to CSV** for external analysis.

The app stores its data locally in `lab_notebook.json` in the project directory.

---

## Features overview

### Molecules tab

- Add molecules with:
  - Name, short name, CAS number, molecular weight.
  - Role: **HBA (acceptor)** or **HBD (donor)**.
  - Supplier, SKU, and webpage.
- Live search by name, short name, or CAS.
- Delete molecules from the list.

### Weighting tab

- **Binary mixtures**:
  - Select Acceptor (HBA) and Donor (HBD).
  - Enter target molar parts (e.g. `3 : 7`).
  - Enter either:
    - Mass of component 1, or
    - Mass of component 2.
  - App suggests the **theoretical mass** of the other component.
  - Enter both real masses to save a sample.
  - Real molar ratio is stored and displayed in the same **`a:b_actual`** format as the target.

- **Pure component** mode:
  - Select a single molecule and a mass, then save the sample.

- Prepared samples list:
  - Shows ID, composition, target ratio, real ratio, masses.
  - Delete samples (this also removes their observations).

### Testing tab

- Select any prepared sample.
- Record:
  - Temperature (°C).
  - Result: **Liquid** or **Solid**.
  - A free-text **Note**.
- Each entry is stored as an `Observation`.

### History & Export tab

- Table of all observations with:
  - Sample ID and components.
  - Target ratio and real ratio (`a:b_actual`).
  - Temperature, result, and note.
- Per-row actions:
  - **Select** one or many observations for export.
  - **Delete** an observation.
- Export buttons:
  - **Export selected…**
    - Exports only the selected observations to a CSV.
  - **Export all…**
    - Exports all observations.
  - Both open a native **Save As** dialog where you choose the file name and path.

CSV columns:

- `Date, Obs ID, Sample ID, Type, Component 1, Mass 1 (mg), Component 2, Mass 2 (mg), Target Ratio, Real Ratio, Temp (C), Result, Note`

---

## Running the app

You need a recent Rust toolchain (Rust 1.72+ recommended).

```bash
cd /Users/liming/Documents/GitHub/find_eutectic_notebook
cargo run
```

This starts the GUI app in debug mode.

For a faster, optimised build:

```bash
cargo build --release
./target/release/eutectic_lab_notebook
```

The executable reads/writes `lab_notebook.json` in the current working directory.

---

## Packaging as a macOS app (optional)

For personal use you can wrap the release binary into a simple `.app` bundle:

1. Build the release binary:

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

4. Provide a minimal `Info.plist` (see Apple docs for full details).

You can then double-click `Eutectic Lab Notebook.app` in Finder. For wider distribution you would additionally code‑sign and notarise the app using an Apple Developer account.

---

## Files to commit vs ignore

Typical recommendations:

- **Commit**:
  - `src/`
  - `Cargo.toml`
  - `Cargo.lock`
  - `README.md`
- **Usually ignore**:
  - `target/` (build artefacts)
  - `.DS_Store` (macOS Finder metadata)
  - `lab_notebook.json` if it contains personal/experiment data rather than example data

You can enforce this with a `.gitignore` similar to:

```gitignore
/target
**/*.DS_Store
lab_notebook.json
```

