#!/bin/bash
#
# VLM Visual Bug Checker - Wrapper Script
#
# Usage:
#   ./scripts/vlm_check.sh                    # Take screenshot and quick check
#   ./scripts/vlm_check.sh --thorough         # Take screenshot and thorough check
#   ./scripts/vlm_check.sh screenshot.png     # Check existing screenshot
#   ./scripts/vlm_check.sh --full-suite       # Run complete visual test suite
#
# Environment:
#   ANTHROPIC_API_KEY - Required for API access
#   DISPLAY           - X display (default :10)
#

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
VLM_SCRIPT="$SCRIPT_DIR/vlm_check/visual_checker.py"
SCREENSHOT_DIR="$PROJECT_DIR/screenshots/vlm_check"
REPORT_DIR="$PROJECT_DIR/test_reports/vlm"

# Default settings
DISPLAY="${DISPLAY:-:10}"
LEVEL="standard"
GAME_TIMEOUT=20
SCREENSHOT_DELAY=10

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

usage() {
    cat << EOF
VLM Visual Bug Checker for Idle Factory

Usage: $(basename "$0") [OPTIONS] [screenshot.png]

Options:
  -q, --quick       Quick check (5 items, fastest)
  -s, --standard    Standard check (10 items, default)
  -t, --thorough    Thorough check (20+ items, slowest)
  -c, --conveyor    Conveyor-specific check
  -u, --ui          UI-specific check
  -k, --chunk       Chunk boundary check

  --full-suite      Run all check types on new screenshots
  --compare FILE    Compare with baseline screenshot
  --no-game         Don't launch game, use existing screenshot
  --delay SEC       Delay before screenshot (default: 10)

  -o, --output DIR  Output directory for reports
  -h, --help        Show this help

Examples:
  # Quick sanity check (launch game, take screenshot, check)
  ./scripts/vlm_check.sh --quick

  # Thorough check on existing screenshot
  ./scripts/vlm_check.sh --thorough screenshots/gameplay.png

  # Full visual test suite (all check types)
  ./scripts/vlm_check.sh --full-suite

  # Check conveyor system specifically
  ./scripts/vlm_check.sh --conveyor

Recommended Timing:
  - After model/texture changes: --thorough
  - After UI changes: --ui
  - After chunk/terrain changes: --chunk
  - After conveyor logic changes: --conveyor
  - Before release: --full-suite
  - Quick CI check: --quick
EOF
}

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[PASS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[FAIL]${NC} $1"
}

# Check prerequisites
check_prerequisites() {
    if [ -z "$ANTHROPIC_API_KEY" ]; then
        log_error "ANTHROPIC_API_KEY environment variable not set"
        echo "  Export your API key: export ANTHROPIC_API_KEY='your-key'"
        exit 1
    fi

    if ! command -v python3 &> /dev/null; then
        log_error "python3 not found"
        exit 1
    fi

    if ! python3 -c "import anthropic" 2>/dev/null; then
        log_warning "anthropic package not installed, installing..."
        pip3 install anthropic --quiet
    fi

    if ! command -v scrot &> /dev/null; then
        log_warning "scrot not found, installing..."
        sudo apt-get install -y scrot
    fi
}

# Take a screenshot of the running game
take_screenshot() {
    local output_path="$1"
    local delay="${2:-$SCREENSHOT_DELAY}"

    mkdir -p "$(dirname "$output_path")"

    log_info "Launching game..."
    cd "$PROJECT_DIR"

    # Kill any existing game instance
    pkill -f "idle_factory" 2>/dev/null || true
    sleep 1

    # Launch game
    DISPLAY="$DISPLAY" cargo run --release 2>/dev/null &
    GAME_PID=$!

    log_info "Waiting ${delay}s for game to start..."
    sleep "$delay"

    # Check if game is still running
    if ! kill -0 $GAME_PID 2>/dev/null; then
        log_error "Game crashed during startup"
        return 1
    fi

    log_info "Taking screenshot..."
    DISPLAY="$DISPLAY" scrot "$output_path"

    # Kill game
    kill $GAME_PID 2>/dev/null || true
    wait $GAME_PID 2>/dev/null || true

    if [ -f "$output_path" ]; then
        log_success "Screenshot saved: $output_path"
        return 0
    else
        log_error "Failed to save screenshot"
        return 1
    fi
}

