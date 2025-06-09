#!/usr/bin/env zsh
# Sets up all examples with proper linking to the local package

set -euo pipefail

# Colors
autoload -U colors && colors

# Directories
SCRIPT_DIR=${0:a:h}
JS_DIR=${SCRIPT_DIR:h}
ROOT_DIR=${JS_DIR:h}

# Print colored message
log() {
    local level=$1
    shift
    case $level in
        info)  echo "$fg_bold[blue]→$reset_color $@" ;;
        ok)    echo "$fg_bold[green]✓$reset_color $@" ;;
        error) echo "$fg_bold[red]✗$reset_color $@" >&2 ;;
    esac
}

# Check if WASM is built
check_wasm_built() {
    if [[ ! -d "$JS_DIR/dist/bundler" ]]; then
        log error "WASM packages not built!"
        log info "Run this first: $ROOT_DIR/build.zsh build-wasm"
        exit 1
    fi
}

# Setup an example
setup_example() {
    local example=$1
    local example_dir=$SCRIPT_DIR/$example

    if [[ ! -d $example_dir ]]; then
        log error "Example directory not found: $example_dir"
        return 1
    fi

    log info "Setting up $example example..."

    cd $example_dir

    # Clean previous installation
    rm -rf node_modules package-lock.json

    # Install dependencies
    npm install

    log ok "$example example ready!"
}

# Main setup
main() {
    log info "Setting up Beamterm examples..."

    # Check prerequisites
    check_wasm_built

    # Setup each example
    local examples=(webpack vite)

    for example in $examples; do
        if [[ -d $SCRIPT_DIR/$example ]]; then
            setup_example $example
        fi
    done

    log ok "All examples set up!"

    echo
    log info "To run an example:"
    echo "  CDN:     cd $SCRIPT_DIR/cdn && npm start"
    echo "  Webpack: cd $SCRIPT_DIR/webpack && npm start"
    echo "  Vite:    cd $SCRIPT_DIR/vite && npm run dev"
}

main
