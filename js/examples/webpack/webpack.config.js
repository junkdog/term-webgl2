const path = require('path');
const HtmlWebpackPlugin = require('html-webpack-plugin');

module.exports = {
    entry: './src/index.js',
    output: {
        path: path.resolve(__dirname, 'dist'),
        filename: 'bundle.js',
        clean: true,
    },
    experiments: {
        // Enable WebAssembly support
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
    plugins: [
        new HtmlWebpackPlugin({
            template: './src/index.html',
            inject: 'body',
        }),
    ],
    devServer: {
        static: {
            directory: path.join(__dirname, 'dist'),
        },
        compress: true,
        port: 8080,
        hot: true,
        open: true,
        // Enable CORS for WASM
        headers: {
            'Cross-Origin-Embedder-Policy': 'require-corp',
            'Cross-Origin-Opener-Policy': 'same-origin',
        },
    },
    // Resolve .wasm files from the @beamterm/renderer package
    resolve: {
        extensions: ['.js', '.wasm'],
        alias: {
            '@beamterm/renderer': path.resolve(__dirname, '../../dist/bundler/beamterm_renderer.js'),
        },
    },
    // Development mode for easier debugging
    mode: 'development',
    devtool: 'source-map',
};