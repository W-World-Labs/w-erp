import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

const frontendRoot = resolve(__dirname, '../..');
describe('W-Radar configurable bootstrap shell', () => {
  it('keeps production telemetry identities out of the HTML shell', () => {
    const shell = readFileSync(resolve(frontendRoot, 'index.html'), 'utf8');
    expect(shell).not.toContain('data-code=');
    expect(shell).not.toContain('platform/bootstrap.js');
  });

  it('requires explicit tracking and bootstrap configuration', () => {
    const telemetry = readFileSync(resolve(frontendRoot, 'src/lib/telemetry.ts'), 'utf8');
    expect(telemetry).toContain('VITE_W_RADAR_CODE');
    expect(telemetry).toContain('VITE_W_RADAR_BOOTSTRAP_URL');
    expect(telemetry).toContain('RADAR_TRACKING_CODE && bootstrapUrl && workspaceId && projectId');
  });
});
