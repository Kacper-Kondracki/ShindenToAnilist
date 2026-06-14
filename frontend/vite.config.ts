import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import tailwindcss from '@tailwindcss/vite';

export default defineConfig({
  base: './',
  plugins: [
    tailwindcss(),
    svelte({
      inspector: true
    })
  ],
  server: {
    host: '127.0.0.1',
    port: 4173,
    strictPort: true
  }
});
