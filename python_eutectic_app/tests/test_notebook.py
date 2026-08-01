import csv
import json
import tempfile
import unittest
from pathlib import Path

try:
    from python_eutectic_app.notebook import CSV_HEADER, LabNotebook
    from python_eutectic_app.storage import DatasetStore, read_json_file
except ModuleNotFoundError:
    from notebook import CSV_HEADER, LabNotebook
    from storage import DatasetStore, read_json_file


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
LEGACY_EXAMPLE = REPOSITORY_ROOT / "eutectic_app" / "lab_notebook.json"


class NotebookTests(unittest.TestCase):
    def setUp(self):
        self.temporary_directory = tempfile.TemporaryDirectory()
        self.data_directory = Path(self.temporary_directory.name) / "data"
        self.notebook = LabNotebook(self.data_directory)

    def tearDown(self):
        self.temporary_directory.cleanup()

    def add_two_molecules(self):
        acceptor = self.notebook.add_molecule(
            "Acceptor",
            "ACC",
            "111-11-1",
            "100.0",
            "80",
            "HBA",
            "Supplier A",
            "A-1",
            "https://example.com/a",
        )
        donor = self.notebook.add_molecule(
            "Donor",
            "DON",
            "222-22-2",
            "50.0",
            "40",
            "HBD",
        )
        return acceptor, donor

    def test_loading_missing_files_does_not_create_or_change_files(self):
        self.assertEqual([], self.notebook.molecules)
        self.assertEqual([], self.notebook.samples)
        self.assertEqual([], self.notebook.observations)
        self.assertFalse(self.data_directory.exists())

    def test_complete_experiment_uses_the_rust_json_shape(self):
        acceptor, donor = self.add_two_molecules()
        sample = self.notebook.add_binary_sample(
            acceptor["id"], donor["id"], "1", "2", "100", "100"
        )
        observation = self.notebook.add_observation(
            sample["id"], "25", "Solid", "Still crystalline"
        )

        self.assertEqual("Binary", sample["type"])
        self.assertNotIn("type_", sample)
        self.assertEqual("111-11-1", sample["c1_cas"])
        self.assertEqual("222-22-2", sample["c2_cas"])
        self.assertAlmostEqual(2.0, sample["real_molar_ratio"])
        self.assertEqual(sample, observation["sample_snapshot"])
        self.assertTrue(observation["timestamp"].endswith("Z"))

        with (self.data_directory / "samples.json").open(encoding="utf-8") as file:
            saved = json.load(file)
        self.assertEqual(sample, saved["samples"][0])
        self.assertEqual(2, saved["next_sample_id"])

    def test_theoretical_mass_matches_the_rust_formula(self):
        acceptor, donor = self.add_two_molecules()
        text = self.notebook.theoretical_mass(
            acceptor["id"], donor["id"], "1", "2", "100", ""
        )
        self.assertEqual(
            "Theoretical mass for DON (component 2) is 100.00 mg.",
            text,
        )

    def test_duplicate_cas_is_rejected_without_using_an_id(self):
        self.add_two_molecules()
        next_id = self.notebook.molecules_data["next_mol_id"]
        with self.assertRaisesRegex(ValueError, "CAS number"):
            self.notebook.add_molecule(
                "Duplicate", "DUP", "111-11-1", "10", "", "HBA"
            )
        self.assertEqual(next_id, self.notebook.molecules_data["next_mol_id"])

    def test_deleting_a_sample_also_deletes_its_observations(self):
        acceptor, donor = self.add_two_molecules()
        sample = self.notebook.add_binary_sample(
            acceptor["id"], donor["id"], "1", "1", "20", "10"
        )
        self.notebook.add_observation(sample["id"], "20", "Liquid")

        self.notebook.delete_sample(sample["id"])

        self.assertEqual([], self.notebook.samples)
        self.assertEqual([], self.notebook.observations)
        reloaded = LabNotebook(self.data_directory)
        self.assertEqual([], reloaded.samples)
        self.assertEqual([], reloaded.observations)

    def test_csv_export_has_the_same_columns_and_format(self):
        acceptor, donor = self.add_two_molecules()
        sample = self.notebook.add_binary_sample(
            acceptor["id"], donor["id"], "1", "2", "100", "100"
        )
        observation = self.notebook.add_observation(
            sample["id"], "25", "Solid", "A note"
        )
        csv_path = self.data_directory / "export.csv"

        self.notebook.export_csv(csv_path, [observation["id"]])

        with csv_path.open(encoding="utf-8", newline="") as file:
            rows = list(csv.reader(file))
        self.assertEqual(CSV_HEADER, rows[0])
        self.assertEqual("Binary", rows[1][3])
        self.assertEqual("1:2.000", rows[1][9])
        self.assertEqual("25.00", rows[1][10])
        self.assertEqual("A note", rows[1][12])

    def test_old_real_dataset_imports_without_changing_the_source(self):
        if not LEGACY_EXAMPLE.exists():
            self.skipTest("The repository's legacy example is not present")
        source_before = LEGACY_EXAMPLE.read_bytes()
        imported_directory = Path(self.temporary_directory.name) / "imported"

        store = DatasetStore(imported_directory)
        store.import_legacy_file(LEGACY_EXAMPLE)
        imported = LabNotebook(imported_directory)

        self.assertEqual(2, len(imported.molecules))
        self.assertEqual(2, len(imported.samples))
        self.assertEqual(4, len(imported.observations))
        self.assertEqual("Binary", imported.samples[0]["type"])
        self.assertNotIn("type_", imported.samples[0])
        self.assertIn("melting_point", imported.molecules[0])
        self.assertIsNotNone(imported.observations[0]["sample_snapshot"])
        self.assertEqual(source_before, LEGACY_EXAMPLE.read_bytes())

    def test_import_refuses_to_overwrite_existing_dataset(self):
        if not LEGACY_EXAMPLE.exists():
            self.skipTest("The repository's legacy example is not present")
        self.data_directory.mkdir(parents=True)
        existing = self.data_directory / "molecules.json"
        existing.write_text('{"molecules": [], "next_mol_id": 1}', encoding="utf-8")

        with self.assertRaises(FileExistsError):
            self.notebook.store.import_legacy_file(LEGACY_EXAMPLE)

    def test_next_id_is_normalised_when_a_file_has_a_stale_value(self):
        path = self.data_directory / "molecules.json"
        path.parent.mkdir(parents=True)
        path.write_text(
            '{"molecules": [{"id": 8}], "next_mol_id": 2}',
            encoding="utf-8",
        )
        data = read_json_file(path, "molecules", "next_mol_id")
        self.assertEqual(9, data["next_mol_id"])


if __name__ == "__main__":
    unittest.main()
