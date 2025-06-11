#!/usr/bin/env node

const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

const TARGETS = ['bundler', 'web', 'nodejs'];
const ROOT_DIR = path.join(__dirname, '..');
const RENDERER_DIR = path.join(ROOT_DIR, 'beamterm-renderer');
const TARGET_DIR = path.join(ROOT_DIR, 'target');
const JS_DIST_DIR = path.join(__dirname, 'dist');

// Ensure directories exist
fs.mkdirSync(JS_DIST_DIR, { recursive: true });

// Build each target
TARGETS.forEach(target => {
    console.log(`\n📦 Building ${target} package...`);

    const outDir = path.join(TARGET_DIR, 'wasm-pack', target);

    // Run wasm-pack with output to target directory
    execSync(`wasm-pack build ${RENDERER_DIR} --target ${target} --out-dir ${outDir} --no-pack`, {
        stdio: 'inherit',
        cwd: ROOT_DIR
    });

    // Copy to js/dist for packaging
    const distDir = path.join(JS_DIST_DIR, target);
    fs.mkdirSync(distDir, { recursive: true });

    // Copy all generated files
    const files = fs.readdirSync(outDir);
    files.forEach(file => {
        if (!file.includes('gitignore') && !file.includes('.json')) {
            fs.copyFileSync(
                path.join(outDir, file),
                path.join(distDir, file)
            );
        }
    });

    console.log(`✅ ${target} package built`);
});

// Build CDN bundle
console.log('\n📦 Building CDN bundle...');
const cdnDir = path.join(JS_DIST_DIR, 'cdn');
fs.mkdirSync(cdnDir, { recursive: true });

// Create wrapper for CDN
const cdnWrapper = `
// Auto-generated CDN wrapper
import init, * as BeamtermExports from '../web/beamterm_renderer.js';

let initialized = false;
let initPromise = null;

async function ensureInitialized() {
    if (!initialized) {
        if (!initPromise) {
            initPromise = init();
        }
        await initPromise;
        initialized = true;
    }
}

// Wrap exports to ensure initialization
const Beamterm = {
    async createRenderer(canvasId) {
        await ensureInitialized();
        return new BeamtermExports.BeamtermRenderer(canvasId);
    },
    Batch: BeamtermExports.Batch,
    CellStyle: BeamtermExports.CellStyle,
    Size: BeamtermExports.Size,
    Span: BeamtermExports.Span,
    // Re-export other classes as needed
};

// For IIFE build
if (typeof window !== 'undefined') {
    window.Beamterm = Beamterm;
}

export default Beamterm;
`;

fs.writeFileSync(path.join(cdnDir, 'wrapper.js'), cdnWrapper);

// Bundle with esbuild
execSync(`npx esbuild ${cdnDir}/wrapper.js --bundle --format=iife --global-name=Beamterm --outfile=${cdnDir}/beamterm.min.js --minify`, {
    stdio: 'inherit'
});

console.log('✅ CDN bundle built');
console.log('\n🎉 All packages built successfully!');