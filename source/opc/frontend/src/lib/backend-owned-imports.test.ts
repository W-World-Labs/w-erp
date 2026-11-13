import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

const root = resolve(__dirname, '..');

describe('backend-owned import UI', () => {
  it('does not parse spreadsheet rows or write business rows from ImportDataDialog', () => {
    const source = readFileSync(resolve(root, 'components/dashboard/ImportDataDialog.tsx'), 'utf8');
    const helper = readFileSync(resolve(root, 'lib/backendImport.ts'), 'utf8');

    expect(helper).toContain('/api/v1/imports/options');
    expect(source).toContain('IMPORT_OPTIONS_ENDPOINT');
    expect(source).toContain('api.runAction<BackendImportResult>');
    expect(source).toContain('selectedOption.import_action');
    expect(source).not.toContain('xlsx');
    expect(source).not.toContain('sheet_to_json');
    expect(source).not.toContain('parseFile');
    expect(source).not.toContain('bulkImportEndpoint');
    expect(source).not.toContain('IMPORT_TABLE_OPTIONS');
    expect(source).not.toContain('api.post<BackendImportResult>');
    expect(source).not.toContain('api.post(`/api/${selectedTable}`');
  });

  it('does not parse spreadsheet rows or call fake AI import analysis from MagicImport', () => {
    const source = readFileSync(resolve(root, 'components/dashboard/MagicImport.tsx'), 'utf8');
    const helper = readFileSync(resolve(root, 'lib/backendImport.ts'), 'utf8');

    expect(helper).toContain('/api/v1/imports/options');
    expect(source).toContain('IMPORT_OPTIONS_ENDPOINT');
    expect(source).toContain('api.runAction<BackendImportResult>');
    expect(source).toContain('selectedOption.import_action');
    expect(source).not.toContain('xlsx');
    expect(source).not.toContain('sheet_to_json');
    expect(source).not.toContain('AI_ANALYZE_URL');
    expect(source).not.toContain('/api/ai/import-analyze');
    expect(source).not.toContain('bulkImportEndpoint');
    expect(source).not.toContain('IMPORT_TABLE_OPTIONS');
    expect(source).not.toContain('api.post<BackendImportResult>');
    expect(source).not.toContain('api.post(`/api/${schema.table}`');
  });
});
