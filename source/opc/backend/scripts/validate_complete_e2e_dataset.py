#!/usr/bin/env python3
"""Read-only E2E validation for the reusable 语数科技 test dataset."""

import argparse
import json
import os
import sys
from typing import Any

import requests


RELATION_CHECKS = [
    ("asset_server", "category_id", "asset_category_rd"),
    ("asset_data", "category_id", "asset_category_rd"),
    ("contact_primary", "client_id", "client_primary"),
    ("interaction_discovery", "client_id", "client_primary"),
    ("interaction_discovery", "contact_id", "contact_primary"),
    ("deal_primary", "client_id", "client_primary"),
    ("deal_primary", "contact_id", "contact_primary"),
    ("quote_primary", "client_id", "client_primary"),
    ("quote_primary", "contact_id", "contact_primary"),
    ("quote_primary", "deal_id", "deal_primary"),
    ("quote_item_platform", "quote_id", "quote_primary"),
    ("milestone_alpha", "project_id", "project_primary"),
    ("kpi_record_mrr", "kpi_id", "kpi_config_mrr"),
    ("application_rd_subsidy", "project_id", "project_primary"),
    ("material_library_business_plan", "source_id", "application_rd_subsidy"),
    ("application_material_business_plan", "application_id", "application_rd_subsidy"),
    ("application_material_business_plan", "reuse_source", "material_library_business_plan"),
    ("material_version_business_plan", "material_id", "material_library_business_plan"),
    ("time_billable", "project_id", "project_primary"),
    ("time_billable", "client_id", "client_primary"),
    ("time_billable", "employee_id", "employee_engineer"),
    ("salary_engineer", "employee_id", "employee_engineer"),
    ("budget_rd", "project_id", "project_primary"),
    ("contract_cloud", "project_id", "project_primary"),
    ("invoice_income", "contract_id", "contract_cloud"),
    ("payment_income", "bank_account_id", "bank_operating"),
    ("payment_income", "invoice_id", "invoice_income"),
    ("bank_transaction_income", "account_id", "bank_operating"),
    ("invoice_reminder_prepayment", "invoice_id", "invoice_income"),
    ("invoice_reminder_prepayment", "client_id", "client_primary"),
    ("asset_record_depreciation", "asset_id", "asset_server"),
    ("data_asset_meta", "asset_id", "asset_data"),
    ("rd_cost_ai", "project_id", "project_primary"),
    ("rd_cost_ai", "employee_id", "employee_engineer"),
    ("rd_cost_ai", "asset_id", "asset_server"),
    ("rd_cost_ai", "invoice_id", "invoice_income"),
    ("rent_office", "contract_id", "contract_cloud"),
    ("utility_power", "rent_record_id", "rent_office"),
    ("tax_declaration_vat", "invoice_id", "invoice_income"),
    ("tax_declaration_vat", "payment_id", "payment_income"),
    ("tax_deadline_cn_vat", "jurisdiction_id", "tax_jurisdiction_cn"),
    ("cloud_resource", "provider_id", "cloud_provider"),
    ("cloud_bill", "provider_id", "cloud_provider"),
    ("profile_ops", "user_id", "current_user"),
    ("user_role_owner", "user_id", "current_user"),
    ("email_log_invoice_reminder", "entity_id", "invoice_income"),
    ("reminder_contract", "entity_id", "contract_cloud"),
]


