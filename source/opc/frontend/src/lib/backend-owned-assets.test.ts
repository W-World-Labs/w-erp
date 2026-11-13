import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(__dirname, "..");

describe("backend-owned asset management UI", () => {
  it("loads asset summary and derived row values from backend endpoints", () => {
    const source = readFileSync(resolve(root, "pages/Assets.tsx"), "utf8");

    expect(source).toContain("/api/assets/overview");
    expect(source).toContain("status=");
    expect(source).toContain("category_id=");
    expect(source).toContain("asset_type=");
    expect(source).toContain("asset.status_badge");
    expect(source).toContain("asset.asset_type_badge");
    expect(source).toContain("asset.expiry_badge");
    expect(source).toContain("asset.current_value_label");
    expect(source).toContain("asset.net_value_label");
    expect(source).toContain("asset.monthly_depreciation_label");
    expect(source).toContain("asset.can_delete");
    expect(source).toContain("asset.delete_action");
    expect(source).toContain("category.delete_action");
    expect(source).toContain("assetOverview.status_options");
    expect(source).toContain("assetOverview.asset_type_options");
    expect(source).toContain("assetOverview.category_options");
    expect(source).toContain("assetOverview.empty_message");
    expect(source).toContain("assetOverview.permissions.can_manage_categories");
    expect(source).toContain("asset.can_mark_maintenance");
    expect(source).toContain("asset.can_mark_scrapped");
    expect(source).toContain("asset.can_mark_in_use");
    expect(source).toContain("asset.mark_maintenance_action");
    expect(source).toContain("asset.mark_scrapped_action");
    expect(source).toContain("asset.mark_in_use_action");
    expect(source).toContain("asset.update_action");
    expect(source).toContain("category.update_action");
    expect(source).toContain("assetOverview.create_action");
    expect(source).toContain("assetOverview.category_create_action");
    expect(source).toContain("api.runAction(action)");
    expect(source).not.toContain("/mark_maintenance");
    expect(source).not.toContain("/mark_scrapped");
    expect(source).not.toContain("/mark_in_use");
    expect(source).not.toContain('<SelectItem value="all">全部状态</SelectItem>');
    expect(source).not.toContain('<SelectItem value="all">全部分类</SelectItem>');
    expect(source).not.toContain('<SelectItem value="all">全部类型</SelectItem>');
    expect(source).not.toContain("const stats = {");
    expect(source).not.toContain("assets.reduce((sum, a)");
    expect(source).not.toContain("assets.filter((a) => a.status === 'in_use')");
    expect(source).not.toContain("const statusMap");
    expect(source).not.toContain("const assetTypeMap");
    expect(source).not.toContain("filteredAssets");
    expect(source).not.toContain("filteredCategories");
    expect(source).not.toContain("renderExpiryBadge");
    expect(source).not.toContain("toLocaleString");
    expect(source).not.toContain("thirtyDaysLater");
    expect(source).not.toContain("daysUntilExpiry =");
    expect(source).not.toContain("new Date(expiryDate)");
    expect(source).not.toContain("api.delete(`/api/assets/${assetToDelete}`)");
    expect(source).not.toContain("api.delete(`/api/categories/${categoryToDelete}`)");
    expect(source).not.toContain("确定要删除这个资产吗？此操作无法撤销。");
    expect(source).not.toContain("确定要删除这个分类吗？已关联的资产将取消分类关联。");
  });

  it("delegates depreciation calculation to the backend asset workflow", () => {
    const source = readFileSync(
      resolve(root, "components/assets/AssetDialog.tsx"),
      "utf8",
    );

    expect(source).not.toContain("function calculateMonthlyDepreciation");
    expect(source).not.toContain("assetData.monthly_depreciation");
    expect(source).not.toContain("calculateMonthlyDepreciation(");
  });

  it("keeps asset form numeric and date normalization on the backend", () => {
    const source = readFileSync(
      resolve(root, "components/assets/AssetDialog.tsx"),
      "utf8",
    );

    expect(source).not.toContain("parseFloat");
    expect(source).not.toContain("parseInt");
    expect(source).not.toContain("format(data.purchase_date");
    expect(source).not.toContain("format(data.expiry_date");
    expect(source).not.toContain("salvage_rate ? parse");
    expect(source).not.toContain("filteredCategories");
    expect(source).not.toContain("categories.filter");
    expect(source).not.toContain("/api/assets/overview?asset_type=");
    expect(source).toContain("asset.data_asset_meta");
    expect(source).not.toContain("/api/data_asset_meta");
    expect(source).toContain("api.runAction(action");
    expect(source).not.toContain("api.put(`/api/assets/${asset.id}`");
    expect(source).not.toContain("api.post<{ id: string }>('/api/assets'");
    expect(source).not.toContain("API_ENDPOINTS.dataAssetMeta");
    expect(source).not.toContain("metas.find");
    expect(source).not.toContain("fetchAssetMeta");
    expect(source).toContain("/api/assets/form_options?asset_type=");
    expect(source).toContain("formOptions?.asset_type_options");
    expect(source).toContain("formOptions?.data_type_options");
    expect(source).toContain("formOptions?.depreciation_method_options");
    expect(source).toContain("formOptions?.default_values");
    expect(source).not.toContain("formOptions?.status_options");
    expect(source).not.toContain("status: data.status");
    expect(source).not.toContain('name="status"');
    expect(source).not.toContain("const statusOptions = [");
    expect(source).not.toContain("const assetTypeOptions = [");
    expect(source).not.toContain("const dataTypeOptions = [");
    expect(source).not.toContain("const intangibleTypeOptions = [");
    expect(source).not.toContain("const depreciationMethodOptions = [");
    expect(source).not.toContain("const licenseTypeOptions = [");
    expect(source).not.toContain("const patentTypeOptions = [");
    expect(source).not.toContain("z.enum(['physical'");
    expect(source).not.toContain("z.enum(['in_use'");
    expect(source).not.toContain("z.enum(['straight_line'");
  });

  it("loads asset category form options from the backend", () => {
    const source = readFileSync(
      resolve(root, "components/assets/CategoryDialog.tsx"),
      "utf8",
    );

    expect(source).toContain("/api/categories/form_options");
    expect(source).toContain("formOptions?.category_type_options");
    expect(source).toContain("formOptions?.default_values");
    expect(source).toContain("api.runAction(action");
    expect(source).not.toContain("api.put(`/api/categories/${category.id}`");
    expect(source).not.toContain("api.post('/api/categories'");
    expect(source).not.toContain("z.enum(['physical'");
    expect(source).not.toContain('<SelectItem value="physical">');
  });

  it("keeps asset code labels and badges owned by the backend", () => {
    const source = readFileSync(
      resolve(root, "components/assets/AssetCodeDialog.tsx"),
      "utf8",
    );

    expect(source).toContain("asset.asset_type_badge");
    expect(source).toContain("asset.status_badge");
    expect(source).not.toContain("assetTypeLabels");
    expect(source).not.toContain("statusLabels");
    expect(source).not.toContain("asset.status === 'active'");
    expect(source).not.toContain("asset.status === 'in_use'");
  });
});
