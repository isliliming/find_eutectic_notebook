"""Command-line entry point for the Python Eutectic Lab Notebook."""

import argparse


def main():
    parser = argparse.ArgumentParser(description="Record binary eutectic experiments.")
    parser.add_argument(
        "--data-dir",
        help="Folder containing molecules.json, samples.json, and observations.json",
    )
    arguments = parser.parse_args()

    try:
        if __package__:
            from .app import run_app
        else:
            from app import run_app
    except ModuleNotFoundError as error:
        if error.name == "_tkinter":
            parser.error(
                "this Python installation has no Tkinter GUI support. "
                "Use a Python installation that includes Tkinter."
            )
        raise
    run_app(arguments.data_dir)


if __name__ == "__main__":
    main()