DIRECT_ENDPOINTS = {
    "company_info_profile": "/api/v1/company_info",
    "social_insurance_shenzhen": "/api/v1/social_insurance_config",
    "asset_category_rd": "/api/v1/categories",
    "branch_hangzhou": "/api/v1/branches",
    "kpi_metric_mrr": "/api/v1/kpi",
    "kpi_config_mrr": "/api/v1/kpi_configs",
    "kpi_record_mrr": "/api/v1/kpi_records",
    "tax_record_income_q2": "/api/v1/tax",
    "client_primary": "/api/v1/clients",
    "contact_primary": "/api/v1/contacts",
    "interaction_discovery": "/api/v1/interactions",
    "deal_primary": "/api/v1/deals",
    "quote_primary": "/api/v1/quotes",
    "quote_item_platform": "/api/v1/quote_items",
    "recurring_invoice_support": "/api/v1/recurring_invoices",
    "project_primary": "/api/v1/projects",
    "milestone_alpha": "/api/v1/milestones",
    "application_rd_subsidy": "/api/v1/applications",
    "application_material_business_plan": "/api/v1/application_materials",
    "material_library_business_plan": "/api/v1/material_library",
    "material_template_rd_subsidy": "/api/v1/material_templates",
    "material_version_business_plan": "/api/v1/material_versions",
    "employee_engineer": "/api/v1/employees",
    "employee_accountant": "/api/v1/employees",
    "time_billable": "/api/v1/time_entries",
    "salary_engineer": "/api/v1/salary",
    "budget_rd": "/api/v1/budgets",
    "bank_operating": "/api/bank/accounts",
    "bank_transaction_income": "/api/v1/bank/transactions",
    "vendor_cloud": "/api/v1/vendors",
    "contract_cloud": "/api/v1/contracts",
    "invoice_income": "/api/v1/invoices",
    "invoice_reminder_prepayment": "/api/v1/invoice_reminders",
    "payment_income": "/api/v1/payments",
    "asset_server": "/api/assets",
    "asset_data": "/api/assets",
    "asset_record_depreciation": "/api/v1/asset_records",
    "data_asset_meta": "/api/v1/data_asset_meta",
    "rd_cost_ai": "/api/v1/rd_costs",
    "license_security": "/api/v1/licenses",
    "rent_office": "/api/v1/rent",
    "utility_power": "/api/v1/utilities",
    "tax_declaration_vat": "/api/v1/tax_declarations",
    "tax_jurisdiction_cn": "/api/v1/tax_jurisdictions",
    "tax_deadline_cn_vat": "/api/v1/tax_deadlines",
    "cloud_provider": "/api/v1/cloud_providers",
    "cloud_resource": "/api/v1/cloud_resources",
    "cloud_bill": "/api/v1/cloud_bills",
    "software_subscription": "/api/v1/software_subscriptions",
    "domain_subscription": "/api/v1/domain_subscriptions",
    "insurance_policy_cyber": "/api/v1/insurance_policies",
    "pension_plan_cn": "/api/v1/pension_plans",
    "profile_ops": "/api/v1/profiles",
    "user_role_owner": "/api/v1/user_roles",
    "ai_configuration_default": "/api/v1/ai_configurations",
    "email_configuration_default": "/api/v1/email_configurations",
    "exchange_rate_usd_cny": "/api/v1/exchange_rates",
    "error_log_sample": "/api/v1/error_logs",
    "reminder_contract": "/api/v1/reminders",
}


LIST_ONLY_ENDPOINTS = {
    "email_log_invoice_reminder": "/api/v1/email/logs",
    "api_key_agent": "/api/api-keys",
    "webhook_accounting": "/api/v1/webhooks",
    "agent_file_attachment": None,
}


def unwrap_record(body: Any) -> dict[str, Any]:
    if isinstance(body, dict) and isinstance(body.get("data"), dict):
        return body["data"]
    if isinstance(body, dict):
        return body
    raise AssertionError(f"expected object response, got {body!r}")


def extract_rows(body: Any) -> list[dict[str, Any]]:
    if isinstance(body, list):
        return [item for item in body if isinstance(item, dict)]
    if isinstance(body, dict):
        for key in ("data", "items", "rows", "files", "keys"):
            value = body.get(key)
            if isinstance(value, list):
                return [item for item in value if isinstance(item, dict)]
    return []


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


