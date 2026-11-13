#!/usr/bin/env python3
"""Audit functional coverage of the reusable 语数科技 E2E dataset."""

import importlib.util
import pathlib
import sys


SCRIPT_DIR = pathlib.Path(__file__).resolve().parent
SEED_SCRIPT = SCRIPT_DIR / "seed_complete_e2e_dataset.py"


FUNCTIONAL_AREAS = {
    "company_and_org": {
        "company_info_profile",
        "branch_hangzhou",
        "profile_ops",
        "user_role_owner",
    },
    "crm_to_cash": {
        "client_primary",
        "contact_primary",
        "interaction_discovery",
        "deal_primary",
        "quote_primary",
        "quote_item_platform",
        "contract_cloud",
        "invoice_income",
        "payment_income",
        "bank_transaction_income",
        "recurring_invoice_support",
        "invoice_reminder_prepayment",
    },
    "delivery_and_rd": {
        "project_primary",
        "milestone_alpha",
        "time_billable",
        "budget_rd",
        "rd_cost_ai",
        "data_asset_meta",
    },
    "people_and_operations": {
        "employee_engineer",
        "employee_accountant",
        "salary_engineer",
        "rent_office",
        "utility_power",
        "vendor_cloud",
        "license_security",
        "reminder_contract",
    },
    "assets_and_cloud": {
        "asset_category_rd",
        "asset_server",
        "asset_data",
        "asset_record_depreciation",
        "cloud_provider",
        "cloud_resource",
        "cloud_bill",
        "software_subscription",
        "domain_subscription",
    },
    "tax_and_compliance": {
        "tax_record_income_q2",
        "tax_declaration_vat",
        "tax_jurisdiction_cn",
        "tax_deadline_cn_vat",
        "social_insurance_shenzhen",
        "insurance_policy_cyber",
        "pension_plan_cn",
    },
    "applications_and_materials": {
        "application_rd_subsidy",
        "application_material_business_plan",
        "material_library_business_plan",
        "material_template_rd_subsidy",
        "material_version_business_plan",
    },
    "platform_and_integrations": {
        "ai_configuration_default",
        "email_configuration_default",
        "email_log_invoice_reminder",
        "exchange_rate_usd_cny",
        "error_log_sample",
        "api_key_agent",
        "webhook_accounting",
        "agent_file_attachment",
    },
}


REQUIRED_ENDPOINTS = {
    "/api/assets",
    "/api/bank/accounts",
    "/api/bank/transactions",
    "/api/api-keys",
    "/api/v1/agent/execute",
    "/api/v1/ai_configurations",
    "/api/v1/application_materials",
    "/api/v1/applications",
    "/api/v1/asset_records",
    "/api/v1/branches",
    "/api/v1/budgets",
    "/api/v1/categories",
    "/api/v1/clients",
    "/api/v1/cloud_bills",
    "/api/v1/cloud_providers",
    "/api/v1/cloud_resources",
    "/api/v1/company_info",
    "/api/v1/contacts",
    "/api/v1/contracts",
    "/api/v1/data_asset_meta",
    "/api/v1/deals",
    "/api/v1/domain_subscriptions",
    "/api/v1/email/send",
    "/api/v1/email_configurations",
    "/api/v1/employees",
    "/api/v1/error_logs",
    "/api/v1/exchange_rates",
    "/api/v1/insurance_policies",
    "/api/v1/interactions",
    "/api/v1/invoice_reminders",
    "/api/v1/invoices",
    "/api/v1/kpi",
    "/api/v1/kpi_configs",
    "/api/v1/kpi_records",
    "/api/v1/licenses",
    "/api/v1/material_library",
    "/api/v1/material_templates",
    "/api/v1/material_versions",
    "/api/v1/milestones",
    "/api/v1/payments",
    "/api/v1/pension_plans",
    "/api/v1/profiles",
    "/api/v1/projects",
    "/api/v1/quote_items",
    "/api/v1/quotes",
    "/api/v1/rd_costs",
    "/api/v1/recurring_invoices",
    "/api/v1/reminders",
    "/api/v1/rent",
    "/api/v1/salary",
    "/api/v1/social_insurance_config",
    "/api/v1/software_subscriptions",
    "/api/v1/tax",
    "/api/v1/tax_deadlines",
    "/api/v1/tax_declarations",
    "/api/v1/tax_jurisdictions",
    "/api/v1/time_entries",
    "/api/v1/user_roles",
    "/api/v1/utilities",
    "/api/v1/vendors",
    "/api/v1/webhooks",
}


def load_seed_module():
    spec = importlib.util.spec_from_file_location("seed_complete_e2e_dataset", SEED_SCRIPT)
    module = importlib.util.module_from_spec(spec)
    assert spec and spec.loader
    spec.loader.exec_module(module)
    return module


def main() -> int:
    module = load_seed_module()
    specs = module.build_dataset()
    slugs = {spec.slug for spec in specs}
    endpoints = {spec.endpoint for spec in specs}

    missing_slugs = {
        area: sorted(required - slugs)
        for area, required in FUNCTIONAL_AREAS.items()
        if required - slugs
    }
    missing_endpoints = sorted(REQUIRED_ENDPOINTS - endpoints)

    print(f"dataset records: {len(specs)}")
    print(f"functional areas: {len(FUNCTIONAL_AREAS)}")
    for area, required in FUNCTIONAL_AREAS.items():
        covered = len(required & slugs)
        print(f"{area}: {covered}/{len(required)}")

    if missing_slugs or missing_endpoints:
        if missing_slugs:
            print("missing slugs:")
            for area, missing in missing_slugs.items():
                print(f"  {area}: {', '.join(missing)}")
        if missing_endpoints:
            print("missing endpoints:")
            for endpoint in missing_endpoints:
                print(f"  {endpoint}")
        return 1

    print("coverage audit passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
