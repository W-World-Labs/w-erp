import { api } from "@/integrations/api/client";

export interface MaterialVersionBadge {
  label: string;
  variant?: "default" | "secondary" | "destructive" | "outline";
  icon?: string;
  class_name?: string;
}

export interface MaterialVersionAction {
  method: "POST";
  endpoint: string;
  confirm_message: string;
  success_title?: string;
  success_message: string;
}

export interface MaterialVersion {
  id: string;
  material_id: string;
  version_number: number;
  file_path: string;
  file_name: string;
  file_size: string;
  file_size_label: string;
  description: string | null;
  change_notes: string | null;
  created_by: string;
  created_at: string;
  created_at_label: string;
  is_current: number | boolean;
  version_badge: MaterialVersionBadge;
  current_badge: MaterialVersionBadge | null;
  latest_badge: MaterialVersionBadge | null;
  card_class_name: string;
  rollback_enabled: boolean;
  rollback_action: MaterialVersionAction | null;
}

export interface MaterialVersionHistoryResponse {
  material_id: string;
  versions: MaterialVersion[];
  current_version: MaterialVersion | null;
  version_count: number;
  version_count_label: string;
  earliest_version_label: string;
}

export interface MaterialVersionWorkflowResponse {
  version: MaterialVersion;
  material: {
    id: string;
    file_path?: string | null;
    file_size?: string | null;
    file_size_label?: string;
    upload_date_label?: string;
    uploaded_summary?: string;
    status?: string;
  };
}

interface CreateVersionParams {
  materialId: string;
  filePath: string;
  fileName: string;
  fileSizeBytes: number;
  description?: string;
  changeNotes?: string;
}

export async function createMaterialVersion(
  params: CreateVersionParams,
): Promise<MaterialVersionWorkflowResponse> {
  return api.post<MaterialVersionWorkflowResponse>(
    `/api/v1/application_materials/${params.materialId}/versions`,
    {
      file_path: params.filePath,
      file_name: params.fileName,
      file_size_bytes: params.fileSizeBytes,
      description: params.description || null,
      change_notes: params.changeNotes || null,
    },
  );
}

export async function getMaterialVersionHistory(
  materialId: string,
): Promise<MaterialVersionHistoryResponse> {
  return api.get<MaterialVersionHistoryResponse>(
    `/api/v1/application_materials/${materialId}/versions`,
  );
}

export async function getCurrentMaterialVersion(materialId: string) {
  const history = await getMaterialVersionHistory(materialId);
  return history.current_version;
}

export async function getMaterialVersions(materialId: string) {
  const history = await getMaterialVersionHistory(materialId);
  return history.versions;
}

export async function rollbackMaterialVersion(
  materialId: string,
  versionId: string,
): Promise<MaterialVersionWorkflowResponse> {
  return api.post<MaterialVersionWorkflowResponse>(
    `/api/v1/application_materials/${materialId}/versions/${versionId}/rollback`,
    {},
  );
}

export async function rollbackMaterialVersionAction(
  action: MaterialVersionAction,
): Promise<MaterialVersionWorkflowResponse> {
  return api.post<MaterialVersionWorkflowResponse>(action.endpoint, {});
}