# Run VLM check on a screenshot
run_check() {
    local image_path="$1"
    local level="$2"
    local output_json="$3"

    log_info "Running $level check on $image_path..."

    if [ -n "$output_json" ]; then
        python3 "$VLM_SCRIPT" "$image_path" --level "$level" --output "$output_json"
    else
        python3 "$VLM_SCRIPT" "$image_path" --level "$level"
    fi

    return $?
}

# Run full test suite
run_full_suite() {
    local timestamp=$(date +%Y%m%d_%H%M%S)
    local suite_dir="$REPORT_DIR/suite_$timestamp"
    mkdir -p "$suite_dir"

    log_info "Running full visual test suite..."

    # Take screenshots at different game states
    local screenshots=()

    # 1. Main menu / startup
    log_info "Capturing startup state..."
    take_screenshot "$suite_dir/01_startup.png" 8
    screenshots+=("$suite_dir/01_startup.png")

    # 2. Gameplay (after some time)
    log_info "Capturing gameplay state..."
    take_screenshot "$suite_dir/02_gameplay.png" 15
    screenshots+=("$suite_dir/02_gameplay.png")

    # Run all check types
    local levels=("standard" "conveyor" "ui" "chunk")
    local all_passed=true

    for screenshot in "${screenshots[@]}"; do
        if [ ! -f "$screenshot" ]; then
            continue
        fi

        for level in "${levels[@]}"; do
            local report_file="$suite_dir/$(basename "$screenshot" .png)_${level}.json"
            log_info "Running $level check on $(basename "$screenshot")..."

            if ! run_check "$screenshot" "$level" "$report_file"; then
                all_passed=false
            fi
        done
    done

    # Generate summary
    log_info "Generating summary..."
    cat << EOF > "$suite_dir/summary.txt"
VLM Visual Test Suite Report
Generated: $(date)
=====================================

Screenshots checked:
$(for s in "${screenshots[@]}"; do echo "  - $(basename "$s")"; done)

Check levels run:
$(for l in "${levels[@]}"; do echo "  - $l"; done)

Results:
$(find "$suite_dir" -name "*.json" -exec basename {} \; | while read f; do
    status=$(python3 -c "import json; print(json.load(open('$suite_dir/$f')).get('result',{}).get('status','?'))" 2>/dev/null || echo "ERROR")
    echo "  $f: $status"
done)

EOF

    cat "$suite_dir/summary.txt"

    if $all_passed; then
        log_success "Full suite completed: All checks passed"
        return 0
    else
        log_error "Full suite completed: Some checks failed"
        return 1
    fi
}

# Main
main() {
    local screenshot_path=""
    local no_game=false
    local full_suite=false

    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -q|--quick)
                LEVEL="quick"
                shift
                ;;
            -s|--standard)
                LEVEL="standard"
                shift
                ;;
            -t|--thorough)
                LEVEL="thorough"
                shift
                ;;
            -c|--conveyor)
                LEVEL="conveyor"
                shift
                ;;
            -u|--ui)
                LEVEL="ui"
                shift
                ;;
            -k|--chunk)
                LEVEL="chunk"
                shift
                ;;
            --full-suite)
                full_suite=true
                shift
                ;;
            --no-game)
                no_game=true
                shift
                ;;
            --delay)
                SCREENSHOT_DELAY="$2"
                shift 2
                ;;
            -o|--output)
                REPORT_DIR="$2"
                shift 2
                ;;
            -h|--help)
                usage
                exit 0
                ;;
            -*)
                echo "Unknown option: $1"
                usage
                exit 1
                ;;
            *)
                screenshot_path="$1"
                no_game=true
                shift
                ;;
        esac
    done

    # Check prerequisites
    check_prerequisites

    mkdir -p "$SCREENSHOT_DIR" "$REPORT_DIR"

    # Full suite mode
    if $full_suite; then
        run_full_suite
        exit $?
    fi

    # Single check mode
    if [ -z "$screenshot_path" ] && ! $no_game; then
        # Take new screenshot
        local timestamp=$(date +%Y%m%d_%H%M%S)
        screenshot_path="$SCREENSHOT_DIR/check_$timestamp.png"
        take_screenshot "$screenshot_path" "$SCREENSHOT_DELAY"
    fi

    if [ ! -f "$screenshot_path" ]; then
        log_error "Screenshot not found: $screenshot_path"
        exit 1
    fi

    # Run check
    local report_file="$REPORT_DIR/$(basename "$screenshot_path" .png)_${LEVEL}.json"
    run_check "$screenshot_path" "$LEVEL" "$report_file"
    exit $?
}

main "$@"
