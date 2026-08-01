"""Tkinter user interface for the Eutectic Lab Notebook."""

import tkinter as tk
from pathlib import Path
from tkinter import filedialog, messagebox, ttk

try:
    from .notebook import LabNotebook
    from .storage import DatasetStore
except ImportError:
    from notebook import LabNotebook
    from storage import DatasetStore


class EutecticApp:
    """Build and control the four notebook tabs."""

    def __init__(self, root, data_directory=None):
        self.root = root
        self.notebook_data = LabNotebook(data_directory)

        self.root.title("Eutectic Lab Notebook - Python")
        self.root.geometry("1200x800")
        self.root.minsize(900, 650)

        self.status_text = tk.StringVar(value="Ready")
        self.data_path_text = tk.StringVar()

        self.build_header()
        self.tabs = ttk.Notebook(self.root)
        self.tabs.pack(fill="both", expand=True, padx=10, pady=(0, 6))

        self.molecules_tab = ttk.Frame(self.tabs, padding=10)
        self.weighing_tab = ttk.Frame(self.tabs, padding=10)
        self.testing_tab = ttk.Frame(self.tabs, padding=10)
        self.history_tab = ttk.Frame(self.tabs, padding=10)

        self.tabs.add(self.molecules_tab, text="1. Molecules")
        self.tabs.add(self.weighing_tab, text="2. Weighing")
        self.tabs.add(self.testing_tab, text="3. Testing")
        self.tabs.add(self.history_tab, text="4. History & Export")

        self.build_molecules_tab()
        self.build_weighing_tab()
        self.build_testing_tab()
        self.build_history_tab()

        status_bar = ttk.Label(
            self.root,
            textvariable=self.status_text,
            relief="sunken",
            anchor="w",
            padding=5,
        )
        status_bar.pack(fill="x", side="bottom")
        self.refresh_everything()

    def build_header(self):
        header = ttk.Frame(self.root, padding=10)
        header.pack(fill="x")

        ttk.Label(
            header,
            text="Eutectic Lab Notebook",
            font=("TkDefaultFont", 16, "bold"),
        ).grid(row=0, column=0, sticky="w")

        ttk.Label(header, text="Data folder:").grid(row=1, column=0, sticky="w", pady=(8, 0))
        path_entry = ttk.Entry(header, textvariable=self.data_path_text, state="readonly")
        path_entry.grid(row=1, column=1, sticky="ew", padx=6, pady=(8, 0))
        ttk.Button(header, text="Choose folder...", command=self.choose_data_folder).grid(
            row=1, column=2, padx=3, pady=(8, 0)
        )
        ttk.Button(
            header,
            text="Import old lab_notebook.json...",
            command=self.import_legacy_data,
        ).grid(row=1, column=3, padx=3, pady=(8, 0))
        header.columnconfigure(1, weight=1)

    def build_molecules_tab(self):
        form = ttk.LabelFrame(self.molecules_tab, text="Register New Molecule", padding=10)
        form.pack(fill="x")

        self.molecule_fields = {}
        field_names = [
            ("name", "Name"),
            ("short_name", "Short name"),
            ("cas_number", "CAS"),
            ("molecular_weight", "Molecular weight"),
            ("melting_point", "Melting point (°C)"),
            ("supplier", "Supplier"),
            ("sku", "SKU"),
            ("webpage", "Webpage"),
        ]
        for index, (key, label) in enumerate(field_names):
            block = index // 4
            row = index % 4
            label_column = block * 2
            entry_column = label_column + 1
            variable = tk.StringVar()
            self.molecule_fields[key] = variable
            ttk.Label(form, text=label).grid(
                row=row, column=label_column, sticky="w", padx=(0, 5), pady=3
            )
            ttk.Entry(form, textvariable=variable).grid(
                row=row, column=entry_column, sticky="ew", padx=(0, 16), pady=3
            )

        self.molecule_role = tk.StringVar(value="HBA")
        ttk.Label(form, text="Role").grid(row=4, column=0, sticky="w", pady=3)
        role_frame = ttk.Frame(form)
        role_frame.grid(row=4, column=1, sticky="w")
        ttk.Radiobutton(
            role_frame,
            text="Acceptor (HBA)",
            variable=self.molecule_role,
            value="HBA",
        ).pack(side="left")
        ttk.Radiobutton(
            role_frame,
            text="Donor (HBD)",
            variable=self.molecule_role,
            value="HBD",
        ).pack(side="left", padx=10)
        ttk.Button(form, text="Add molecule", command=self.add_molecule).grid(
            row=4, column=3, sticky="e", pady=5
        )
        form.columnconfigure(1, weight=1)
        form.columnconfigure(3, weight=1)

        list_frame = ttk.LabelFrame(self.molecules_tab, text="Registered Molecules", padding=10)
        list_frame.pack(fill="both", expand=True, pady=(10, 0))

        search_frame = ttk.Frame(list_frame)
        search_frame.pack(fill="x", pady=(0, 6))
        ttk.Label(search_frame, text="Search:").pack(side="left")
        self.molecule_search = tk.StringVar()
        search_entry = ttk.Entry(search_frame, textvariable=self.molecule_search)
        search_entry.pack(side="left", fill="x", expand=True, padx=6)
        search_entry.bind("<KeyRelease>", lambda event: self.refresh_molecule_tree())
        ttk.Button(search_frame, text="Delete selected", command=self.delete_molecule).pack(
            side="right"
        )

        columns = ("short", "name", "cas", "mw", "mp", "role", "supplier", "webpage")
        self.molecule_tree = self.make_tree(list_frame, columns)
        headings = ["Short", "Name", "CAS", "MW", "Melting Point", "Role", "Supplier", "Webpage"]
        widths = [80, 150, 100, 80, 100, 70, 120, 180]
        self.set_tree_headings(self.molecule_tree, columns, headings, widths)

    def build_weighing_tab(self):
        form = ttk.LabelFrame(self.weighing_tab, text="Sample Preparation / Weighing", padding=10)
        form.pack(fill="x")

        self.sample_mode = tk.StringVar(value="Binary")
        ttk.Label(form, text="Mode").grid(row=0, column=0, sticky="w", pady=3)
        mode_frame = ttk.Frame(form)
        mode_frame.grid(row=0, column=1, columnspan=3, sticky="w")
        ttk.Radiobutton(
            mode_frame,
            text="Binary mixture",
            variable=self.sample_mode,
            value="Binary",
            command=self.update_weighing_mode,
        ).pack(side="left")
        ttk.Radiobutton(
            mode_frame,
            text="Pure component",
            variable=self.sample_mode,
            value="Pure",
            command=self.update_weighing_mode,
        ).pack(side="left", padx=10)

        self.component_1 = ttk.Combobox(form, state="readonly")
        self.component_2 = ttk.Combobox(form, state="readonly")
        self.component_1_ids = []
        self.component_2_ids = []
        ttk.Label(form, text="Acceptor / molecule").grid(row=1, column=0, sticky="w", pady=3)
        self.component_1.grid(row=1, column=1, sticky="ew", padx=(0, 16), pady=3)
        ttk.Label(form, text="Donor").grid(row=1, column=2, sticky="w", pady=3)
        self.component_2.grid(row=1, column=3, sticky="ew", pady=3)

        self.parts_1 = tk.StringVar(value="1")
        self.parts_2 = tk.StringVar(value="1")
        self.mass_1 = tk.StringVar()
        self.mass_2 = tk.StringVar()
        weighing_fields = [
            (2, 0, "Acceptor parts", self.parts_1),
            (2, 2, "Donor parts", self.parts_2),
            (3, 0, "Mass component 1 (mg)", self.mass_1),
            (3, 2, "Actual mass component 2 (mg)", self.mass_2),
        ]
        for row, column, label, variable in weighing_fields:
            ttk.Label(form, text=label).grid(row=row, column=column, sticky="w", pady=3)
            entry = ttk.Entry(form, textvariable=variable)
            entry.grid(row=row, column=column + 1, sticky="ew", padx=(0, 16), pady=3)
            entry.bind("<KeyRelease>", lambda event: self.refresh_theoretical_mass())

        self.component_1.bind("<<ComboboxSelected>>", lambda event: self.refresh_theoretical_mass())
        self.component_2.bind("<<ComboboxSelected>>", lambda event: self.refresh_theoretical_mass())
        self.theoretical_text = tk.StringVar()
        ttk.Label(form, textvariable=self.theoretical_text).grid(
            row=4, column=0, columnspan=3, sticky="w", pady=5
        )
        self.save_sample_button = ttk.Button(
            form,
            text="Save binary sample",
            command=self.save_sample,
        )
        self.save_sample_button.grid(row=4, column=3, sticky="e", pady=5)
        form.columnconfigure(1, weight=1)
        form.columnconfigure(3, weight=1)

        list_frame = ttk.LabelFrame(self.weighing_tab, text="Prepared Samples", padding=10)
        list_frame.pack(fill="both", expand=True, pady=(10, 0))
        ttk.Button(list_frame, text="Delete selected", command=self.delete_sample).pack(
            anchor="e", pady=(0, 6)
        )
        columns = ("id", "composition", "target", "real", "masses", "created")
        self.sample_tree = self.make_tree(list_frame, columns)
        headings = ["ID", "Composition", "Target Ratio", "Real Ratio", "Masses (mg)", "Created"]
        widths = [60, 210, 100, 100, 130, 180]
        self.set_tree_headings(self.sample_tree, columns, headings, widths)

    def build_testing_tab(self):
        form = ttk.LabelFrame(self.testing_tab, text="Phase Observation", padding=10)
        form.pack(fill="both", expand=True)

        self.test_sample = ttk.Combobox(form, state="readonly")
        self.test_sample_ids = []
        ttk.Label(form, text="Sample").grid(row=0, column=0, sticky="nw", pady=3)
        self.test_sample.grid(row=0, column=1, sticky="ew", pady=3)
        self.test_sample.bind("<<ComboboxSelected>>", lambda event: self.show_sample_details())

        self.sample_details = tk.StringVar(value="Select a sample to record observations.")
        ttk.Label(form, textvariable=self.sample_details).grid(
            row=1, column=1, sticky="w", pady=(2, 12)
        )

        self.test_temperature = tk.StringVar(value="25.0")
        ttk.Label(form, text="Temperature (°C)").grid(row=2, column=0, sticky="w", pady=3)
        ttk.Entry(form, textvariable=self.test_temperature).grid(row=2, column=1, sticky="ew", pady=3)

        self.test_result = tk.StringVar(value="Liquid")
        ttk.Label(form, text="Result").grid(row=3, column=0, sticky="w", pady=3)
        result_frame = ttk.Frame(form)
        result_frame.grid(row=3, column=1, sticky="w", pady=3)
        ttk.Radiobutton(
            result_frame,
            text="Liquid",
            variable=self.test_result,
            value="Liquid",
        ).pack(side="left")
        ttk.Radiobutton(
            result_frame,
            text="Solid",
            variable=self.test_result,
            value="Solid",
        ).pack(side="left", padx=10)

        ttk.Label(form, text="Note").grid(row=4, column=0, sticky="nw", pady=3)
        self.test_note = tk.Text(form, height=6, wrap="word")
        self.test_note.grid(row=4, column=1, sticky="nsew", pady=3)
        ttk.Button(form, text="Record result", command=self.add_observation).grid(
            row=5, column=1, sticky="e", pady=8
        )
        form.columnconfigure(1, weight=1)
        form.rowconfigure(4, weight=1)

    def build_history_tab(self):
        toolbar = ttk.Frame(self.history_tab)
        toolbar.pack(fill="x", pady=(0, 6))
        ttk.Button(toolbar, text="Export selected...", command=self.export_selected).pack(
            side="left"
        )
        ttk.Button(toolbar, text="Export all...", command=self.export_all).pack(
            side="left", padx=6
        )
        ttk.Button(toolbar, text="Delete selected", command=self.delete_observations).pack(
            side="right"
        )

        columns = (
            "obs",
            "sample",
            "components",
            "target",
            "real",
            "temperature",
            "result",
            "note",
            "date",
        )
        self.history_tree = self.make_tree(self.history_tab, columns, selectmode="extended")
        headings = [
            "Obs ID",
            "Sample",
            "Components",
            "Target Ratio",
            "Real Ratio",
            "Temp (°C)",
            "Result",
            "Note",
            "Date",
        ]
        widths = [65, 70, 180, 90, 90, 80, 70, 220, 150]
        self.set_tree_headings(self.history_tree, columns, headings, widths)

    def make_tree(self, parent, columns, selectmode="browse"):
        container = ttk.Frame(parent)
        container.pack(fill="both", expand=True)
        tree = ttk.Treeview(
            container,
            columns=columns,
            show="headings",
            selectmode=selectmode,
        )
        vertical = ttk.Scrollbar(container, orient="vertical", command=tree.yview)
        horizontal = ttk.Scrollbar(container, orient="horizontal", command=tree.xview)
        tree.configure(yscrollcommand=vertical.set, xscrollcommand=horizontal.set)
        tree.grid(row=0, column=0, sticky="nsew")
        vertical.grid(row=0, column=1, sticky="ns")
        horizontal.grid(row=1, column=0, sticky="ew")
        container.columnconfigure(0, weight=1)
        container.rowconfigure(0, weight=1)
        return tree

    def set_tree_headings(self, tree, columns, headings, widths):
        for index in range(len(columns)):
            tree.heading(columns[index], text=headings[index])
            tree.column(columns[index], width=widths[index], minwidth=50)

    def show_status(self, text):
        self.status_text.set(text)

    def show_error(self, error):
        self.show_status(str(error))
        messagebox.showerror("Eutectic Lab Notebook", str(error), parent=self.root)

    def selected_id(self, tree):
        selected = tree.selection()
        if not selected:
            return None
        return int(selected[0])

    def combobox_id(self, combobox, ids):
        index = combobox.current()
        if index < 0 or index >= len(ids):
            return None
        return ids[index]

    def choose_data_folder(self):
        folder = filedialog.askdirectory(
            title="Choose a folder containing the three JSON files",
            initialdir=str(self.notebook_data.store.data_directory),
            parent=self.root,
        )
        if not folder:
            return
        try:
            self.notebook_data.use_data_directory(folder)
            self.refresh_everything()
            self.show_status("Dataset loaded from " + folder)
        except (OSError, ValueError) as error:
            self.show_error(error)

    def import_legacy_data(self):
        old_file = filedialog.askopenfilename(
            title="Choose the old lab_notebook.json",
            filetypes=[("JSON files", "*.json"), ("All files", "*")],
            parent=self.root,
        )
        if not old_file:
            return
        destination = filedialog.askdirectory(
            title="Choose an empty destination folder for the converted files",
            parent=self.root,
        )
        if not destination:
            return
        try:
            destination_store = DatasetStore(destination)
            destination_store.import_legacy_file(old_file)
            self.notebook_data.use_data_directory(destination)
            self.refresh_everything()
            self.show_status("Old dataset imported into " + destination)
        except (OSError, ValueError) as error:
            self.show_error(error)

    def add_molecule(self):
        values = self.molecule_fields
        try:
            self.notebook_data.add_molecule(
                values["name"].get(),
                values["short_name"].get(),
                values["cas_number"].get(),
                values["molecular_weight"].get(),
                values["melting_point"].get(),
                self.molecule_role.get(),
                values["supplier"].get(),
                values["sku"].get(),
                values["webpage"].get(),
            )
            for variable in values.values():
                variable.set("")
            self.refresh_molecule_tree()
            self.refresh_molecule_choices()
            self.show_status("Molecule added.")
        except (OSError, ValueError) as error:
            self.show_error(error)

    def delete_molecule(self):
        molecule_id = self.selected_id(self.molecule_tree)
        if molecule_id is None:
            self.show_error("Select a molecule first.")
            return
        if not messagebox.askyesno("Delete molecule", "Delete the selected molecule?", parent=self.root):
            return
        try:
            self.notebook_data.delete_molecule(molecule_id)
            self.refresh_molecule_tree()
            self.refresh_molecule_choices()
            self.show_status("Molecule deleted.")
        except (OSError, ValueError) as error:
            self.show_error(error)

    def update_weighing_mode(self):
        is_binary = self.sample_mode.get() == "Binary"
        state = "readonly" if is_binary else "disabled"
        self.component_2.configure(state=state)
        self.save_sample_button.configure(
            text="Save binary sample" if is_binary else "Save pure sample"
        )
        self.refresh_molecule_choices()
        self.refresh_theoretical_mass()

    def refresh_molecule_choices(self):
        if self.sample_mode.get() == "Binary":
            first_choices = self.notebook_data.molecules_with_role("HBA")
        else:
            first_choices = self.notebook_data.molecules
        second_choices = self.notebook_data.molecules_with_role("HBD")

        self.component_1_ids = [item["id"] for item in first_choices]
        self.component_2_ids = [item["id"] for item in second_choices]
        self.component_1["values"] = [
            "#" + str(item["id"]) + " " + item["short_name"] for item in first_choices
        ]
        self.component_2["values"] = [
            "#" + str(item["id"]) + " " + item["short_name"] for item in second_choices
        ]
        self.component_1.set("")
        self.component_2.set("")

    def refresh_theoretical_mass(self):
        if self.sample_mode.get() != "Binary":
            self.theoretical_text.set("")
            return
        component_1_id = self.combobox_id(self.component_1, self.component_1_ids)
        component_2_id = self.combobox_id(self.component_2, self.component_2_ids)
        text = self.notebook_data.theoretical_mass(
            component_1_id,
            component_2_id,
            self.parts_1.get(),
            self.parts_2.get(),
            self.mass_1.get(),
            self.mass_2.get(),
        )
        self.theoretical_text.set(text)

    def save_sample(self):
        component_1_id = self.combobox_id(self.component_1, self.component_1_ids)
        try:
            if self.sample_mode.get() == "Binary":
                component_2_id = self.combobox_id(self.component_2, self.component_2_ids)
                self.notebook_data.add_binary_sample(
                    component_1_id,
                    component_2_id,
                    self.parts_1.get(),
                    self.parts_2.get(),
                    self.mass_1.get(),
                    self.mass_2.get(),
                )
                self.mass_2.set("")
                message = "Binary sample saved."
            else:
                self.notebook_data.add_pure_sample(component_1_id, self.mass_1.get())
                message = "Pure sample saved."
            self.refresh_sample_tree()
            self.refresh_test_sample_choices()
            self.refresh_history_tree()
            self.show_status(message)
        except (OSError, ValueError) as error:
            self.show_error(error)

    def delete_sample(self):
        sample_id = self.selected_id(self.sample_tree)
        if sample_id is None:
            self.show_error("Select a sample first.")
            return
        if not messagebox.askyesno(
            "Delete sample",
            "Delete this sample and all its observations?",
            parent=self.root,
        ):
            return
        try:
            self.notebook_data.delete_sample(sample_id)
            self.refresh_sample_tree()
            self.refresh_test_sample_choices()
            self.refresh_history_tree()
            self.show_status("Sample deleted.")
        except (OSError, ValueError) as error:
            self.show_error(error)

    def show_sample_details(self):
        sample_id = self.combobox_id(self.test_sample, self.test_sample_ids)
        sample = self.notebook_data.find_sample(sample_id)
        if sample is None:
            self.sample_details.set("Select a sample to record observations.")
            return

        component_1 = self.notebook_data.find_molecule(sample.get("c1_id"))
        melting_1 = self.melting_point_text(component_1)
        if self.notebook_data.sample_type(sample) == "Pure":
            details = "Melting point: " + melting_1 + " (" + sample["c1_name"] + ")"
        else:
            component_2 = self.notebook_data.find_molecule(sample.get("c2_id"))
            melting_2 = self.melting_point_text(component_2)
            details = (
                "Melting points: "
                + melting_1
                + " ("
                + sample["c1_name"]
                + ") + "
                + melting_2
                + " ("
                + str(sample.get("c2_name") or "?")
                + ")"
            )
        self.sample_details.set(details)

    def melting_point_text(self, molecule):
        if molecule is None or molecule.get("melting_point") is None:
            return "Unknown"
        return format(float(molecule["melting_point"]), ".1f") + " °C"

    def add_observation(self):
        sample_id = self.combobox_id(self.test_sample, self.test_sample_ids)
        note = self.test_note.get("1.0", "end").strip()
        try:
            self.notebook_data.add_observation(
                sample_id,
                self.test_temperature.get(),
                self.test_result.get(),
                note,
            )
            self.test_note.delete("1.0", "end")
            self.refresh_history_tree()
            self.show_status("Observation recorded.")
        except (OSError, ValueError) as error:
            self.show_error(error)

    def export_selected(self):
        selected = self.history_tree.selection()
        if not selected:
            self.show_error("Select one or more observations first.")
            return
        observation_ids = [int(item) for item in selected]
        self.choose_csv_path(observation_ids)

    def export_all(self):
        self.choose_csv_path(None)

    def choose_csv_path(self, observation_ids):
        path = filedialog.asksaveasfilename(
            title="Export experiment data",
            initialfile="experiment_data.csv",
            defaultextension=".csv",
            filetypes=[("CSV files", "*.csv")],
            parent=self.root,
        )
        if not path:
            return
        try:
            self.notebook_data.export_csv(path, observation_ids)
            self.show_status("CSV exported to " + path)
        except OSError as error:
            self.show_error(error)

    def delete_observations(self):
        selected = self.history_tree.selection()
        if not selected:
            self.show_error("Select one or more observations first.")
            return
        if not messagebox.askyesno(
            "Delete observations",
            "Delete the selected observations?",
            parent=self.root,
        ):
            return
        try:
            for item in selected:
                self.notebook_data.delete_observation(int(item))
            self.refresh_history_tree()
            self.show_status("Selected observations deleted.")
        except (OSError, ValueError) as error:
            self.show_error(error)

    def refresh_everything(self):
        self.data_path_text.set(str(self.notebook_data.store.data_directory))
        self.refresh_molecule_tree()
        self.refresh_molecule_choices()
        self.refresh_sample_tree()
        self.refresh_test_sample_choices()
        self.refresh_history_tree()
        self.update_weighing_mode()

    def clear_tree(self, tree):
        for item in tree.get_children():
            tree.delete(item)

    def refresh_molecule_tree(self):
        self.clear_tree(self.molecule_tree)
        query = self.molecule_search.get().strip().lower()
        for molecule in self.notebook_data.molecules:
            searchable = " ".join(
                [
                    str(molecule.get("name", "")),
                    str(molecule.get("short_name", "")),
                    str(molecule.get("cas_number", "")),
                ]
            ).lower()
            if query and query not in searchable:
                continue
            melting_point = molecule.get("melting_point")
            melting_text = "-" if melting_point is None else format(float(melting_point), ".1f")
            values = (
                molecule.get("short_name", ""),
                molecule.get("name", ""),
                molecule.get("cas_number", ""),
                format(float(molecule.get("mw", 0)), ".3f"),
                melting_text,
                molecule.get("role", ""),
                molecule.get("supplier", ""),
                molecule.get("webpage", ""),
            )
            self.molecule_tree.insert("", "end", iid=str(molecule["id"]), values=values)

    def refresh_sample_tree(self):
        self.clear_tree(self.sample_tree)
        for sample in reversed(self.notebook_data.samples):
            if self.notebook_data.sample_type(sample) == "Pure":
                composition = "Pure " + sample.get("c1_name", "")
                masses = format(float(sample.get("c1_mass", 0)), ".2f")
            else:
                composition = (
                    sample.get("c1_name", "")
                    + " + "
                    + str(sample.get("c2_name") or "?")
                )
                masses = (
                    format(float(sample.get("c1_mass", 0)), ".2f")
                    + " / "
                    + format(float(sample.get("c2_mass", 0)), ".2f")
                )
            values = (
                "#" + str(sample["id"]),
                composition,
                sample.get("ratio_label", ""),
                self.notebook_data.real_ratio_display(sample),
                masses,
                str(sample.get("created_at", "")),
            )
            self.sample_tree.insert("", "end", iid=str(sample["id"]), values=values)

    def refresh_test_sample_choices(self):
        choices = list(reversed(self.notebook_data.samples))
        self.test_sample_ids = [sample["id"] for sample in choices]
        self.test_sample["values"] = [self.notebook_data.sample_label(sample) for sample in choices]
        self.test_sample.set("")
        self.show_sample_details()

    def refresh_history_tree(self):
        self.clear_tree(self.history_tree)
        for observation in reversed(self.notebook_data.observations):
            sample = self.notebook_data.find_sample(observation.get("sample_id"))
            if sample is None:
                sample = observation.get("sample_snapshot")
            if sample is None:
                sample_id = "#" + str(observation.get("sample_id", ""))
                components = "Unknown sample"
                target = "-"
                real = "-"
            else:
                sample_id = "#" + str(sample.get("id", ""))
                if self.notebook_data.sample_type(sample) == "Pure":
                    components = "Pure " + sample.get("c1_name", "")
                else:
                    components = (
                        sample.get("c1_name", "")
                        + " + "
                        + str(sample.get("c2_name") or "?")
                    )
                target = sample.get("ratio_label", "")
                real = self.notebook_data.real_ratio_display(sample)
            values = (
                "#" + str(observation.get("id", "")),
                sample_id,
                components,
                target,
                real,
                format(float(observation.get("temperature", 0)), ".2f"),
                observation.get("result", ""),
                observation.get("note") or "",
                observation.get("timestamp", ""),
            )
            self.history_tree.insert(
                "",
                "end",
                iid=str(observation["id"]),
                values=values,
            )


def run_app(data_directory=None):
    root = tk.Tk()
    EutecticApp(root, data_directory)
    root.mainloop()
