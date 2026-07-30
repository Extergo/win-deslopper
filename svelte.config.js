import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),
  kit: {
    adapter: adapter({
      pages: 'frontend/build',
      assets: 'frontend/build',
      fallback: 'index.html',
      strict: true
    }),
    files: {
      assets: 'frontend/static',
      lib: 'frontend/src/lib',
      routes: 'frontend/src/routes',
      appTemplate: 'frontend/src/app.html'
    },
    outDir: 'frontend/.svelte-kit'
  }
};

export default config;
