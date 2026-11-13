import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(__dirname, "..");

describe("backend-owned vendors UI", () => {
  it("loads vendor statistics, filtering, options, and badges from the backend overview endpoint", () => {
    const source = readFileSync(resolve(root, "pages/Vendors.tsx"), "utf8");

    expect(source).toContain("/api/v1/vendors/overview");
    expect(source).toContain("vendor_type");
    expect(source).toContain("statusFilter");
    expect(source).toContain("overview.type_options");
    expect(source).toContain("overview.status_options");
    expect(source).toContain("vendor.type_badge");
    expect(source).toContain("vendor.status_badge");
    expect(source).toContain("overview.create_action");
    expect(source).toContain("selectedVendor.update_action");
    expect(source).toContain("api.runAction(action");
    expect(source).toContain("vendor.delete_action");
    expect(source).toContain("overview.empty_message");
    expect(source).not.toContain('api.post("/api/vendors"');
    expect(source).not.toContain("api.put(`/api/vendors/${id}`");
    expect(source).not.toContain('<SelectItem value="all">全部类型</SelectItem>');
    expect(source).not.toContain('<SelectItem value="all">全部状态</SelectItem>');
    expect(source).not.toContain("setVendorToDelete(vendor.id)");
    expect(source).not.toContain("deleteMutation.mutate(vendorToDelete)");
    expect(source).not.toContain("api.delete(`/api/vendors/${id}`)");

    expect(source).not.toContain("getTypeBadge");
    expect(source).not.toContain("getStatusBadge");
    expect(source).not.toContain("filteredVendors");
    expect(source).not.toContain("vendors.filter");
    expect(source).not.toContain("typeFilter ===");
    expect(source).not.toContain("v.vendor_type");
    expect(source).not.toContain(
      "vendors.filter((v: any) => v.vendor_type === 'supplier'",
    );
    expect(source).not.toContain(
      "vendors.filter((v: any) => v.vendor_type === 'customer'",
    );
    expect(source).not.toContain(
      "vendors.filter((v: any) => v.status === 'active'",
    );
    expect(source).not.toContain("const supplierCount =");
    expect(source).not.toContain("const customerCount =");
    expect(source).not.toContain("const activeCount =");
  });

  it("loads vendor form options from the backend", () => {
    const source = readFileSync(
      resolve(root, "components/vendors/VendorDialog.tsx"),
      "utf8",
    );

    expect(source).toContain("/api/v1/vendors/form_options");
    expect(source).toContain("formOptions?.vendor_type_options");
    expect(source).toContain("formOptions?.status_options");
    expect(source).toContain("formOptions?.default_values");
    expect(source).not.toContain("z.enum(['supplier'");
    expect(source).not.toContain("z.enum(['active'");
    expect(source).not.toContain('<SelectItem value="supplier">');
    expect(source).not.toContain('<SelectItem value="active">');
  });
});
