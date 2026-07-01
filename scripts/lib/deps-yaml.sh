#!/usr/bin/env bash
# Lightweight YAML list helpers for deps/deps.yml (no external parser required).

deps_yaml_list() {
    local file="$1"
    local key="$2"
    awk -v key="${key}:" '
        $0 ~ "^" key "$" || $0 ~ "^" key " " { found=1; next }
        found && /^  - / {
            line=$0
            sub(/^  - /, "", line)
            gsub(/^["'\''"]|["'\''"]$/, "", line)
            print line
            next
        }
        found && /^[^ #]/ && !/^  - / { exit }
    ' "$file"
}

deps_yaml_nested_list() {
    local file="$1"
    local section="$2"
    local key="$3"
    awk -v section="${section}:" -v key="${key}:" '
        $0 ~ "^" section "$" || $0 ~ "^" section " " { in_section=1; next }
        in_section && ($0 ~ "^  " key "$" || $0 ~ "^  " key " ") { in_list=1; next }
        in_section && in_list && /^    - / {
            line=$0
            sub(/^    - /, "", line)
            gsub(/^["'\''"]|["'\''"]$/, "", line)
            print line
            next
        }
        in_section && in_list && /^  [^ ]/ && !/^    - / { in_list=0 }
        in_section && /^[^ #]/ && !/^  / { in_section=0; in_list=0 }
    ' "$file"
}

deps_yaml_value() {
    local file="$1"
    local key="$2"
    awk -v key="${key}:" '
        $0 ~ "^" key " " {
            sub(/^[^:]*:[[:space:]]*/, "")
            gsub(/^["'\''"]|["'\''"]$/, "", $0)
            print $0
            exit
        }
    ' "$file"
}

deps_yaml_nested_value() {
    local file="$1"
    local section="$2"
    local key="$3"
    awk -v section="${section}:" -v key="${key}:" '
        $0 ~ "^" section "$" || $0 ~ "^" section " " { in_section=1; next }
        in_section && ($0 ~ "^  " key " ") {
            sub(/^[^:]*:[[:space:]]*/, "")
            gsub(/^["'\''"]|["'\''"]$/, "", $0)
            print $0
            exit
        }
        in_section && /^[^ #]/ && !/^  / { exit }
    ' "$file"
}

deps_yaml_deep_list() {
    local file="$1"
    local section="$2"
    local subsection="$3"
    local key="$4"
    awk -v section="${section}:" -v subsection="${subsection}:" -v key="${key}:" '
        $0 ~ "^" section "$" || $0 ~ "^" section " " { in_section=1; next }
        in_section && ($0 ~ "^  " subsection "$" || $0 ~ "^  " subsection " ") { in_sub=1; next }
        in_sub && ($0 ~ "^    " key "$" || $0 ~ "^    " key " ") { in_list=1; next }
        in_sub && in_list && /^      - / {
            line=$0
            sub(/^      - /, "", line)
            gsub(/^["'\''"]|["'\''"]$/, "", line)
            print line
            next
        }
        in_sub && in_list && /^    [^ ]/ && !/^      - / { in_list=0 }
        in_sub && /^  [^ ]/ && !/^    / { in_sub=0; in_list=0 }
        in_section && /^[^ #]/ && !/^  / { exit }
    ' "$file"
}

detect_pkg_manager() {
    local configured="$1"
    if [[ "$configured" == "apt" ]]; then
        echo "apt"
        return
    fi
    if [[ "$configured" == "pacman" ]]; then
        echo "pacman"
        return
    fi
    if command -v pacman >/dev/null 2>&1; then
        echo "pacman"
    elif command -v apt-get >/dev/null 2>&1; then
        echo "apt"
    else
        echo "none"
    fi
}

deps_yaml_deep_value() {
    local file="$1"
    local section="$2"
    local subsection="$3"
    local key="$4"
    awk -v section="${section}:" -v subsection="${subsection}:" -v key="${key}:" '
        $0 ~ "^" section "$" || $0 ~ "^" section " " { in_section=1; next }
        in_section && ($0 ~ "^  " subsection "$" || $0 ~ "^  " subsection " ") { in_sub=1; next }
        in_sub && ($0 ~ "^    " key " ") {
            sub(/^[^:]*:[[:space:]]*/, "")
            gsub(/^["'\''"]|["'\''"]$/, "", $0)
            print $0
            exit
        }
        in_sub && /^  [^ ]/ && !/^    / { exit }
        in_section && /^[^ #]/ && !/^  / { exit }
    ' "$file"
}

deps_yaml_bool() {
    local value
    value="$(printf '%s' "$1" | tr '[:upper:]' '[:lower:]')"
    [[ "$value" == "true" || "$value" == "1" || "$value" == "yes" ]]
}
