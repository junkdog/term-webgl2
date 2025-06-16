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
    execSync(`wasm-pack build ${RENDERER_DIR} --target ${target} --out-dir ${outDir} --no-pack --features js-api`, {
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

// First, copy the web build files to a temp location for bundling
const tempDir = path.join(JS_DIST_DIR, '.temp-cdn');
fs.mkdirSync(tempDir, { recursive: true });

// Copy web files to temp
fs.copyFileSync(
    path.join(JS_DIST_DIR, 'web', 'beamterm_renderer.js'),
    path.join(tempDir, 'beamterm_renderer.js')
);
fs.copyFileSync(
    path.join(JS_DIST_DIR, 'web', 'beamterm_renderer_bg.wasm'),
    path.join(tempDir, 'beamterm_renderer_bg.wasm')
);

// Create a CDN entry point that properly bundles everything
const cdnEntry = `
// CDN bundle for Beamterm
import init, * as BeamtermModule from './beamterm_renderer.js';

// Create a promise for initialization
let initPromise = null;
let initialized = false;

// Helper to ensure initialization
async function ensureInit(wasmUrl) {
    if (!initialized) {
        if (!initPromise) {
            if (wasmUrl) {
                initPromise = init(wasmUrl);
            } else {
                // Auto-detect WASM URL based on script location
                const currentScript = (typeof document !== 'undefined' && document.currentScript) 
                    ? document.currentScript.src 
                    : import.meta.url;
                const baseUrl = currentScript.substring(0, currentScript.lastIndexOf('/'));
                const autoWasmUrl = baseUrl + '/beamterm_bg.wasm';
                initPromise = init(autoWasmUrl);
            }
        }
        await initPromise;
        initialized = true;
    }
    return BeamtermModule;
}

// Create the public API
const Beamterm = {
    // Initialize with optional WASM path
    async init(wasmUrl) {
        await ensureInit(wasmUrl);
        return this;
    },
    
    // Direct constructor access (requires init first)
    get BeamtermRenderer() { 
        return initialized ? BeamtermModule.BeamtermRenderer : undefined;
    },
    
    // Helper functions need to be wrapped to ensure init
    style() {
        if (!initialized) {
            throw new Error('Beamterm not initialized. Call Beamterm.init() first or use await Beamterm.createRenderer()');
        }
        return BeamtermModule.style();
    },
    
    cell(symbol, style) {
        if (!initialized) {
            throw new Error('Beamterm not initialized. Call Beamterm.init() first or use await Beamterm.createRenderer()');
        }
        return BeamtermModule.cell(symbol, style);
    },
    
    // Expose classes (these will be undefined until init)
    get BeamtermRenderer() { return BeamtermModule.BeamtermRenderer; },
    get Batch() { return BeamtermModule.Batch; },
    get CellStyle() { return BeamtermModule.CellStyle; },
    get Size() { return BeamtermModule.Size; },
    
    // Version info
    version: '0.1.1',
    
    // For debugging
    get initialized() { return initialized; },
    get module() { return initialized ? BeamtermModule : null; }
};

// Auto-initialize if we're in a browser with a script tag
if (typeof window !== 'undefined' && typeof document !== 'undefined') {
    window.Beamterm = Beamterm;
    
    // For convenience, start initialization immediately
    // Users can still await Beamterm.init() or just await createRenderer
    ensureInit().catch(err => {
        console.warn('Beamterm auto-initialization failed:', err);
        console.warn('You may need to call Beamterm.init() with the correct WASM path');
    });
}

export default Beamterm;
`;

fs.writeFileSync(path.join(tempDir, 'cdn-entry.js'), cdnEntry);

// Bundle with esbuild
console.log('📦 Bundling with esbuild...');
execSync(`npx esbuild ${tempDir}/cdn-entry.js --bundle --format=iife --global-name=Beamterm --outfile=${cdnDir}/beamterm.min.js --minify --sourcemap --loader:.wasm=file`, {
    stdio: 'inherit'
});

// Copy WASM file to CDN directory with consistent name
fs.copyFileSync(
    path.join(tempDir, 'beamterm_renderer_bg.wasm'),
    path.join(cdnDir, 'beamterm_bg.wasm')
);

// Also copy with original name for compatibility
fs.copyFileSync(
    path.join(tempDir, 'beamterm_renderer_bg.wasm'),
    path.join(cdnDir, 'beamterm_renderer_bg.wasm')
);

// Clean up temp directory
fs.rmSync(tempDir, { recursive: true, force: true });

console.log('✅ CDN bundle built');
console.log('\n🎉 All packages built successfully!');
console.log('\nCDN files:');
console.log(`  ${cdnDir}/beamterm.min.js (${(fs.statSync(path.join(cdnDir, 'beamterm.min.js')).size / 1024).toFixed(1)} KB)`);
console.log(`  ${cdnDir}/beamterm_bg.wasm (${(fs.statSync(path.join(cdnDir, 'beamterm_bg.wasm')).size / 1024).toFixed(1)} KB)`);
console.log('\nCDN usage:');
console.log('  <script src="https://unpkg.com/@beamterm/renderer/dist/cdn/beamterm.min.js"></script>');
console.log('  <script>');
console.log('    // Option 1: Auto-initialization');
console.log('    const renderer = await Beamterm.createRenderer("#terminal");');
console.log('    ');
console.log('    // Option 2: Manual initialization with custom WASM path');
console.log('    await Beamterm.init("/custom/path/to/beamterm_bg.wasm");');
console.log('    const renderer = await Beamterm.createRenderer("#terminal");');
console.log('  </script>');