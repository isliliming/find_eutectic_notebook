# Eutectic Lab Notebook — Python

This folder is a beginner-friendly Python rewrite of the Rust application in
the repository root. It provides the same four workflows:

1. Register and search molecules.
2. Prepare binary mixtures or pure samples.
3. Record liquid/solid observations.
4. Review, delete, select, and export observations.

The GUI uses only Python's standard `tkinter` library. The Python interpreter
and virtual environment are managed by [uv](https://docs.astral.sh/uv/). There
are no third-party Python packages.

## Set up the environment

Install uv if it is not already installed, then run these commands from the
repository root:

```bash
cd python_eutectic_app
uv sync
```

The project is pinned to Python 3.13 in `.python-version`, and uv creates `.venv`
inside this folder. Because this is a Tkinter desktop app, `pyproject.toml`
instructs uv to use a system Python that includes the operating system's Tcl/Tk
runtime. On this Mac, uv uses Homebrew Python 3.13 with `python-tk@3.13`.

uv will report an error instead of downloading a standalone Python if no system
Python 3.13 with Tkinter is available. On macOS with Homebrew, install it with:

```bash
brew install python-tk@3.13
```

On another operating system, install Python 3.13 and its Tkinter support using
the operating system's Python installer, then run `uv sync` again.

You can check the environment with:

```bash
uv run python -c "import sys, tkinter; print(sys.version); print('Tkinter', tkinter.TkVersion)"
```

## Start the app

From `python_eutectic_app`, run:

```bash
uv run python main.py
```

Run `uv sync` again after changing `pyproject.toml` or pulling dependency
updates. You do not need to activate `.venv`; `uv run` selects it automatically.

By default, the Python and Rust projects use the same current dataset files:

```text
lab_data/molecules.json
lab_data/samples.json
lab_data/observations.json
```

The Python app only writes a file when you add or delete data. Opening the app
does not rewrite the files.

To start with a different three-file dataset:

```bash
uv run python main.py --data-dir /path/to/lab_data
```

You can also change the data folder from the toolbar. If the three files do not
exist yet, they are created when the first record of each kind is saved.

> Before using either the Rust app or Python app on important experiment data,
> keep a backup. Do not edit the same files in both applications at the same
> time, because the last application to save will win.

## Use the real older example dataset

`eutectic_app/lab_notebook.json` was made by an older Rust version. That version
stored all records in one file and used `type_`; the current Rust version stores
three files and uses `type`. The Python app includes a safe importer for it.

In the GUI, click **Import old lab_notebook.json...**, choose the old file, then
choose a new empty destination folder. The original file is never modified.

The same conversion can be run in the terminal:

```bash
uv run python import_legacy.py \
  ../eutectic_app/lab_notebook.json \
  /tmp/eutectic_imported_data
```

The importer refuses to replace any existing `molecules.json`, `samples.json`,
or `observations.json` file in the destination.

## Project structure

```text
python_eutectic_app/
├── pyproject.toml     # Python version and project dependencies
├── uv.lock            # Reproducible uv environment lockfile
├── app.py             # Tkinter widgets and button actions
├── notebook.py        # Validation, calculations, records, and CSV export
├── storage.py         # Current JSON format and old-data importer
├── main.py            # Start command and --data-dir option
├── import_legacy.py   # Optional command-line conversion tool
└── tests/             # Standard-library unit tests
```

When changing calculations or record rules, edit `notebook.py`. When changing
the JSON layout, edit `storage.py`. When changing only appearance, edit
`app.py`. This separation helps prevent a GUI change from accidentally changing
the experiment data format.

## Run the tests

From `python_eutectic_app`:

```bash
uv run python -m unittest discover -s tests -v
```

The tests use temporary folders. They do not write to `lab_data` or the real
example dataset.