class Validator:
    def __init__(self, base_url: str, session: str, manifest: dict[str, Any]) -> None:
        self.base_url = base_url.rstrip("/")
        self.manifest = manifest
        self.ids = manifest["ids"]
        self.http = requests.Session()
        self.http.headers.update({"Authorization": f"Bearer {session}"})
        self.records: dict[str, dict[str, Any]] = {}
        self.passed = 0

    def url(self, endpoint: str) -> str:
        return f"{self.base_url}{endpoint}"

    def check(self, name: str, fn) -> None:
        fn()
        self.passed += 1
        print(f"ok {name}")

    def get_json(self, endpoint: str) -> Any:
        response = self.http.get(self.url(endpoint), timeout=15)
        require(200 <= response.status_code < 300, f"{endpoint} returned {response.status_code}: {response.text[:300]}")
        return response.json()

    def validate_auth_and_company(self) -> None:
        me = self.get_json("/auth/me")
        require(me.get("email"), "auth/me should return current user email")
        require(me.get("id") == self.ids["current_user"], "auth/me should match manifest current_user")
        companies = self.get_json("/api/companies")
        rows = companies.get("data", [])
        company = next((row for row in rows if row.get("id") == self.manifest["company_id"]), None)
        require(company is not None, "manifest company_id should be listed")
        require(company.get("name") == "语数科技", "manifest company should be 语数科技")

        anonymous = requests.get(self.url("/api/companies"), timeout=15)
        require(anonymous.status_code == 401, "anonymous /api/companies should be 401")

    def validate_records_exist(self) -> None:
        for slug, endpoint in DIRECT_ENDPOINTS.items():
            record_id = self.ids.get(slug)
            require(record_id, f"manifest missing {slug}")
            record = unwrap_record(self.get_json(f"{endpoint}/{record_id}"))
            require(record.get("id") == record_id, f"{slug} id mismatch")
            self.records[slug] = record

    def validate_list_only_records_exist(self) -> None:
        for slug, endpoint in LIST_ONLY_ENDPOINTS.items():
            record_id = self.ids.get(slug)
            require(record_id, f"manifest missing {slug}")
            if endpoint is None:
                continue
            rows = extract_rows(self.get_json(endpoint))
            record = next((row for row in rows if row.get("id") == record_id), None)
            require(record is not None, f"{slug} should be present in {endpoint}")
            self.records[slug] = record

    def validate_relations(self) -> None:
        for slug, field, target_slug in RELATION_CHECKS:
            actual = self.records[slug].get(field)
            expected = self.ids[target_slug]
            require(actual == expected, f"{slug}.{field} expected {expected}, got {actual}")

    def validate_dataset_semantics(self) -> None:
        require(self.records["company_info_profile"]["company_name"] == "语数科技", "company_info should identify 语数科技")
        require(self.records["asset_data"]["asset_type"] == "data", "asset_data should be a data asset")
        require(self.records["data_asset_meta"]["data_type"] == "dataset", "data_asset_meta should describe a dataset")
        require(self.records["payment_income"]["status"] == "completed", "payment should be completed")
        require(float(self.records["invoice_income"]["amount"]) > 0, "invoice amount should be positive")
        require(float(self.records["cloud_bill"]["total_amount"]) > 0, "cloud bill total should be positive")
        require(self.records["application_material_business_plan"]["status"] == "completed", "application material should be completed")
        require(self.records["recurring_invoice_support"]["status"] == "active", "recurring invoice should be active")
        require(self.records["tax_jurisdiction_cn"]["code"] == "CN", "tax jurisdiction should cover CN")
        require(self.records["email_configuration_default"]["smtp_password"].startswith("w-id://workspace/"), "SMTP password should use a W-ID secret ref")
        require(self.records["ai_configuration_default"]["api_key"].startswith("w-id://workspace/"), "AI API key should use a W-ID secret ref")
        require("key" not in self.records.get("api_key_agent", {}), "API key list should not expose raw key")

    def validate_operational_endpoints(self) -> None:
        for endpoint in [
            "/api/stats/dashboard",
            "/api/system/status",
            "/api/v1/kpi/dashboard",
            "/api/v1/applications/overview",
            f"/api/v1/application_materials/overview?application_id={self.ids['application_rd_subsidy']}",
            "/api/v1/recurring_invoices/overview",
            "/api/v1/tax_calendar/overview",
            "/api/v1/settings/overview",
            "/api/v1/operation_logs/overview",
            "/api/v1/audit_center/overview",
            "/api/v1/cloud_cost/overview",
            "/api/v1/reports/financial-health",
            "/api/v1/reports/cashflow-forecast",
            "/api/v1/reports/profit-analysis",
            "/api/v1/reports/deals-conversion",
            "/api/v1/currency/convert?from=USD&to=CNY&amount=100",
            "/api/v1/ar/summary",
            "/api/v1/ar/aging",
            "/api/v1/security/audit",
            "/api/v1/api-catalog",
            "/api/v1/email/logs",
            "/api/api-keys",
            "/api/v1/webhooks",
            "/api/v1/agent/tools",
            f"/api/v1/files/list/payments/{self.ids['payment_income']}",
        ]:
            self.get_json(endpoint)
        files = self.get_json(f"/api/v1/files/list/payments/{self.ids['payment_income']}").get("data", [])
        require(
            any(item.get("id") == self.ids["agent_file_attachment"] for item in files),
            "payment evidence attachment should be listed",
        )

    def run(self) -> None:
        self.check("auth/company isolation", self.validate_auth_and_company)
        self.check("all manifest records exist", self.validate_records_exist)
        self.check("list-only records exist", self.validate_list_only_records_exist)
        self.check("business relations are intact", self.validate_relations)
        self.check("dataset semantics", self.validate_dataset_semantics)
        self.check("operational endpoints", self.validate_operational_endpoints)
        print(f"validated {self.passed} groups for {self.manifest['company_name']}")


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Validate the complete 语数科技 E2E dataset.")
    parser.add_argument("--base-url", default=os.environ.get("ONEMAN_BASE_URL", "http://127.0.0.1:8987"))
    parser.add_argument("--session", default=os.environ.get("SESSION_ID"), required=False)
    parser.add_argument("--manifest", required=True)
    return parser.parse_args(argv)


def main(argv: list[str]) -> int:
    args = parse_args(argv)
    if not args.session:
        print("SESSION_ID is required", file=sys.stderr)
        return 2
    with open(args.manifest, encoding="utf-8") as handle:
        manifest = json.load(handle)
    Validator(args.base_url, args.session, manifest).run()
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
