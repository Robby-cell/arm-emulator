#!/usr/bin/env bash

BASE=$(dirname "$0")

function _remove_old_translations() {
    echo "Removing old translations..."

    base_dir="${BASE}/assets/translations"
    for file in $(ls ${base_dir} | grep -E 'app_(\w{2})\.qm$'); do
        translation_path="${base_dir}/${file}"
        echo "Removing ${translation_path}"
        rm "${translation_path}"
    done

    echo "Old translations removed."
}

_remove_old_translations

uv run "${BASE}/scripts/build_translations.py" --dir="${BASE}/gui" --translations_dir="${BASE}/assets/translations" \
    --languages \
    en ru pl es ar
