"""Read and write the JSON files used by the Rust application.

The program deliberately keeps the data as normal Python dictionaries.  This
makes the JSON structure easy to see and keeps this module beginner-friendly.
"""

import json
from pathlib import Path


MOLECULES_FILE_NAME = "molecules.json"
SAMPLES_FILE_NAME = "samples.json"
OBSERVATIONS_FILE_NAME = "observations.json"


def default_data_directory():
    """Return the repository's existing lab_data directory."""
    app_directory = Path(__file__).resolve().parent
    return app_directory.parent / "lab_data"


def read_json_file(path, list_name, next_id_name):
    """Read one Rust-compatible data file and normalise its next ID."""
    path = Path(path)
    if not path.exists():
        return {list_name: [], next_id_name: 1}

    with path.open("r", encoding="utf-8") as file:
        data = json.load(file)

    if not isinstance(data, dict):
        raise ValueError(str(path) + " must contain a JSON object.")
    if list_name not in data or not isinstance(data[list_name], list):
        raise ValueError(str(path) + " must contain a '" + list_name + "' list.")

    largest_id = 0
    for item in data[list_name]:
        if not isinstance(item, dict):
            raise ValueError("Every item in " + str(path) + " must be a JSON object.")
        item_id = item.get("id", 0)
        if isinstance(item_id, int) and item_id > largest_id:
            largest_id = item_id

    next_id = data.get(next_id_name, 1)
    if not isinstance(next_id, int) or next_id <= largest_id:
        next_id = largest_id + 1
    data[next_id_name] = next_id
    return data


def write_json_file(path, data):
    """Write pretty JSON in the same shape that serde_json writes."""
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as file:
        json.dump(data, file, indent=2, ensure_ascii=False)


def safe_next_id(items, requested_next_id):
    """Return an unused positive ID for a list of records."""
    largest_id = 0
    for item in items:
        item_id = item.get("id", 0)
        if isinstance(item_id, int) and item_id > largest_id:
            largest_id = item_id
    if not isinstance(requested_next_id, int) or requested_next_id <= largest_id:
        return largest_id + 1
    return requested_next_id


class DatasetStore:
    """Own the three paths used by one notebook dataset."""

    def __init__(self, data_directory=None):
        if data_directory is None:
            data_directory = default_data_directory()
        self.set_data_directory(data_directory)

    def set_data_directory(self, data_directory):
        data_directory = Path(data_directory).expanduser().resolve()
        self.data_directory = data_directory
        self.molecules_path = data_directory / MOLECULES_FILE_NAME
        self.samples_path = data_directory / SAMPLES_FILE_NAME
        self.observations_path = data_directory / OBSERVATIONS_FILE_NAME

    def load_molecules(self):
        return read_json_file(self.molecules_path, "molecules", "next_mol_id")

    def load_samples(self):
        return read_json_file(self.samples_path, "samples", "next_sample_id")

    def load_observations(self):
        return read_json_file(
            self.observations_path,
            "observations",
            "next_obs_id",
        )

    def save_molecules(self, data):
        write_json_file(self.molecules_path, data)

    def save_samples(self, data):
        write_json_file(self.samples_path, data)

    def save_observations(self, data):
        write_json_file(self.observations_path, data)

    def import_legacy_file(self, legacy_path):
        """Convert the older combined lab_notebook.json into three files.

        The caller chooses the destination by constructing this store with the
        desired data directory.  Existing destination files are not silently
        replaced; this check keeps real experiment data safe.
        """
        destination_paths = [
            self.molecules_path,
            self.samples_path,
            self.observations_path,
        ]
        for path in destination_paths:
            if path.exists():
                raise FileExistsError(
                    "Import stopped because the destination already contains "
                    + path.name
                )

        with Path(legacy_path).open("r", encoding="utf-8") as file:
            old_data = json.load(file)

        needed_lists = ["molecules", "samples", "observations"]
        for name in needed_lists:
            if name not in old_data or not isinstance(old_data[name], list):
                raise ValueError("The legacy file has no valid '" + name + "' list.")

        molecules = []
        for old_molecule in old_data["molecules"]:
            molecule = dict(old_molecule)
            if "melting_point" not in molecule:
                molecule["melting_point"] = None
            molecules.append(molecule)

        molecule_by_id = {}
        for molecule in molecules:
            molecule_by_id[molecule.get("id")] = molecule

        samples = []
        for old_sample in old_data["samples"]:
            sample = dict(old_sample)
            if "type" not in sample:
                sample["type"] = sample.pop("type_", "Binary")

            component_1 = molecule_by_id.get(sample.get("c1_id"), {})
            component_2 = molecule_by_id.get(sample.get("c2_id"), {})
            if "c1_cas" not in sample:
                sample["c1_cas"] = component_1.get("cas_number", "")
            if "c2_cas" not in sample:
                if sample.get("c2_id") is None:
                    sample["c2_cas"] = None
                else:
                    sample["c2_cas"] = component_2.get("cas_number", "")
            samples.append(sample)

        sample_by_id = {}
        for sample in samples:
            sample_by_id[sample.get("id")] = sample

        observations = []
        for old_observation in old_data["observations"]:
            observation = dict(old_observation)
            if "note" not in observation:
                observation["note"] = None
            if "sample_snapshot" not in observation:
                sample = sample_by_id.get(observation.get("sample_id"))
                observation["sample_snapshot"] = dict(sample) if sample else None
            observations.append(observation)

        molecules_data = {
            "molecules": molecules,
            "next_mol_id": safe_next_id(molecules, old_data.get("next_mol_id", 1)),
        }
        samples_data = {
            "samples": samples,
            "next_sample_id": safe_next_id(samples, old_data.get("next_sample_id", 1)),
        }
        observations_data = {
            "observations": observations,
            "next_obs_id": safe_next_id(
                observations,
                old_data.get("next_obs_id", 1),
            ),
        }

        self.data_directory.mkdir(parents=True, exist_ok=True)
        self.save_molecules(molecules_data)
        self.save_samples(samples_data)
        self.save_observations(observations_data)
