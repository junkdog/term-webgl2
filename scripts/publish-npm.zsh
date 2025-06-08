#!/usr/bin/env zsh
# Publishes the NPM package

set -euo pipefail

# Colors
autoload -U colors && colors

# Directories
SCRIPT_DIR=${0:a:h}
ROOT_DIR=${SCRIPT_DIR:h}
JS_DIR=$ROOT_DIR/js

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

# Check if logged in to npm
check_npm_auth() {
    if ! npm whoami &>/dev/null; then
        log error "Not logged in to npm"
        log info "Please run: npm login"
        exit 1
    fi

    local npm_user=$(npm whoami)
    log ok "Logged in as: $npm_user"
}

# Get version from Cargo.toml
get_version() {
    local cargo_toml=$ROOT_DIR/Cargo.toml
    local version=$(grep -E '^version\s*=' $cargo_toml | head -1 | cut -d'"' -f2)

    if [[ -z $version ]]; then
        log error "Could not extract version from Cargo.toml"
        exit 1
    fi

    echo $version
}

# Update package.json version
update_package_version() {
    local version=$1
    local package_json=$JS_DIR/package.json

    log info "Updating package.json version to $version..."

    # Use npm version to update (handles git tagging if needed)
    cd $JS_DIR
    npm version $version --no-git-tag-version --allow-same-version
}

# Main publish process
main() {
    log info "Starting NPM publish process..."

    # Change to JS directory
    cd $JS_DIR

    # Check authentication
    check_npm_auth

    # Get version
    local version=$(get_version)
    log info "Version: $version"

    # Update package version
    update_package_version $version

    # Build packages
    log info "Building packages..."
    $ROOT_DIR/build.zsh build-wasm

    # Dry run first
    log info "Running publish dry run..."
    npm publish --dry-run --access public

    # Confirm
    echo
    log warn "Ready to publish @beamterm/renderer@$version to NPM"
    echo -n "Continue? [y/N] "
    read -q || { echo; exit 1 }
    echo

    # Publish
    log info "Publishing to NPM..."
    npm publish --access public

    log ok "Successfully published @beamterm/renderer@$version!"

    # Show package info
    echo
    log info "Package info:"
    echo "  https://www.npmjs.com/package/@beamterm/renderer"
    echo "  npm install @beamterm/renderer"
    echo "  CDN: https://unpkg.com/@beamterm/renderer"
}

main
