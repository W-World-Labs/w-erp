import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(__dirname, "..");

describe("backend-owned cloud cost UI", () => {
  it("loads cloud cost totals from the backend overview endpoint", () => {
    const files = [
      "pages/CloudCostManagement.tsx",
      "components/cloud-cost/CostOverviewStats.tsx",
      "components/cloud-cost/CloudProviderList.tsx",
      "components/cloud-cost/CloudResourceList.tsx",
      "components/cloud-cost/CloudBillList.tsx",
      "components/cloud-cost/DomainSubscriptionList.tsx",
      "components/cloud-cost/SoftwareSubscriptionList.tsx",
    ];
    const source = files
      .map((file) => readFileSync(resolve(root, file), "utf8"))
      .join("\n");

    expect(source).toContain("/api/v1/cloud_cost/overview");
    expect(source).toContain("monthly_cloud_cost_label");
    expect(source).toContain("provider.status_badge");
    expect(source).toContain("provider.delete_action");
    expect(source).toContain("resource.monthly_cost_label");
    expect(source).toContain("resource.delete_action");
    expect(source).toContain("resource.can_start");
    expect(source).toContain("resource.can_stop");
    expect(source).toContain("resource.can_terminate");
    expect(source).toContain("resource.start_action");
    expect(source).toContain("resource.stop_action");
    expect(source).toContain("resource.terminate_action");
    expect(source).toContain("api.runAction(action)");
    expect(source).not.toContain("/start");
    expect(source).not.toContain("/stop");
    expect(source).not.toContain("/terminate");
    expect(source).toContain("bill.total_amount_label");
    expect(source).toContain("bill.delete_action");
    expect(source).toContain("bill.update_action");
    expect(source).toContain("bill.can_mark_paid");
    expect(source).toContain("bill.mark_paid_action");
    expect(source).not.toContain("/mark_paid");
    expect(source).toContain("domain.expiry_state");
    expect(source).toContain("domain.delete_action");
    expect(source).toContain("domain.can_activate");
    expect(source).toContain("domain.can_mark_pending_transfer");
    expect(source).toContain("domain.activate_action");
    expect(source).toContain("domain.mark_pending_transfer_action");
    expect(source).not.toContain("/activate");
    expect(source).not.toContain("/mark_pending_transfer");
    expect(source).toContain("sub.billing_cycle_label");
    expect(source).toContain("sub.delete_action");
    expect(source).toContain("sub.can_start_trial");
    expect(source).toContain("sub.can_cancel");
    expect(source).toContain("sub.start_trial_action");
    expect(source).toContain("sub.cancel_action");
    expect(source).not.toContain("/start_trial");
    expect(source).not.toContain("/cancel");

    expect(source).not.toContain("useAuth");
    expect(source).not.toContain("cloudProviders");
    expect(source).not.toContain("cloudResources");
    expect(source).not.toContain("domainSubscriptions");
    expect(source).not.toContain("softwareSubscriptions");
    expect(source).not.toContain(".reduce((sum");
    expect(source).not.toContain("totalMonthlyCost =");
    expect(source).not.toContain(".toFixed");
    expect(source).not.toContain("safeFormat");
    expect(source).not.toContain("differenceInDays");
    expect(source).not.toContain("getProviderTypeLabel");
    expect(source).not.toContain("getResourceTypeLabel");
    expect(source).not.toContain("getStatusColor");
    expect(source).not.toContain("getStatusLabel");
    expect(source).not.toContain("billing_cycle ===");
    expect(source).not.toContain("api.delete(`/api/v1/cloud_providers/${deletingId}`)");
    expect(source).not.toContain("api.delete(`/api/v1/cloud_resources/${deletingId}`)");
    expect(source).not.toContain("api.delete(`/api/v1/cloud_bills/${deletingId}`)");
    expect(source).not.toContain("api.delete(`/api/v1/domain_subscriptions/${deletingId}`)");
    expect(source).not.toContain("api.delete(`/api/v1/software_subscriptions/${deletingId}`)");
  });

  it("keeps cloud cost form numeric normalization on the backend", () => {
    const files = [
      "components/cloud-cost/CloudProviderDialog.tsx",
      "components/cloud-cost/SoftwareSubscriptionDialog.tsx",
      "components/cloud-cost/CloudBillDialog.tsx",
      "components/cloud-cost/CloudResourceDialog.tsx",
      "components/cloud-cost/DomainSubscriptionDialog.tsx",
    ];
    const source = files
      .map((file) => readFileSync(resolve(root, file), "utf8"))
      .join("\n");

    expect(source).not.toContain("parseFloat");
    expect(source).not.toContain("parseInt");
    expect(source).not.toContain("monthly_cost: data.monthly_cost ? parse");
    expect(source).not.toContain("total_amount: parse");
    expect(source).not.toContain("license_count: parse");
    expect(source).not.toContain("new Date().toISOString");
    expect(source).not.toContain("start_date: new Date()");
    expect(source).toContain("billing_period: data.bill_month || undefined");
    expect(source).not.toContain("bill_date: data.bill_date");
    expect(source).not.toContain("payment_status: data.payment_status");
    expect(source).not.toContain("status: data.payment_status");
    expect(source).not.toContain("payment_status:");
    expect(source).not.toContain("payment_date:");
    expect(source).not.toContain("status: data.status");
    expect(source).not.toContain('name="status"');
    expect(source).not.toContain("formOptions?.resource_status_options");
    expect(source).not.toContain("formOptions?.domain_status_options");
    expect(source).not.toContain("formOptions?.software_status_options");
    expect(source).not.toContain("defaults.resource_status");
    expect(source).not.toContain("defaults.domain_status");
    expect(source).not.toContain("defaults.software_status");
    expect(source).toContain("api.runAction(action");
    expect(source).not.toContain("api.put(`${API_ENDPOINTS.cloudBills}/${initialData.id}`");
    expect(source).not.toContain("api.post(API_ENDPOINTS.cloudBills");
  });

  it("loads cloud cost form options and defaults from the backend", () => {
    const files = [
      "components/cloud-cost/CloudProviderDialog.tsx",
      "components/cloud-cost/SoftwareSubscriptionDialog.tsx",
      "components/cloud-cost/CloudBillDialog.tsx",
      "components/cloud-cost/CloudResourceDialog.tsx",
      "components/cloud-cost/DomainSubscriptionDialog.tsx",
    ];
    const source = files
      .map((file) => readFileSync(resolve(root, file), "utf8"))
      .join("\n");

    expect(source).toContain("/api/v1/cloud_cost/form_options");
    expect(source).toContain("formOptions?.provider_type_options");
    expect(source).toContain("formOptions?.provider_options");
    expect(source).toContain("formOptions?.resource_type_options");
    expect(source).toContain("formOptions?.purchase_type_options");
    expect(source).not.toContain("formOptions?.resource_status_options");
    expect(source).not.toContain("billStatusOptions");
    expect(source).not.toContain("formOptions?.domain_status_options");
    expect(source).toContain("formOptions?.software_subscription_type_options");
    expect(source).toContain("formOptions?.billing_cycle_options");
    expect(source).not.toContain("formOptions?.software_status_options");
    expect(source).toContain("formOptions?.default_values");
    expect(source).not.toContain("<SelectItem value=\"pending\">");
    expect(source).not.toContain("<SelectItem value=\"active\">");
    expect(source).not.toContain("<SelectItem value=\"monthly\">");
    expect(source).not.toContain("<SelectItem value=\"quarterly\">");
    expect(source).not.toContain("<SelectItem value=\"running\">");
    expect(source).not.toContain("<SelectItem value=\"aliyun\">");
    expect(source).not.toContain("useAuth");
  });

  it("loads cloud provider select options from the backend form options endpoint", () => {
    const files = [
      "components/cloud-cost/CloudBillDialog.tsx",
      "components/cloud-cost/CloudResourceDialog.tsx",
    ];
    const source = files
      .map((file) => readFileSync(resolve(root, file), "utf8"))
      .join("\n");

    expect(source).toContain("/api/v1/cloud_cost/form_options");
    expect(source).toContain("provider_options");
    expect(source).not.toContain("API_ENDPOINTS.cloudProviders");
    expect(source).not.toContain("cloudProviders.list");
    expect(source).not.toContain("useAuth");
    expect(source).not.toContain("provider_name");
  });
});
