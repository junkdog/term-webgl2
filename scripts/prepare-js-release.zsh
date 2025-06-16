#!/usr/bin/env zsh
# Prepare JS package for release

set -euo pipefail

# Colors
autoload -U colors && colors

# Directories
SCRIPT_DIR=${0:a:h}
ROOT_DIR=${SCRIPT_DIR:h}
JS_DIR=${ROOT_DIR}/js

# Print colored message
log() {
    local level=$1
    shift
    case $level in
        info)  echo "$fg_bold[blue]→$reset_color $@" ;;
        ok)    echo "$fg_bold[green]✓$reset_color $@" ;;
        error) echo "$fg_bold[red]✗$reset_color $@" >&2 ;;
        warn)  echo "$fg_bold[yellow]⚠$reset_color $@" ;;
    esac
}

# Main preparation
main() {
    log info "Preparing @beamterm/renderer for release..."
    echo

    # Copy LICENSE from root if it doesn't exist
    if [[ ! -f "$JS_DIR/LICENSE" ]]; then
        log info "Copying LICENSE from root..."
        cp "$ROOT_DIR/LICENSE" "$JS_DIR/LICENSE"
        log ok "LICENSE copied"
    else
        log ok "LICENSE already exists"
    fi

    # Ensure CHANGELOG exists
    if [[ ! -f "$JS_DIR/CHANGELOG.md" ]]; then
        log error "CHANGELOG.md is missing!"
        log info "Please create it or use the template"
        exit 1
    else
        log ok "CHANGELOG.md exists"
    fi

    # Check .npmignore
    if [[ ! -f "$JS_DIR/.npmignore" ]]; then
        log error ".npmignore is missing!"
        log info "This could include unwanted files in the package"
        exit 1
    else
        log ok ".npmignore exists"
    fi

    # Verify dist folder exists
    if [[ ! -d "$JS_DIR/dist" ]]; then
        log error "dist/ folder missing!"
        log info "Run: $ROOT_DIR/build.zsh build-wasm"
        exit 1
    else
        log ok "dist/ folder exists"
    fi

    # Check for required dist subdirectories
    local required_dirs=(bundler web nodejs cdn)
    local missing_dirs=()
    
    for dir in $required_dirs; do
        if [[ ! -d "$JS_DIR/dist/$dir" ]]; then
            missing_dirs+=($dir)
        fi
    done
    
    if [[ ${#missing_dirs[@]} -gt 0 ]]; then
        log error "Missing dist subdirectories: ${missing_dirs[*]}"
        log info "Run: $ROOT_DIR/build.zsh build-wasm"
        exit 1
    else
        log ok "All dist subdirectories present"
    fi

    # Check package size
    echo
    log info "Package contents preview:"
    cd "$JS_DIR"
    
    # Get package details
    local pack_output=$(npm pack --dry-run 2>&1)
    local file_count=$(echo "$pack_output" | grep -c "npm notice" || true)
    local package_size=$(echo "$pack_output" | grep "package size" | awk '{print $5, $6}' || echo "unknown")
    local unpacked_size=$(echo "$pack_output" | grep "unpacked size" | awk '{print $5, $6}' || echo "unknown")
    
    log info "Files in package: $file_count"
    log info "Package size: $package_size"
    log info "Unpacked size: $unpacked_size"

    # Verify WASM file is included
    if echo "$pack_output" | grep -q "\.wasm"; then
        log ok "WASM file included in package"
    else
        log warn "No WASM file found in package!"
        log info "This might indicate a build issue"
    fi

    # Check current version
    echo
    local current_version=$(node -p "require('./package.json').version")
    log info "Current version: $current_version"

    # Final status
    echo
    log ok "Package is ready for release!"
    echo
    log info "Next steps:"
    echo "  1. Update CHANGELOG.md with release notes"
    echo "  2. Run: npm version patch/minor/major"
    echo "  3. Run: npm publish"
    echo
}

# Run main
main "$@"
