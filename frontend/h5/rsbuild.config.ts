import { defineConfig } from '@rsbuild/core';
import { pluginVue } from '@rsbuild/plugin-vue';
import dotenv from 'dotenv';

// load .env
dotenv.config();

export default defineConfig({
  plugins: [pluginVue()],
  source: {
    define: {
      'process.env.API_BASE_URL': JSON.stringify(process.env.API_BASE_URL)
    }
  }
});
