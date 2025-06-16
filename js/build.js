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

// Create a CDN entry point
const cdnEntry = `
import init, * as BeamtermModule from './beamterm_renderer.js';

// Initialization state
let initPromise = null;
let initialized = false;

// Helper to ensure initialization
async function ensureInit(wasmUrl) {
    if (!initialized) {
        if (!initPromise) {
            if (wasmUrl) {
                initPromise = init(wasmUrl);
            } else {
                // Auto-detect WASM URL
                let baseUrl = '';
                try {
                    if (typeof document !== 'undefined' && document.currentScript) {
                        baseUrl = document.currentScript.src.substring(0, document.currentScript.src.lastIndexOf('/'));
                    } else if (typeof location !== 'undefined') {
                        baseUrl = location.href.substring(0, location.href.lastIndexOf('/'));
                    }
                } catch (e) {
                    // Fallback to relative path
                }
                const autoWasmUrl = baseUrl ? baseUrl + '/beamterm_bg.wasm' : './beamterm_bg.wasm';
                initPromise = init(autoWasmUrl);
            }
        }
        await initPromise;
        initialized = true;
    }
}

// Create the Beamterm API object
const Beamterm = {
    async init(wasmUrl) {
        await ensureInit(wasmUrl);
        return this;
    },
    
    style() {
        if (!initialized) {
            throw new Error('Beamterm not initialized. Call await Beamterm.init() first.');
        }
        return BeamtermModule.style();
    },
    
    cell(symbol, style) {
        if (!initialized) {
            throw new Error('Beamterm not initialized. Call await Beamterm.init() first.');
        }
        return BeamtermModule.cell(symbol, style);
    },
    
    BeamtermRenderer: function(canvasId) {
        if (!initialized) {
            throw new Error('Beamterm not initialized. Call await Beamterm.init() first.');
        }
        return new BeamtermModule.BeamtermRenderer(canvasId);
    },
    
    CellStyle: function() {
        if (!initialized) {
            throw new Error('Beamterm not initialized. Call await Beamterm.init() first.');
        }
        return new BeamtermModule.CellStyle();
    },
    
    Cell: function(symbol, style) {
        if (!initialized) {
            throw new Error('Beamterm not initialized. Call await Beamterm.init() first.');
        }
        return new BeamtermModule.Cell(symbol, style);
    },
    
    get Batch() { 
        return initialized ? BeamtermModule.Batch : undefined; 
    },
    
    get Size() { 
        return initialized ? BeamtermModule.Size : undefined; 
    },
    
    version: '0.2.0',
    
    async createRenderer(canvasId) {
        await this.init();
        return new this.BeamtermRenderer(canvasId);
    },
    
    get initialized() { return initialized; },
    get module() { return initialized ? BeamtermModule : null; }
};

// Auto-initialize in browser
if (typeof window !== 'undefined') {
    // Start auto-init but don't block
    ensureInit().catch(err => {
        console.warn('Beamterm auto-initialization failed:', err);
        console.warn('Call await Beamterm.init() with correct WASM path if needed');
    });
}

export default Beamterm;
`;

fs.writeFileSync(path.join(tempDir, 'cdn-entry.js'), cdnEntry);

// Bundle with esbuild
console.log('📦 Bundling with esbuild...');
execSync(`npx esbuild ${tempDir}/cdn-entry.js --bundle --format=iife --global-name=Beamterm --outfile=${cdnDir}/beamterm.min.js --minify --sourcemap --platform=browser`, {
    stdio: 'inherit'
});

// Add the simple fix to unwrap the default export
const unwrapCode = `
// Fix esbuild's default export wrapper
if (typeof window !== 'undefined' && window.Beamterm && window.Beamterm.default) {
    window.Beamterm = window.Beamterm.default;
}
`;

// Append the unwrap code to the bundle
fs.appendFileSync(path.join(cdnDir, 'beamterm.min.js'), unwrapCode);

// Copy WASM file to CDN directory with consistent name
fs.copyFileSync(
    path.join(tempDir, 'beamterm_renderer_bg.wasm'),
    path.join(cdnDir, 'beamterm_bg.wasm')
);

// Clean up temp directory
fs.rmSync(tempDir, { recursive: true, force: true });

console.log('✅ CDN bundle built');
console.log('✅ Added default export unwrapper');

console.log('\n🎉 All packages built successfully!');
console.log('\nCDN files:');
console.log(`  ${cdnDir}/beamterm.min.js (${(fs.statSync(path.join(cdnDir, 'beamterm.min.js')).size / 1024).toFixed(1)} KB)`);
console.log(`  ${cdnDir}/beamterm_bg.wasm (${(fs.statSync(path.join(cdnDir, 'beamterm_bg.wasm')).size / 1024).toFixed(1)} KB)`);
console.log('\nCDN usage:');
console.log('  <script src="https://unpkg.com/@beamterm/renderer/dist/cdn/beamterm.min.js"></script>');
console.log('  <script>');
console.log('    await Beamterm.init();');
console.log('    const renderer = new Beamterm.BeamtermRenderer("#terminal");');
console.log('  </script>');