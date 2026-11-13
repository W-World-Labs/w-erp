import type { BackendAction } from "@/integrations/api/client";

export const IMPORT_OPTIONS_ENDPOINT = "/api/v1/imports/options";

export interface BackendImportOption {
  value: string;
  label: string;
  import_action: BackendAction | null;
}

export interface BackendImportOptions {
  table_options: BackendImportOption[];
}

export interface BackendImportResult {
  table: string;
  total: number;
  success: number;
  failed: number;
  errors: Array<{
    row: number;
    error: string;
  }>;
}

export function isSupportedBackendImportFile(file: File) {
  return file.name.toLowerCase().endsWith('.csv');
}

export function fileToBase64(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(String(reader.result || ''));
    reader.onerror = () => reject(new Error("文件读取失败"));
    reader.readAsDataURL(file);
  });
}
