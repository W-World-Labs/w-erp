import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(__dirname, "..");

describe("backend-owned contracts UI", () => {
  it("loads stats, filtering, status labels, and AI risk labels from backend overview", () => {
    const source = readFileSync(resolve(root, "pages/Contracts.tsx"), "utf8");
    const contracts = readFileSync(
      resolve(root, "lib/api/contracts.ts"),
      "utf8",
    );

    expect(source).toContain("API_ENDPOINTS.contractsOverview");
    expect(contracts).toContain("contractsOverview");
    expect(source).toContain("overview.permissions");
    expect(source).toContain("overview.create_action");
    expect(source).toContain("selectedContract.update_action");
    expect(source).toContain("api.runAction(action");
    expect(source).toContain("contract.delete_action");
    expect(source).toContain("stats.ai_guard_display");
    expect(source).not.toContain("api.post(API_ENDPOINTS.contracts");
    expect(source).not.toContain("api.put(`${API_ENDPOINTS.contracts}/${id}`");
    expect(source).not.toContain('status_options: [{ value: "all", label: "全部状态" }]');
    expect(source).not.toContain("useAuth()");
    expect(source).not.toContain("isAdmin");
    expect(source).not.toContain("handleDelete(contract.id)");
    expect(source).not.toContain(
      "api.delete(`${API_ENDPOINTS.contracts}/${id}`)",
    );
    expect(source).not.toContain("deleteMutation.mutate(contractToDelete)");

    expect(source).not.toContain('queryKey: ["contracts"]');
    expect(source).not.toContain("contracts.filter");
    expect(source).not.toContain(".reduce(");
    expect(source).not.toContain("activeCount");
    expect(source).not.toContain("expiredCount");
    expect(source).not.toContain("highRiskCount");
    expect(source).not.toContain("getStatusBadge");
    expect(source).not.toContain("contract.status ===");
    expect(source).not.toContain("ai_risk_level?.includes");
    expect(source).not.toContain("guardDanger");
    expect(source).not.toContain('stats.ai_guard_tone ===');

    const reviewDialog = readFileSync(
      resolve(root, "components/contracts/ContractReviewDialog.tsx"),
      "utf8",
    );
    expect(reviewDialog).toContain("contract.ai_reviewed_at_label");
    expect(reviewDialog).toContain("contract.ai_review_display");
    expect(reviewDialog).toContain("contract.counterparty_type_badge");
    expect(reviewDialog).toContain("contract.retrigger_review_action");
    expect(reviewDialog).toContain("api.runAction(action)");
    expect(reviewDialog).not.toContain("/retrigger_review");
    expect(reviewDialog).not.toContain("API_ENDPOINTS.contracts");
    expect(reviewDialog).not.toContain("toLocaleString");
    expect(reviewDialog).not.toContain("new Date(contract.ai_reviewed_at)");
    expect(reviewDialog).not.toContain("counterpartyLabels");
    expect(reviewDialog).not.toContain("getRiskConfig");
    expect(reviewDialog).not.toContain("contract.ai_review_status ===");
    expect(reviewDialog).not.toContain("_retrigger_review");
    expect(reviewDialog).not.toContain("content: contract?.content");
    expect(reviewDialog).not.toContain("api.put(`/api/contracts/");
  });

  it("stores backend-owned uploaded file ids instead of frontend-generated paths", () => {
    const source = readFileSync(resolve(root, "pages/Contracts.tsx"), "utf8");
    const preview = readFileSync(
      resolve(root, "components/common/FilePreviewDialog.tsx"),
      "utf8",
    );

    expect(source).toContain("const uploadResult = await files.upload(values.file)");
    expect(source).toContain("file_path = uploadResult.id");
    expect(source).toContain("file_name = uploadResult.original_name");
    expect(source).not.toContain("crypto.randomUUID");
    expect(source).not.toContain("fileExt");

    expect(preview).toContain("files.download(filePath)");
    expect(preview).not.toContain("/api/files/download");
  });

  it("keeps contract form amount normalization on the backend", () => {
    const dialog = readFileSync(
      resolve(root, "components/contracts/ContractDialog.tsx"),
      "utf8",
    );

    expect(dialog).not.toContain("parseFloat");
    expect(dialog).not.toContain("amount: parse");
    expect(dialog).not.toContain("format(new Date()");
    expect(dialog).not.toContain("start_date: format(new Date()");
    expect(dialog).toContain("/api/v1/contracts/form_options");
    expect(dialog).toContain("formOptions?.contract_type_options");
    expect(dialog).toContain("formOptions?.counterparty_type_options");
    expect(dialog).toContain("formOptions?.default_values");
    expect(dialog).not.toContain('<SelectItem value="purchase">');
    expect(dialog).not.toContain('<SelectItem value="vc">');
  });
});
