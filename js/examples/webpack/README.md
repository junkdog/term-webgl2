# js/examples/webpack/README.md

# Beamterm Webpack Example

This example demonstrates how to use Beamterm with Webpack 5.

## Prerequisites

Build the WASM packages first:
```bash
# From project root
./build.zsh build-wasm
```

## Setup

```bash
npm install
npm start
```

Then open http://localhost:8080 in your browser.

## Key Configuration

Webpack 5 has built-in support for WASM with the `asyncWebAssembly` experiment:

```javascript
module.exports = {
    experiments: {
        asyncWebAssembly: true,
    },
    module: {
        rules: [
            {
                test: /\.wasm$/,
                type: 'webassembly/async',
            },
        ],
    },
};
```

## Project Structure

```
webpack/
├── package.json
├── webpack.config.js
├── README.md
├── src/
│   ├── index.html
│   └── index.js
└── dist/            # Generated on build
```

## Building for Production

```bash
npm run build
```

This creates an optimized bundle in the `dist/` directory.

