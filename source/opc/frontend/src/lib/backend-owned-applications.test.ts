import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(__dirname, "..");

describe("backend-owned applications UI", () => {
  it("delegates application workflow calculations to backend endpoints", () => {
    const source = readFileSync(
      resolve(root, "pages/Applications.tsx"),
      "utf8",
    );
    const contracts = readFileSync(
      resolve(root, "lib/api/contracts.ts"),
      "utf8",
    );
    const materialVersions = readFileSync(
      resolve(root, "components/applications/MaterialVersionHistory.tsx"),
      "utf8",
    );

    expect(source).toContain("API_ENDPOINTS.applicationsOverview");
    expect(source).toContain("API_ENDPOINTS.applicationMaterialsOverview");
    expect(source).toContain("overview?.create_from_template_action");
    expect(source).toContain("overview?.material_library_create_action");
    expect(source).toContain("materialsOverview?.reuse_action");
    expect(source).toContain("api.runAction(action");
    expect(source).toContain("recommended_templates");
    expect(source).toContain("template.visual_icon");
    expect(source).toContain("template.visual_color");
    expect(source).toContain("template.card_class_name");
    expect(source).toContain("template.recommended_badge");
    expect(source).toContain("recommendationContext.registered_capital_label");
    expect(source).toContain("recommendationContext.years_in_business_label");
    expect(source).toContain("recommendationContext.recommended_count_label");
    expect(source).toContain("app.status_badge");
    expect(source).toContain("app.deadline_label");
    expect(source).toContain("app.deadline_badge");
    expect(source).toContain("app.progress_label");
    expect(source).toContain("app.row_class_name");
    expect(source).toContain("app.delete_action");
    expect(source).toContain("item.upload_date_label");
    expect(source).toContain("item.used_count_label");
    expect(source).toContain("item.delete_action");
    expect(source).toContain("file_path: uploaded.id");
    expect(source).toContain("files.download(item.file_path)");
    expect(materialVersions).toContain("version.version_badge");
    expect(materialVersions).toContain("version.current_badge.variant");
    expect(materialVersions).toContain("version.current_badge.icon");
    expect(source).toContain("material.status_badge");
    expect(source).toContain("material.required_badge");
    expect(source).toContain("material.uploaded_summary");
    expect(source).toContain("materialsOverview?.required_materials");
    expect(source).toContain("materialsOverview?.pending_materials");
    expect(source).toContain("overview?.material_library_items");
    expect(source).toContain("overview?.type_options ?? []");
    expect(contracts).toContain("applicationsOverview");
    expect(contracts).toContain("applicationsFromTemplate");
    expect(contracts).toContain("applicationMaterialsReuse");
    expect(contracts).toContain("applicationMaterialsOverview");

    expect(source).not.toContain("applications.length + 1");
    expect(source).not.toContain("APP${");
    expect(source).not.toContain("materialTemplates.filter");
    expect(source).not.toContain("API_ENDPOINTS.companyInfo");
    expect(source).not.toContain("getRecommendedTemplates");
    expect(source).not.toContain("differenceInDays");
    expect(source).not.toContain("registeredCapital");
    expect(source).not.toContain("yearsInBusiness");
    expect(source).not.toContain('status === "preparing"');
    expect(source).not.toContain('status === "approved"');
    expect(source).not.toContain("getTemplateVisual");
    expect(source).not.toContain("templates.filter");
    expect(source).not.toContain("template.recommended_badge ?");
    expect(source).not.toContain("{template.recommended_badge &&");
    expect(source).not.toContain("border-primary/30");
    expect(source).not.toContain("确定要删除此申报项目吗");
    expect(source).not.toContain("确定要删除这个材料吗");
    expect(source).not.toContain("申报项目已删除");
    expect(source).not.toContain("材料已从材料库删除");
    expect(source).not.toContain("deleteApplicationMutation.mutate(app.id)");
    expect(source).not.toContain("deleteLibraryItemMutation.mutate(item)");
    expect(source).not.toContain("api.delete(`${API_ENDPOINTS.applications}/${id}`)");
    expect(source).not.toContain("api.delete(`${API_ENDPOINTS.materialLibrary}/${item.id}`)");
    expect(source).not.toContain("deadline_status ===");
    expect(source).not.toContain("daysLeft");
    expect(source).not.toContain("getStatusBadge");
    expect(source).not.toContain("getMaterialStatusBadge");
    expect(source).not.toContain("file_path: uploaded.url");
    expect(source).not.toContain("TODO: implement file download");
    expect(materialVersions).not.toContain("version.current_badge ? \"default\" : \"secondary\"");
    expect(materialVersions).not.toContain('<CheckCircle className="h-3 w-3" />');
    expect(source).not.toContain("const data = null");
    expect(source).not.toContain("new Date(item.upload_date)");
    expect(source).not.toContain("new Date(material.upload_date)");
    expect(source).not.toContain(".toLocaleDateString");
    expect(source).not.toContain(".toLocaleString");
    expect(source).not.toContain("Number(recommendationContext.registered_capital)");
    expect(source).not.toContain("materials.filter((m) => m.required)");
    expect(source).not.toContain('materials.filter((m) => m.status === "pending")');
    expect(source).not.toContain("completedMaterials = materials.filter");
    expect(source).not.toContain('[{ value: "all", label: "全部类型" }]');
    expect(source).not.toContain('queryKey: ["material_library"]');
    expect(source).not.toContain("api.get<{ data: any[]; count: number }>");
    expect(source).not.toContain(
      "api.post(API_ENDPOINTS.applicationMaterials,",
    );
    expect(source).not.toContain("api.post(API_ENDPOINTS.applicationsFromTemplate");
    expect(source).not.toContain("api.post(API_ENDPOINTS.materialLibrary");
    expect(source).not.toContain("api.post(API_ENDPOINTS.applicationMaterialsReuse");
  });
});
