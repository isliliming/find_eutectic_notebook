"""Business rules for the Eutectic Lab Notebook.

There is no GUI code here.  Keeping calculations and file changes separate
makes them easier to understand, test, and update.
"""

import csv
import math
from copy import deepcopy
from datetime import datetime, timezone
from pathlib import Path

try:
    from .storage import DatasetStore
except ImportError:
    from storage import DatasetStore


CSV_HEADER = [
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
]


def utc_now_text():
    """Return a timestamp compatible with chrono's JSON timestamps."""
    timestamp = datetime.now(timezone.utc).isoformat()
    return timestamp.replace("+00:00", "Z")


def date_for_csv(timestamp):
    """Turn an ISO timestamp into the date format used by the Rust export."""
    try:
        parsed = datetime.fromisoformat(timestamp.replace("Z", "+00:00"))
        return parsed.strftime("%Y-%m-%d %H:%M")
    except (AttributeError, ValueError):
        return str(timestamp)


def number(value, field_name, positive=False):
    """Convert a value to float and give a useful validation error."""
    try:
        result = float(str(value).strip())
    except (TypeError, ValueError):
        raise ValueError(field_name + " must be a number.")
    if not math.isfinite(result):
        raise ValueError(field_name + " must be a finite number.")
    if positive and result <= 0:
        raise ValueError(field_name + " must be a positive number.")
    return result


