import { describe, it, expect, vi, beforeEach } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

vi.mock("@/integrations/api/client", () => ({
  api: {
    get: vi.fn(),
    post: vi.fn().mockResolvedValue({}),
  },
}));

import {
  createMaterialVersion,
  getCurrentMaterialVersion,
  getMaterialVersions,
  rollbackMaterialVersion,
  rollbackMaterialVersionAction,
} from "./materialVersions";
import { api } from "@/integrations/api/client";

const root = resolve(__dirname, "..");

describe("backend-owned material versions", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("delegates version creation and current-version selection to backend", async () => {
    await createMaterialVersion({
      materialId: "mat-001",
      filePath: "/uploads/doc.pdf",
      fileName: "doc.pdf",
      fileSizeBytes: 1_572_864,
      description: "更新了图表",
      changeNotes: "替换了第三页的数据图",
    });

    expect(api.post).toHaveBeenCalledWith(
      "/api/v1/application_materials/mat-001/versions",
      {
        file_path: "/uploads/doc.pdf",
        file_name: "doc.pdf",
        file_size_bytes: 1_572_864,
        description: "更新了图表",
        change_notes: "替换了第三页的数据图",
      },
    );
    expect(api.get).not.toHaveBeenCalled();
  });

  it("passes null optional fields without calculating defaults locally", async () => {
    await createMaterialVersion({
      materialId: "mat-001",
      filePath: "/uploads/doc.pdf",
      fileName: "doc.pdf",
      fileSizeBytes: 1024,
    });

    expect(api.post).toHaveBeenCalledWith(
      "/api/v1/application_materials/mat-001/versions",
      expect.objectContaining({
        description: null,
        change_notes: null,
      }),
    );
  });

  it("reads backend-owned current version and version list", async () => {
    (api.get as ReturnType<typeof vi.fn>).mockResolvedValue({
      versions: [
        { id: "v2", material_id: "mat-001", version_number: 2 },
        { id: "v1", material_id: "mat-001", version_number: 1 },
      ],
      current_version: { id: "v2", material_id: "mat-001", version_number: 2 },
      version_count_label: "共 2 个版本",
      earliest_version_label: "最早版本：2026-05-01",
    });

    const versions = await getMaterialVersions("mat-001");
    const current = await getCurrentMaterialVersion("mat-001");

    expect(versions).toHaveLength(2);
    expect(current).toEqual(expect.objectContaining({ id: "v2" }));
    expect(api.get).toHaveBeenCalledWith(
      "/api/v1/application_materials/mat-001/versions",
    );
  });

  it("delegates rollback to backend workflow", async () => {
    await rollbackMaterialVersion("mat-001", "v1");

    expect(api.post).toHaveBeenCalledWith(
      "/api/v1/application_materials/mat-001/versions/v1/rollback",
      {},
    );
  });

  it("executes backend-provided rollback actions directly", async () => {
    await rollbackMaterialVersionAction({
      method: "POST",
      endpoint: "/api/v1/application_materials/mat-001/versions/v1/rollback",
      confirm_message: "确认回滚",
      success_title: "回滚成功",
      success_message: "已回滚到版本 1",
    });

    expect(api.post).toHaveBeenCalledWith(
      "/api/v1/application_materials/mat-001/versions/v1/rollback",
      {},
    );
  });

  it("keeps material version UI free of local business calculations", () => {
    const libSource = readFileSync(resolve(root, "lib/materialVersions.ts"), "utf8");
    const uploadSource = readFileSync(
      resolve(root, "components/applications/MaterialUploadDialog.tsx"),
      "utf8",
    );
    const historySource = readFileSync(
      resolve(root, "components/applications/MaterialVersionHistory.tsx"),
      "utf8",
    );

    expect(libSource).toContain("/api/v1/application_materials/${params.materialId}/versions");
    expect(libSource).not.toContain("/api/material_versions");
    expect(libSource).not.toContain(".filter(");
    expect(libSource).not.toContain(".reduce(");
    expect(libSource).not.toContain("newVersionNumber");
    expect(libSource).not.toContain("is_current: 1");

    expect(uploadSource).toContain("fileSizeBytes");
    expect(uploadSource).toContain("filePath: uploadResult.id");
    expect(uploadSource).not.toContain("fileSizeKB");
    expect(uploadSource).not.toContain("toFixed");
    expect(uploadSource).not.toContain("new Date().toISOString");
    expect(uploadSource).not.toContain("uploadResult?.filename");
    expect(uploadSource).not.toContain("api.put(`/api/application_materials");

    expect(historySource).toContain("getMaterialVersionHistory");
    expect(historySource).toContain("rollbackMaterialVersionAction");
    expect(historySource).toContain("version.rollback_action");
    expect(historySource).toContain("files.download(version.file_path)");
    expect(historySource).toContain("currentVersion");
    expect(historySource).not.toContain("TODO: implement file download");
    expect(historySource).not.toContain("const data = null");
    expect(historySource).not.toContain("formatDistanceToNow");
    expect(historySource).not.toContain("new Date(version.created_at)");
    expect(historySource).not.toContain("versions.find");
    expect(historySource).not.toContain("index === 0");
    expect(historySource).not.toContain("api.put");
    expect(historySource).not.toContain("confirm(`确定要回滚到版本");
  });
});
