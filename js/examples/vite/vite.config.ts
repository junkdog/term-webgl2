// js/examples/vite/vite.config.ts
import { defineConfig } from 'vite';
import wasm from 'vite-plugin-wasm';

export default defineConfig({
    base: './',
    plugins: [wasm()],
    build: {
        target: 'esnext',
    },
    optimizeDeps: {
        // This helps Vite handle the WASM module correctly
        exclude: ['@beamterm/renderer']
    }
});