import { defineConfig } from '@rsbuild/core';
import { pluginVue } from '@rsbuild/plugin-vue';
import dotenv from 'dotenv';

// load .env (optional; .env.example documents the defaults)
dotenv.config();

const apiBaseUrl =
  process.env.NODE_ENV === 'production'
    ? process.env.API_BASE_URL_PROD || '/api'
    : process.env.API_BASE_URL_DEV || 'http://127.0.0.1:8080';

export default defineConfig({
  plugins: [pluginVue()],
  html: {
    template: './index.html',
  },
  source: {
    define: {
      'process.env.API_BASE_URL': JSON.stringify(apiBaseUrl),
    },
  },
});
