#!/usr/bin/env zsh
# Diagnostic script to check WASM build status

set -euo pipefail

# Colors
autoload -U colors && colors

# Directories
SCRIPT_DIR=${0:a:h}
ROOT_DIR=${SCRIPT_DIR:h}
JS_DIR=$ROOT_DIR/js
TARGET_DIR=$ROOT_DIR/target/wasm-pack

# Print colored message
log() {
    local level=$1
    shift
    case $level in
        info)  echo "$fg_bold[blue]→$reset_color $@" ;;
        ok)    echo "$fg_bold[green]✓$reset_color $@" ;;
        warn)  echo "$fg_bold[yellow]⚠$reset_color $@" ;;
        error) echo "$fg_bold[red]✗$reset_color $@" >&2 ;;
    esac
}

# Check if file exists and show size
check_file() {
    local file=$1
    local desc=$2
    
    if [[ -f $file ]]; then
        local size=$(ls -lh $file | awk '{print $5}')
        log ok "$desc ($size)"
        return 0
    else
        log error "$desc - NOT FOUND"
        return 1
    fi
}

# Check a target build
check_target() {
    local target=$1
    log info "Checking $target build..."
    
    local target_dir=$TARGET_DIR/$target
    local dist_dir=$JS_DIR/dist/$target
    
    # Check if directories exist
    if [[ ! -d $target_dir ]]; then
        log error "  Target directory not found: $target_dir"
        return 1
    fi
    
    if [[ ! -d $dist_dir ]]; then
        log error "  Dist directory not found: $dist_dir"
        return 1
    fi
    
    # Check for key files
    local has_error=0
    
    check_file "$dist_dir/beamterm_renderer.js" "  Main JS" || has_error=1
    check_file "$dist_dir/beamterm_renderer_bg.wasm" "  WASM file" || has_error=1
    check_file "$dist_dir/beamterm_renderer.d.ts" "  TypeScript definitions" || has_error=1
    
    if [[ $target == "bundler" ]]; then
        check_file "$dist_dir/beamterm_renderer_bg.js" "  WASM bindings" || has_error=1
    fi
    
    return $has_error
}

# Main diagnostic
main() {
    log info "Beamterm WASM Build Diagnostics"
    echo
    
    # Check if wasm-pack is installed
    if command -v wasm-pack &>/dev/null; then
        local version=$(wasm-pack --version)
        log ok "wasm-pack installed: $version"
    else
        log error "wasm-pack not installed!"
        log info "Install with: curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh"
        exit 1
    fi
    
    # Check if font atlas exists
    if [[ -f $ROOT_DIR/data/bitmap_font.atlas ]]; then
        log ok "Font atlas found"
    else
        log warn "Font atlas not found - run: ./build.zsh atlas"
    fi
    
    echo
    
    # Check each target
    local all_good=0
    for target in bundler web nodejs; do
        check_target $target || all_good=1
        echo
    done
    
    # Check CDN bundle
    log info "Checking CDN bundle..."
    check_file "$JS_DIR/dist/cdn/beamterm.min.js" "  CDN bundle" || all_good=1
    
    echo
    
    # Summary
    if [[ $all_good -eq 0 ]]; then
        log ok "All WASM builds look good!"
        
        echo
        log info "Test the builds:"
        echo "  1. Open: $JS_DIR/test/test-wasm.html"
        echo "  2. Or run: cd $JS_DIR/examples/vite && npm run dev"
    else
        log error "Some WASM builds are missing or incomplete"
        log info "Run: ./build.zsh build-wasm"
    fi
}

main
