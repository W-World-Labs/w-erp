import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(__dirname, "..");

describe("backend-owned rent and utility management UI", () => {
  it("loads rent and utility totals, trends, and expiry counts from the backend overview endpoint", () => {
    const source = readFileSync(resolve(root, "pages/Rent.tsx"), "utf8");

    expect(source).toContain("/api/v1/rent/overview");
    expect(source).toContain("rent_search=");
    expect(source).toContain("utility_search=");
    expect(source).toContain("rentStats.total_label");
    expect(source).toContain("rentStats.area_label");
    expect(source).toContain("utilityStats.total_label");
    expect(source).toContain("rent.status_badge");
    expect(source).toContain("rent.monthly_rent_label");
    expect(source).toContain("rent.area_label");
    expect(source).toContain("rent.contract_period_label");
    expect(source).toContain("rent.next_payment_label");
    expect(source).toContain("overview.rent_create_action");
    expect(source).toContain("selectedRent.update_action");
    expect(source).toContain("rent.can_delete");
    expect(source).toContain("rent.delete_action");
    expect(source).toContain("utility.utility_type_icon");
    expect(source).toContain("utility.utility_type_label");
    expect(source).toContain("utility.status_badge");
    expect(source).toContain("utility.unit_price_label");
    expect(source).toContain("utility.amount_label");
    expect(source).toContain("overview.utility_create_action");
    expect(source).toContain("selectedUtility.update_action");
    expect(source).toContain("utility.can_mark_paid");
    expect(source).toContain("utility.mark_paid_action");
    expect(source).toContain("api.runAction(action)");
    expect(source).not.toContain("api.post(API_ENDPOINTS.rent");
    expect(source).not.toContain("api.put(`${API_ENDPOINTS.rent}/${selectedRent.id}`");
    expect(source).not.toContain("api.post(API_ENDPOINTS.utilities");
    expect(source).not.toContain("api.put(\n          `${API_ENDPOINTS.utilities}/${selectedUtility.id}`");
    expect(source).not.toContain("/mark_paid");
    expect(source).not.toContain("API_ENDPOINTS.utilities}/${id}/mark_paid");
    expect(source).toContain("utility.can_delete");
    expect(source).toContain("utility.delete_action");
    expect(source).toContain("payload.total_label");
    expect(source).toContain("payload.electricity_label");
    expect(source).toContain("overview.empty_messages.rents");
    expect(source).toContain("overview.empty_messages.utilities");
    expect(source).not.toContain("const rentTrend = (() =>");
    expect(source).not.toContain("const utilityTrend = (() =>");
    expect(source).not.toContain("const rentStats = {");
    expect(source).not.toContain("const utilityStats = {");
    expect(source).not.toContain("threeMonthsLater");
    expect(source).not.toContain("subMonths");
    expect(source).not.toContain(".reduce((sum, r)");
    expect(source).not.toContain(".reduce((sum, u)");
    expect(source).not.toContain("useAuth");
    expect(source).not.toContain("getStatusBadge");
    expect(source).not.toContain("getTypeIcon");
    expect(source).not.toContain("getTypeName");
    expect(source).not.toContain("filteredRents");
    expect(source).not.toContain("filteredUtilities");
    expect(source).not.toContain("rents?.filter");
    expect(source).not.toContain("utilities?.filter");
    expect(source).not.toContain("toLocaleString");
    expect(source).not.toContain("toFixed");
    expect(source).not.toContain("format(new Date()");
    expect(source).not.toContain('status: "paid"');
    expect(source).not.toContain('type === "rent"');
    expect(source).not.toContain('variables.type === "rent"');
    expect(source).not.toContain("API_ENDPOINTS.rent : API_ENDPOINTS.utilities");
  });

  it("keeps rent and utility form numeric calculations on the backend", () => {
    const source = [
      readFileSync(resolve(root, "components/rent/RentDialog.tsx"), "utf8"),
      readFileSync(
        resolve(root, "components/utilities/UtilityDialog.tsx"),
        "utf8",
      ),
    ].join("\n");

    expect(source).toContain("/api/v1/rent/form_options");
    expect(source).toContain("formOptions?.payment_cycle_options");
    expect(source).toContain("formOptions?.utility_type_options");
    expect(source).toContain("formOptions?.default_values");
    expect(source).not.toContain("parseFloat");
    expect(source).not.toContain("usage * unitPrice");
    expect(source).not.toContain("const amount =");
    expect(source).not.toContain("format(new Date()");
    expect(source).not.toContain("contract_start_date: format(new Date()");
    expect(source).not.toContain("period: format(new Date()");
    expect(source).not.toContain("<SelectItem value=\"monthly\">");
    expect(source).not.toContain("<SelectItem value=\"electricity\">");
  });
});
