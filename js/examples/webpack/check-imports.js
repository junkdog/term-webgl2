#!/usr/bin/env node
// Quick script to verify the WASM module exports are available

const fs = require('fs');
const path = require('path');

console.log('🔍 Checking Beamterm WASM exports...\n');

// Check if the bundler build exists
const bundlerPath = path.join(__dirname, '../../dist/bundler/');
const jsFile = path.join(bundlerPath, 'beamterm_renderer.js');
const dtsFile = path.join(bundlerPath, 'beamterm_renderer.d.ts');
const wasmFile = path.join(bundlerPath, 'beamterm_renderer_bg.wasm');

// Check files exist
const files = [
    { path: jsFile, desc: 'Main JS file' },
    { path: dtsFile, desc: 'TypeScript definitions' },
    { path: wasmFile, desc: 'WASM binary' }
];

let allGood = true;

files.forEach(({ path: filePath, desc }) => {
    if (fs.existsSync(filePath)) {
        const stats = fs.statSync(filePath);
        console.log(`✅ ${desc}: ${(stats.size / 1024).toFixed(1)} KB`);
    } else {
        console.log(`❌ ${desc}: NOT FOUND`);
        allGood = false;
    }
});

console.log();

// Check TypeScript definitions content
if (fs.existsSync(dtsFile)) {
    const content = fs.readFileSync(dtsFile, 'utf-8');
    const exports = content.match(/export\s+(class|function|interface|const)\s+(\w+)/g) || [];

    if (exports.length > 0) {
        console.log(`📦 Found ${exports.length} exports:`);

        // Check for expected exports
        const expectedExports = ['BeamtermRenderer', 'CellStyle', 'Batch', 'Size'];
        expectedExports.forEach(name => {
            if (content.includes(`export class ${name}`) ||
                content.includes(`export interface ${name}`)) {
                console.log(`   ✅ ${name}`);
            } else {
                console.log(`   ❌ ${name} - NOT FOUND`);
                allGood = false;
            }
        });
    } else {
        console.log('❌ No exports found in TypeScript definitions!');
        console.log('   This usually means the js-api feature was not enabled.');
        allGood = false;
    }
} else {
    console.log('❌ Cannot check exports - TypeScript definitions not found');
}

console.log();

// Summary
if (allGood) {
    console.log('✅ Everything looks good! You can run: npm start');
} else {
    console.log('❌ Issues found. Please run from project root:');
    console.log('   ./build.zsh build-wasm');
    process.exit(1);
}

// Try to import and check
console.log('\n🧪 Attempting to load the module...');
try {
    const beamtermPath = path.join(bundlerPath, 'beamterm_renderer.js');
    const module = require(beamtermPath);

    console.log('✅ Module loaded successfully');
    console.log('   Available exports:', Object.keys(module).join(', '));

    if (!module.BeamtermRenderer) {
        console.log('   ⚠️  Warning: BeamtermRenderer class not found in exports');
    }
} catch (error) {
    console.log('❌ Failed to load module:', error.message);
    console.log('   Note: This is expected in Node.js environment');
    console.log('   The module should work fine in the browser');
}