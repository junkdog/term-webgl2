import { defineConfig } from 'vite';
import wasm from 'vite-plugin-wasm';

export default defineConfig({
    base: './',
    plugins: [wasm()],
    build: {
        target: 'esnext',
    },
    optimizeDeps: {
        exclude: ['@beamterm/renderer']
    },
    server: {
        port: 3001,
        open: true
    }
}); 