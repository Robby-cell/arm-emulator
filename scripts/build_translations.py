import argparse
import os
import subprocess
import sys
import time
from pathlib import Path

# Third-party imports (for translation)
try:
    from deep_translator import GoogleTranslator
    from lxml import etree
except ImportError:
    print("❌ Error: Missing dependencies.")
    print("Please run: pip install lxml deep-translator")
    sys.exit(1)

# Configuration Defaults
SOURCE_LANG = "en"
TRANSLATIONS_DIR = Path("assets/translations")


def get_pyside_tool_command(tool_name):
    """
    Returns the command list to run a PySide6 tool reliably via python -m.
    """
    return [f"pyside6-{tool_name}"]


def find_sources(root_dir) -> list[str]:
    """
    1. Recursively finds all .py files.
    2. Generates a temporary .pro file for lupdate to use.
    """
    print(f"Scanning for Python files in '{root_dir}'...")

    py_files = list(Path(root_dir).rglob("*.py"))

    if not py_files:
        print("❌ No Python files found.")
        sys.exit(1)

    print(f"   Found {len(py_files)} source files.")

    output = [py_file.as_posix() for py_file in py_files]

    return output


def run_lupdate(sources: list[str], languages: list[str]):
    """
    Runs lupdate to extract strings from the sources defined in the .pro file
    into .ts files for each language.
    """
    print("\nRunning lupdate (Extracting strings)...")

    # Ensure directory exists
    TRANSLATIONS_DIR.mkdir(parents=True, exist_ok=True)

    ts_files = []
    for lang in languages:
        ts_file = TRANSLATIONS_DIR / f"app_{lang}.ts"
        ts_files.append(str(ts_file))

    # Command: python -m PySide6.lupdate temp.pro -ts file1.ts file2.ts ...
    cmd = get_pyside_tool_command("lupdate") + sources + ["-ts"] + ts_files

    try:
        subprocess.run(cmd, check=True)
        print("   lupdate complete.")
    except subprocess.CalledProcessError:
        print("   Error running lupdate.")
        sys.exit(1)

    return ts_files


def translate_single_file(ts_file_path, target_lang):
    """
    Parses a .ts file, finds unfinished translations, and translates them.
    """
    print(f"\nProcessing '{ts_file_path}' (Target: {target_lang.upper()})...")

    if target_lang == SOURCE_LANG:
        print("   Skipping translation (Target is source language).")
        return

    try:
        parser = etree.XMLParser(remove_blank_text=True)
        tree = etree.parse(ts_file_path, parser)
        root = tree.getroot()
    except Exception as e:
        print(f"   ❌ Error parsing XML: {e}")
        return

    # Initialize Translator
    try:
        translator = GoogleTranslator(source=SOURCE_LANG, target=target_lang)
    except Exception as e:
        print(f"   Invalid language code '{target_lang}': {e}")
        return

    count = 0
    errors = 0

    for message in root.findall(".//message"):
        source = message.find("source")
        translation = message.find("translation")

        if source is None or translation is None:
            continue

        original_text = source.text

        # Check if unfinished or empty
        is_unfinished = translation.get("type") == "unfinished"
        is_empty = translation.text is None or not translation.text.strip()

        if (is_unfinished or is_empty) and original_text:
            try:
                # Perform translation
                translated_text = translator.translate(original_text)

                # Update XML
                translation.text = translated_text

                # Mark as finished
                if "type" in translation.attrib:
                    del translation.attrib["type"]

                print(f"   Trans: '{original_text}' -> '{translated_text}'")
                count += 1
                time.sleep(0.1)  # Rate limiting

            except Exception as e:
                print(f"   Failed to translate: {e}")
                errors += 1

    if count > 0:
        tree.write(
            ts_file_path, encoding="utf-8", xml_declaration=True, pretty_print=True
        )
        print(f"   Saved {count} new translations.")
    else:
        print("   File is already up to date.")


def run_lrelease(ts_files):
    """
    Runs lrelease to compile .ts files into binary .qm files.
    """
    print("\nRunning lrelease (Compiling binaries)...")

    cmd = get_pyside_tool_command("lrelease") + ts_files

    try:
        subprocess.run(cmd, check=True)
        print("   lrelease complete. .qm files generated.")
    except subprocess.CalledProcessError:
        print("   Error running lrelease.")


def cleanup(): ...


def main():
    global TRANSLATIONS_DIR

    parser = argparse.ArgumentParser(description="Automated Qt Translation Pipeline")
    parser.add_argument(
        "--languages",
        nargs="+",
        required=True,
        help="List of target language codes (e.g. en es ru)",
    )
    parser.add_argument(
        "--dir", default="gui", help="Root directory to scan for python files"
    )
    parser.add_argument("--translations_dir", default="assets/translations", help="Translations directory")

    args = parser.parse_args()

    TRANSLATIONS_DIR = Path(args.translations_dir)

    # 1. Scan for files
    sources = find_sources(args.dir)

    try:
        # 2. Extract strings (lupdate)
        # We enforce 'en' (source) is always updated so we have a base file
        langs_to_process = set(args.languages)
        generated_ts_files = run_lupdate(sources, list(langs_to_process))

        # 3. Translate content
        for ts_file in generated_ts_files:
            # Extract lang code from filename "assets/translations/app_es.ts"
            filename = Path(ts_file).stem  # "app_es"
            lang_code = filename.split("_")[-1]  # "es"

            translate_single_file(ts_file, lang_code)

        # 4. Compile binaries (lrelease)
        run_lrelease(generated_ts_files)

    finally:
        cleanup()


if __name__ == "__main__":
    main()
