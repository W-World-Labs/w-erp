// Mark the dual build outputs so Node resolves each correctly regardless of the
// root package.json `type` (which is left as the CommonJS default).
import { writeFileSync, mkdirSync } from 'node:fs';

mkdirSync('dist/esm', { recursive: true });
mkdirSync('dist/cjs', { recursive: true });
writeFileSync('dist/esm/package.json', JSON.stringify({ type: 'module' }, null, 2) + '\n');
writeFileSync('dist/cjs/package.json', JSON.stringify({ type: 'commonjs' }, null, 2) + '\n');
