#!/usr/bin/env bash
# Check for vendor dependency updates and optionally apply them.
#
# Modes:
#   (no flags)   — show current vs available versions, highlight major bumps
#   --apply      — update patch/minor versions in vendor-versions.json, then re-download
#   --allow-major — like --apply but also applies major version bumps (manual review advised)
#
# Requirements: npm, jq
# Usage:
#   bash scripts/update-vendor.sh               # check only
#   bash scripts/update-vendor.sh --apply        # apply safe (patch/minor) updates
#   bash scripts/update-vendor.sh --allow-major  # apply all updates incl. major

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
VERSIONS_FILE="$SCRIPT_DIR/vendor-versions.json"
DOWNLOAD_SCRIPT="$SCRIPT_DIR/download-vendor.sh"

APPLY=false
ALLOW_MAJOR=false
for arg in "$@"; do
    case "$arg" in
        --apply)       APPLY=true ;;
        --allow-major) APPLY=true; ALLOW_MAJOR=true ;;
    esac
done

if ! command -v bun &>/dev/null; then
    echo "❌ bun is required. Install from https://bun.sh"; exit 1
fi
if ! command -v jq &>/dev/null; then
    echo "❌ jq is required: brew install jq"; exit 1
fi

# Colour helpers (gracefully degrade if no tty)
if [ -t 1 ]; then
    RED='\033[0;31m'; YELLOW='\033[1;33m'; GREEN='\033[0;32m'
    CYAN='\033[0;36m'; BOLD='\033[1m'; RESET='\033[0m'
else
    RED=''; YELLOW=''; GREEN=''; CYAN=''; BOLD=''; RESET=''
fi

# semver helpers
normalize_ver() {
    # Strip any "pkgname@" prefix, return the bare semver
    echo "$1" | grep -oE '[0-9]+\.[0-9]+\.[0-9]+([.-][a-zA-Z0-9]+)?' | head -1
}
major() { echo "$1" | cut -d. -f1; }
minor() { echo "$1" | cut -d. -f2; }
patch() { echo "$1" | cut -d. -f3 | cut -d- -f1; }

# Build the "same major" range, handling major=0 (where ^0.0.0 is too broad)
same_major_range() {
    local ver="$1" maj="$2"
    if [ "$maj" = "0" ]; then
        echo "^0.$(minor "$ver").0"
    else
        echo "^${maj}.0.0"
    fi
}

semver_gt() {
    # returns 0 (true) if $1 > $2
    local a="$1" b="$2"
    [ "$(printf '%s\n%s' "$a" "$b" | sort -V | tail -1)" = "$a" ] && [ "$a" != "$b" ]
}

fetch_latest_in_major() {
    local pkg="$1" range="$2"
    npm view "${pkg}@${range}" version --json 2>/dev/null \
        | jq -r 'if type == "array" then last else . end' 2>/dev/null || echo ""
}

fetch_latest_overall() {
    local pkg="$1"
    npm view "${pkg}" version --json 2>/dev/null \
        | jq -r 'if type == "array" then last else . end' 2>/dev/null || echo ""
}

echo ""
echo -e "${BOLD}Vendor dependency update check${RESET}"
echo -e "Versions file: ${CYAN}$VERSIONS_FILE${RESET}"
echo ""
printf "%-42s  %-12s  %-14s  %-14s  %s\n" "Package" "Current" "Latest(same)" "Latest(any)" "Status"
printf "%-42s  %-12s  %-14s  %-14s  %s\n" "-------" "-------" "------------" "-----------" "------"

# Packages with special coupling that shouldn't be bumped independently
COUPLED_PACKAGES=("react" "react-dom" "@excalidraw/excalidraw")

HAS_SAFE_UPDATE=false
HAS_MAJOR_UPDATE=false
NEW_VERSIONS_FILE="$(mktemp)"  # pkg=version lines (bash 3 compatible)

