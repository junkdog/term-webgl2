#!/usr/bin/env node

const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

const EXAMPLES_DIR = path.join(__dirname, '..', 'examples');
const DIST_DIR = path.join(__dirname, '..', 'examples-dist');

console.log('📦 Building beamterm examples for GitHub Pages...\n');

// Ensure clean dist directory
if (fs.existsSync(DIST_DIR)) {
    fs.rmSync(DIST_DIR, { recursive: true });
}
fs.mkdirSync(DIST_DIR, { recursive: true });

function buildExample(exampleName) {
    const exampleDir = path.join(EXAMPLES_DIR, exampleName);
    const exampleDistDir = path.join(DIST_DIR, exampleName);

    if (!fs.existsSync(exampleDir)) {
        console.log(`⚠️  Example ${exampleName} not found, skipping...`);
        return false;
    }

    console.log(`🔨 Building ${exampleName} example...`);

    try {
        // Install dependencies
        console.log(`   Installing dependencies...`);
        execSync('npm install', {
            cwd: exampleDir,
            stdio: 'pipe'
        });

        // Build the example
        console.log(`   Building production bundle...`);
        execSync('npm run build', {
            cwd: exampleDir,
            stdio: 'pipe'
        });

        // Copy build output to dist
        const buildDir = path.join(exampleDir, 'dist');
        if (fs.existsSync(buildDir)) {
            fs.cpSync(buildDir, exampleDistDir, { recursive: true });
            console.log(`✅ ${exampleName} built successfully`);
            return true;
        } else {
            console.log(`❌ ${exampleName} build directory not found`);
            return false;
        }

    } catch (error) {
        console.error(`❌ Failed to build ${exampleName}:`);
        console.error(error.message);

        // Log more details if available
        if (error.stdout) {
            console.error('STDOUT:', error.stdout.toString());
        }
        if (error.stderr) {
            console.error('STDERR:', error.stderr.toString());
        }

        return false;
    }
}

// Build all examples
const examples = ['webpack', 'vite'];
const results = [];

for (const example of examples) {
    const success = buildExample(example);
    results.push({ example, success });
    console.log(''); // Empty line for readability
}

// Summary
// Copy landing page
console.log('📝 Creating landing page...');
const landingPageSrc = path.join(EXAMPLES_DIR, 'landing-page.html');
const landingPageDest = path.join(DIST_DIR, 'index.html');

if (fs.existsSync(landingPageSrc)) {
    fs.copyFileSync(landingPageSrc, landingPageDest);
    console.log('✅ Landing page created');
} else {
    console.log('⚠️  Landing page template not found, creating basic one...');

    // Fallback basic landing page
    const basicLandingPage = `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>beamterm Examples</title>
    <style>
        body { font-family: system-ui; max-width: 800px; margin: 50px auto; padding: 20px; }
        .examples { display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 20px; }
        .example { border: 1px solid #ddd; padding: 20px; border-radius: 8px; }
        .example a { text-decoration: none; color: #0066cc; }
    </style>
</head>
<body>
    <h1>🚀 beamterm Examples</h1>
    <p>High-performance WebGL2 terminal renderer demos</p>
    <div class="examples">
        <div class="example">
            <h3><a href="./webpack/">Webpack Example</a></h3>
            <p>Classic bundler setup with JavaScript</p>
        </div>
        <div class="example">
            <h3><a href="./vite/">Vite + TypeScript</a></h3>
            <p>Modern development with TypeScript</p>
        </div>
    </div>
</body>
</html>`;

    fs.writeFileSync(landingPageDest, basicLandingPage);
    console.log('✅ Basic landing page created');
}

console.log('');

console.log('📊 Build Summary:');
console.log('================');

let allSuccessful = true;
for (const { example, success } of results) {
    const status = success ? '✅ Success' : '❌ Failed';
    console.log(`  ${example.padEnd(12)} ${status}`);
    if (!success) allSuccessful = false;
}

console.log('');

if (allSuccessful) {
    console.log('🎉 All examples built successfully!');
    console.log(`📁 Output directory: ${DIST_DIR}`);

    // List what was built
    console.log('\n📋 Built examples:');
    for (const { example, success } of results) {
        if (success) {
            const examplePath = path.join(DIST_DIR, example);
            const files = fs.readdirSync(examplePath);
            console.log(`  ${example}/`);
            files.slice(0, 5).forEach(file => {
                console.log(`    ${file}`);
            });
            if (files.length > 5) {
                console.log(`    ... and ${files.length - 5} more files`);
            }
        }
    }
} else {
    console.log('💥 Some examples failed to build. Check the errors above.');
    process.exit(1);
}

// Validate that WASM files exist
console.log('\n🔍 Validating WASM files...');
const wasmChecks = [
    'webpack/bundle.js',
    'vite/index.html',
];

let wasmValid = true;
for (const check of wasmChecks) {
    const filePath = path.join(DIST_DIR, check);
    if (fs.existsSync(filePath)) {
        console.log(`  ✅ ${check}`);
    } else {
        console.log(`  ❌ ${check} - Missing!`);
        wasmValid = false;
    }
}

if (!wasmValid) {
    console.log('\n⚠️  Some expected files are missing. The examples may not work correctly.');
    console.log('   Make sure WASM packages are built before running this script.');
}

console.log('\n🚀 Examples ready for deployment!');