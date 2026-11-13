import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(__dirname, "..");

describe("backend-owned settings UI", () => {
  it("loads setup completion, checks, recommendations, and config display rows from the backend overview endpoint", () => {
    const source = readFileSync(resolve(root, "pages/Settings.tsx"), "utf8");
    const dependencyAlertSource = readFileSync(
      resolve(root, "components/common/DependencyAlert.tsx"),
      "utf8",
    );
    const contracts = readFileSync(resolve(root, "lib/api/contracts.ts"), "utf8");

    expect(contracts).toContain("settingsOverview");
    expect(source).toContain("API_ENDPOINTS.settingsOverview");
    expect(source).toContain("settingsOverview.setup_checks");
    expect(source).toContain("settingsOverview.setup_completion");
    expect(source).toContain("setupCompletion.rate");
    expect(source).toContain("setupCompletion.rate_label");
    expect(source).toContain("setupCompletion.summary_label");
    expect(source).toContain("settingsOverview.ai_configurations");
    expect(source).toContain("settingsOverview.email_configurations");
    expect(source).toContain("settingsOverview.ai_configuration_create_action");
    expect(source).toContain("settingsOverview.email_configuration_create_action");
    expect(source).toContain("settingsOverview.settings_sections");
    expect(source).toContain("section.fields");
    expect(source).toContain("section.items");
    expect(source).toContain("section.toggles");
    expect(source).toContain("field.value");
    expect(source).toContain("item.action.label");
    expect(source).toContain("config.display_name");
    expect(source).toContain("config.endpoint_label");
    expect(source).toContain("config.sender_label");
    expect(source).toContain("config.api_key_label");
    expect(source).toContain("config.delete_action");
    expect(source).toContain("config.update_action");
    expect(source).toContain("result.impact_label");
    expect(source).toContain("result.card_class_name");
    expect(source).toContain("result.status_icon");
    expect(source).toContain("result.status_badge");
    expect(source).toContain("result.action_button");
    expect(source).toContain("setupCompletion.success_alert");
    expect(source).toContain("recommendation_items");

    expect(source).not.toContain("company-info-check");
    expect(source).not.toContain("employees-check");
    expect(source).not.toContain("asset-categories-check");
    expect(source).not.toContain("branches-check");
    expect(source).not.toContain("ai-configurations");
    expect(source).not.toContain("email-configurations");
    expect(source).not.toContain("const checkResults = [");
    expect(source).not.toContain("completedCount");
    expect(source).not.toContain("completionRate");
    expect(source).not.toContain("Math.round");
    expect(source).not.toContain(".filter(r => r.status)");
    expect(source).not.toContain("config.api_key?.substring");
    expect(source).not.toContain("result.status ?");
    expect(source).not.toContain("!result.status");
    expect(source).not.toContain("!setupCompletion.needs_attention");
    expect(source).not.toContain("border-green-200 bg-green-50/50");
    expect(source).not.toContain("bg-orange-100 text-orange-700");
    expect(source).not.toContain("科技创新有限公司");
    expect(source).not.toContain("91110000MA01234567");
    expect(source).not.toContain("北京市朝阳区创新路123号");
    expect(source).not.toContain("2024-01-20 02:00:00");
    expect(source).not.toContain('["研发部", "销售部", "财务部"');
    expect(source).not.toContain("defaultChecked");
    expect(source).not.toContain("确定要删除这个 AI 配置吗");
    expect(source).not.toContain("确定要删除这个邮件配置吗");
    expect(source).not.toContain("AI 配置已删除");
    expect(source).not.toContain("邮件配置已删除");

    expect(dependencyAlertSource).toContain("/api/dependencies/${dependency}");
    expect(dependencyAlertSource).toContain("dependencyStatus?.exists");
    expect(dependencyAlertSource).toContain("dependencyStatus?.message");
    expect(dependencyAlertSource).toContain("dependencyStatus?.action_text");
    expect(dependencyAlertSource).not.toContain("DEPENDENCY_CONFIGS");
    expect(dependencyAlertSource).not.toContain("config.table");
    expect(dependencyAlertSource).not.toContain("api.get<{ data: any[] }>");
  });

  it("keeps settings config ownership on the backend", () => {
    const source = [
      readFileSync(resolve(root, "pages/Settings.tsx"), "utf8"),
      readFileSync(resolve(root, "components/settings/AIConfigDialog.tsx"), "utf8"),
      readFileSync(resolve(root, "components/settings/EmailConfigDialog.tsx"), "utf8"),
    ].join("\n");

    expect(source).toContain("type BackendAction");
    expect(source).toContain("api.runAction(action");
    expect(source).toContain("config?.update_action");
    expect(source).toContain("createAction");
    expect(source).not.toContain("api.post(API_ENDPOINTS.aiConfigurations");
    expect(source).not.toContain("api.put(`${API_ENDPOINTS.aiConfigurations}");
    expect(source).not.toContain("api.post(API_ENDPOINTS.emailConfigurations");
    expect(source).not.toContain("api.put(`${API_ENDPOINTS.emailConfigurations}");
    expect(source).not.toContain("useAuth");
    expect(source).not.toContain("user_id: user.id");
    expect(source).not.toContain("请先登录");
  });
});