class LabNotebook:
    def __init__(self, data_directory=None):
        self.store = DatasetStore(data_directory)
        self.reload()

    def reload(self):
        self.molecules_data = self.store.load_molecules()
        self.samples_data = self.store.load_samples()
        self.observations_data = self.store.load_observations()

    @property
    def molecules(self):
        return self.molecules_data["molecules"]

    @property
    def samples(self):
        return self.samples_data["samples"]

    @property
    def observations(self):
        return self.observations_data["observations"]

    def use_data_directory(self, data_directory):
        old_directory = self.store.data_directory
        self.store.set_data_directory(data_directory)
        try:
            self.reload()
        except Exception:
            self.store.set_data_directory(old_directory)
            self.reload()
            raise

    def find_molecule(self, molecule_id):
        for molecule in self.molecules:
            if molecule.get("id") == molecule_id:
                return molecule
        return None

    def find_sample(self, sample_id):
        for sample in self.samples:
            if sample.get("id") == sample_id:
                return sample
        return None

    def molecules_with_role(self, role):
        result = []
        for molecule in self.molecules:
            if molecule.get("role") == role:
                result.append(molecule)
        return result

    def add_molecule(
        self,
        name,
        short_name,
        cas_number,
        molecular_weight,
        melting_point,
        role,
        supplier="",
        sku="",
        webpage="",
    ):
        name = name.strip()
        short_name = short_name.strip()
        cas_number = cas_number.strip()

        if not name:
            raise ValueError("Name is required.")
        if not short_name:
            raise ValueError("Short name is required.")
        if role not in ("HBA", "HBD"):
            raise ValueError("Role must be HBA or HBD.")

        if cas_number:
            for molecule in self.molecules:
                old_cas = str(molecule.get("cas_number", "")).strip()
                if old_cas.lower() == cas_number.lower():
                    raise ValueError("A molecule with this CAS number is already registered.")

        molecular_weight = number(molecular_weight, "MW", positive=True)
        if str(melting_point).strip():
            melting_point_value = number(melting_point, "Melting point")
            if melting_point_value <= 0:
                melting_point_value = None
        else:
            melting_point_value = None

        molecule_id = self.molecules_data["next_mol_id"]
        self.molecules_data["next_mol_id"] = molecule_id + 1
        molecule = {
            "id": molecule_id,
            "name": name,
            "short_name": short_name,
            "cas_number": cas_number,
            "mw": molecular_weight,
            "melting_point": melting_point_value,
            "role": role,
            "supplier": supplier.strip(),
            "sku": sku.strip(),
            "webpage": webpage.strip(),
        }
        self.molecules.append(molecule)
        self.store.save_molecules(self.molecules_data)
        return molecule

    def delete_molecule(self, molecule_id):
        old_length = len(self.molecules)
        self.molecules_data["molecules"] = [
            molecule
            for molecule in self.molecules
            if molecule.get("id") != molecule_id
        ]
        if len(self.molecules) == old_length:
            raise ValueError("Molecule not found.")
        self.store.save_molecules(self.molecules_data)

    def theoretical_mass(self, c1_id, c2_id, parts_1, parts_2, mass_1, mass_2):
        """Calculate the missing mass when exactly one mass is provided."""
        component_1 = self.find_molecule(c1_id)
        component_2 = self.find_molecule(c2_id)
        if component_1 is None or component_2 is None:
            return ""

        try:
            parts_1 = number(parts_1, "Acceptor parts", positive=True)
            parts_2 = number(parts_2, "Donor parts", positive=True)
        except ValueError:
            return ""

        try:
            mass_1_value = float(str(mass_1).strip())
        except ValueError:
            mass_1_value = 0.0
        try:
            mass_2_value = float(str(mass_2).strip())
        except ValueError:
            mass_2_value = 0.0

        has_mass_1 = mass_1_value > 0
        has_mass_2 = mass_2_value > 0
        if has_mass_1 == has_mass_2:
            return ""

        if has_mass_1:
            ratio = parts_2 / parts_1
            answer = (mass_1_value / component_1["mw"]) * ratio * component_2["mw"]
            return (
                "Theoretical mass for "
                + component_2["short_name"]
                + " (component 2) is "
                + format(answer, ".2f")
                + " mg."
            )

        inverse_ratio = parts_1 / parts_2
        answer = (mass_2_value / component_2["mw"]) * inverse_ratio * component_1["mw"]
        return (
            "Theoretical mass for "
            + component_1["short_name"]
            + " (component 1) is "
            + format(answer, ".2f")
            + " mg."
        )

    def add_binary_sample(self, c1_id, c2_id, parts_1, parts_2, mass_1, mass_2):
        component_1 = self.find_molecule(c1_id)
        component_2 = self.find_molecule(c2_id)
        if component_1 is None:
            raise ValueError("Select an acceptor.")
        if component_2 is None:
            raise ValueError("Select a donor.")
        if c1_id == c2_id:
            raise ValueError("Select two different molecules.")

        mass_1 = number(mass_1, "Mass 1", positive=True)
        mass_2 = number(mass_2, "Mass 2", positive=True)
        parts_1_text = str(parts_1).strip()
        parts_2_text = str(parts_2).strip()

        moles_1 = mass_1 / component_1["mw"]
        moles_2 = mass_2 / component_2["mw"]
        real_ratio = moles_2 / moles_1

        sample_id = self.samples_data["next_sample_id"]
        self.samples_data["next_sample_id"] = sample_id + 1
        sample = {
            "id": sample_id,
            "type": "Binary",
            "c1_id": component_1["id"],
            "c1_name": component_1["short_name"],
            "c1_cas": component_1.get("cas_number", ""),
            "c1_mass": mass_1,
            "c2_id": component_2["id"],
            "c2_name": component_2["short_name"],
            "c2_cas": component_2.get("cas_number", ""),
            "c2_mass": mass_2,
            "ratio_label": parts_1_text + ":" + parts_2_text,
            "real_molar_ratio": real_ratio,
            "created_at": utc_now_text(),
        }
        self.samples.append(sample)
        self.store.save_samples(self.samples_data)
        return sample

    def add_pure_sample(self, molecule_id, mass):
        molecule = self.find_molecule(molecule_id)
        if molecule is None:
            raise ValueError("Select a molecule.")
        mass = number(mass, "Mass", positive=True)

        sample_id = self.samples_data["next_sample_id"]
        self.samples_data["next_sample_id"] = sample_id + 1
        sample = {
            "id": sample_id,
            "type": "Pure",
            "c1_id": molecule["id"],
            "c1_name": molecule["short_name"],
            "c1_cas": molecule.get("cas_number", ""),
            "c1_mass": mass,
            "c2_id": None,
            "c2_name": None,
            "c2_cas": None,
            "c2_mass": None,
            "ratio_label": "Pure",
            "real_molar_ratio": None,
            "created_at": utc_now_text(),
        }
        self.samples.append(sample)
        self.store.save_samples(self.samples_data)
        return sample

    def delete_sample(self, sample_id):
        old_length = len(self.samples)
        self.samples_data["samples"] = [
            sample for sample in self.samples if sample.get("id") != sample_id
        ]
        if len(self.samples) == old_length:
            raise ValueError("Sample not found.")

        self.observations_data["observations"] = [
            observation
            for observation in self.observations
            if observation.get("sample_id") != sample_id
        ]
        self.store.save_samples(self.samples_data)
        self.store.save_observations(self.observations_data)

    def add_observation(self, sample_id, temperature, result, note=""):
        sample = self.find_sample(sample_id)
        if sample is None:
            raise ValueError("Select a sample.")
        temperature = number(temperature, "Temperature")
        if result not in ("Liquid", "Solid"):
            raise ValueError("Result must be Liquid or Solid.")

        observation_id = self.observations_data["next_obs_id"]
        self.observations_data["next_obs_id"] = observation_id + 1
        observation = {
            "id": observation_id,
            "sample_id": sample_id,
            "temperature": temperature,
            "result": result,
            "timestamp": utc_now_text(),
            "note": note.strip() or None,
            "sample_snapshot": deepcopy(sample),
        }
        self.observations.append(observation)
        self.store.save_observations(self.observations_data)
        return observation

    def delete_observation(self, observation_id):
        old_length = len(self.observations)
        self.observations_data["observations"] = [
            observation
            for observation in self.observations
            if observation.get("id") != observation_id
        ]
        if len(self.observations) == old_length:
            raise ValueError("Observation not found.")
        self.store.save_observations(self.observations_data)

    def sample_type(self, sample):
        return sample.get("type", sample.get("type_", "Unknown"))

    def sample_label(self, sample):
        if self.sample_type(sample) == "Pure":
            return "#" + str(sample["id"]) + " Pure " + sample["c1_name"]
        return (
            "#"
            + str(sample["id"])
            + " "
            + sample["c1_name"]
            + " + "
            + str(sample.get("c2_name") or "?")
            + " ("
            + sample.get("ratio_label", "")
            + ")"
        )

    def real_ratio_display(self, sample):
        if self.sample_type(sample) != "Binary":
            return "N/A"
        real_ratio = sample.get("real_molar_ratio")
        ratio_label = sample.get("ratio_label", "")
        if real_ratio is None or ":" not in ratio_label:
            return "N/A"
        first_part = ratio_label.split(":", 1)[0].strip()
        try:
            displayed_second = float(real_ratio) * float(first_part)
        except (TypeError, ValueError):
            return "N/A"
        return first_part + ":" + format(displayed_second, ".3f")

    def export_csv(self, path, observation_ids=None):
        """Export all observations, or only IDs supplied by the caller."""
        selected_ids = None
        if observation_ids is not None:
            selected_ids = set(observation_ids)

        path = Path(path)
        path.parent.mkdir(parents=True, exist_ok=True)
        with path.open("w", encoding="utf-8", newline="") as file:
            writer = csv.writer(file)
            writer.writerow(CSV_HEADER)
            for observation in self.observations:
                if selected_ids is not None and observation.get("id") not in selected_ids:
                    continue
                writer.writerow(self.csv_row(observation))

    def csv_row(self, observation):
        sample = self.find_sample(observation.get("sample_id"))
        if sample is None:
            sample = observation.get("sample_snapshot")

        date_text = date_for_csv(observation.get("timestamp", ""))
        temperature = format(float(observation.get("temperature", 0)), ".2f")
        result = observation.get("result", "")
        note = observation.get("note") or ""

        if sample is None:
            return [
                date_text,
                observation.get("id", ""),
                observation.get("sample_id", ""),
                "Unknown",
                "",
                "",
                "",
                "",
                "",
                "",
                temperature,
                result,
                note,
            ]

        mass_2 = ""
        if sample.get("c2_mass") is not None:
            mass_2 = format(float(sample["c2_mass"]), ".3f")
        return [
            date_text,
            observation.get("id", ""),
            sample.get("id", ""),
            self.sample_type(sample),
            sample.get("c1_name", ""),
            format(float(sample.get("c1_mass", 0)), ".3f"),
            sample.get("c2_name") or "",
            mass_2,
            sample.get("ratio_label", ""),
            self.real_ratio_display(sample),
            temperature,
            result,
            note,
        ]
