// Test script for Node.js environment
// Note: Due to wasm-bindgen limitations in Node.js, we only test module loading
const { JSDOM } = require('jsdom');

// Setup minimal DOM for testing
const dom = new JSDOM(`<!DOCTYPE html><body><canvas id="test-canvas"></canvas></body>`);
global.window = dom.window;
global.document = dom.window.document;
global.WebGL2RenderingContext = class WebGL2RenderingContext {}; // Mock

async function runTests() {
    console.log('🧪 Testing beamterm WASM module in Node.js...\n');

    let moduleLoaded = false;

    // Try different build targets to find one that works
    const targets = [
        { name: 'nodejs', path: '../dist/nodejs/beamterm_renderer.js' },
        { name: 'bundler', path: '../dist/bundler/beamterm_renderer.js' }
    ];

    for (const target of targets) {
        try {
            console.log(`Trying ${target.name} build...`);
            const module = require(target.path);

            // Check if basic exports exist
            if ('CellStyle' in module && 'BeamtermRenderer' in module) {
                console.log(`✅ ${target.name} module loaded successfully`);
                console.log('   Exports found: CellStyle, BeamtermRenderer');
                moduleLoaded = true;
                break;
            }
        } catch (error) {
            console.log(`   ⚠️  ${target.name} failed: ${error.message}`);
        }
    }

    if (moduleLoaded) {
        console.log('\n✅ Module loading test passed!');
        console.log('ℹ️  Full API testing requires a browser environment (use Playwright tests)');
        process.exit(0);
    } else {
        console.error('\n❌ Failed to load any WASM module variant');
        console.error('Make sure to run: ./build.zsh build-wasm');
        process.exit(1);
    }
}

runTests();