while IFS= read -r pkg; do
    current=$(normalize_ver "$(jq -r --arg p "$pkg" '.dependencies[$p]' "$VERSIONS_FILE")")
    cur_major=$(major "$current")
    cur_range=$(same_major_range "$current" "$cur_major")

    # Query npm (in parallel would be nicer but keep it simple)
    latest_same=$(fetch_latest_in_major "$pkg" "$cur_range")
    latest_all=$(fetch_latest_overall "$pkg")

    # Normalise empties
    [ -z "$latest_same" ] && latest_same="$current"
    [ -z "$latest_all"  ] && latest_all="$current"

    # Determine status
    same_major_latest=$(major "$latest_all")
    is_major_bump=false
    [ "$same_major_latest" != "$cur_major" ] && is_major_bump=true

    is_coupled=false
    for cp in "${COUPLED_PACKAGES[@]}"; do
        [ "$cp" = "$pkg" ] && is_coupled=true && break
    done

    if [ "$current" = "$latest_all" ]; then
        status="${GREEN}up to date${RESET}"
    elif [ "$is_major_bump" = true ]; then
        status="${RED}⚠ major bump (manual review)${RESET}"
        HAS_MAJOR_UPDATE=true
        if $ALLOW_MAJOR && ! $is_coupled; then
            echo "$pkg=$latest_all" >> "$NEW_VERSIONS_FILE"
        elif $ALLOW_MAJOR && $is_coupled; then
            status="${RED}⚠ major + coupled (edit manually)${RESET}"
        fi
    elif semver_gt "$latest_same" "$current" 2>/dev/null; then
        status="${YELLOW}patch/minor available${RESET}"
        HAS_SAFE_UPDATE=true
        if $APPLY; then
            echo "$pkg=$latest_same" >> "$NEW_VERSIONS_FILE"
        fi
    else
        status="${GREEN}up to date${RESET}"
    fi

    printf "%-42s  %-12s  %-14s  %-14s  " "$pkg" "$current" "$latest_same" "$latest_all"
    echo -e "$status"

done < <(jq -r '.dependencies | keys[]' "$VERSIONS_FILE")

echo ""

# ── Apply updates ──────────────────────────────────────────────────────────
NEW_VERSIONS_COUNT=$(wc -l < "$NEW_VERSIONS_FILE" | tr -d ' ')
if $APPLY && [ "$NEW_VERSIONS_COUNT" -gt 0 ]; then
    echo -e "${BOLD}Applying updates to $VERSIONS_FILE ...${RESET}"

    TMP_JSON="$(mktemp)"
    cp "$VERSIONS_FILE" "$TMP_JSON"

    while IFS='=' read -r pkg new_ver; do
        echo "  $pkg: $(jq -r --arg p "$pkg" '.dependencies[$p]' "$VERSIONS_FILE") → $new_ver"
        jq --arg p "$pkg" --arg v "$new_ver" \
            '.dependencies[$p] = $v' "$TMP_JSON" > "${TMP_JSON}.new" \
            && mv "${TMP_JSON}.new" "$TMP_JSON"
    done < "$NEW_VERSIONS_FILE"

    mv "$TMP_JSON" "$VERSIONS_FILE"
    rm -f "$NEW_VERSIONS_FILE"
    echo ""
    echo -e "${GREEN}✓ vendor-versions.json updated.${RESET}"
    echo ""
    echo "Re-downloading vendor files..."
    bash "$DOWNLOAD_SCRIPT"

elif $APPLY && [ "$NEW_VERSIONS_COUNT" -eq 0 ]; then
    echo -e "${GREEN}✓ Nothing to update.${RESET}"

else
    rm -f "$NEW_VERSIONS_FILE"
    # Advice
    if $HAS_SAFE_UPDATE; then
        echo -e "Run ${CYAN}bash scripts/update-vendor.sh --apply${RESET} to apply patch/minor updates."
    fi
    if $HAS_MAJOR_UPDATE; then
        echo -e "Run ${CYAN}bash scripts/update-vendor.sh --allow-major${RESET} to include major bumps."
        echo -e "${YELLOW}Review release notes before applying major bumps.${RESET}"
        echo -e "Coupled packages (react/react-dom/@excalidraw) must be updated together manually."
    fi
    if ! $HAS_SAFE_UPDATE && ! $HAS_MAJOR_UPDATE; then
        echo -e "${GREEN}✓ All vendor dependencies are up to date.${RESET}"
    fi
fi

echo ""
