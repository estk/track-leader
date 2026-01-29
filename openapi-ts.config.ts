import { defineConfig } from '@hey-api/openapi-ts';

export default defineConfig({
  client: '@hey-api/client-fetch',
  input: './openapi.json',
  output: './src/lib/api-generated',
  plugins: [
    '@hey-api/typescript',
    '@hey-api/sdk',
  ],
});
