"""Convert an old combined lab_notebook.json without opening the GUI."""

import argparse

try:
    from .storage import DatasetStore
except ImportError:
    from storage import DatasetStore


def main():
    parser = argparse.ArgumentParser(
        description="Convert lab_notebook.json to the current three-file format."
    )
    parser.add_argument("legacy_file", help="Path to the old lab_notebook.json")
    parser.add_argument("destination", help="Empty folder for the converted JSON files")
    arguments = parser.parse_args()

    store = DatasetStore(arguments.destination)
    store.import_legacy_file(arguments.legacy_file)
    print("Imported dataset into " + str(store.data_directory))


if __name__ == "__main__":
    main()

