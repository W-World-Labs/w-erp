#!/usr/bin/env python3
"""Contract tests for the reusable OPC E2E seed dataset."""

import importlib.util
import pathlib
import unittest


SCRIPT = pathlib.Path(__file__).with_name("seed_complete_e2e_dataset.py")


def load_module():
    spec = importlib.util.spec_from_file_location("seed_complete_e2e_dataset", SCRIPT)
    module = importlib.util.module_from_spec(spec)
    assert spec and spec.loader
    spec.loader.exec_module(module)
    return module


class CompleteE2EDatasetContractTest(unittest.TestCase):
    def test_catalog_covers_core_e2e_domains_with_dependencies(self):
        module = load_module()
        specs = module.build_dataset()
        slugs = {spec.slug for spec in specs}

        required = {
            "client_primary",
            "contact_primary",
            "deal_primary",
            "quote_primary",
            "project_primary",
            "time_billable",
            "invoice_income",
            "bank_operating",
            "payment_income",
            "employee_engineer",
            "salary_engineer",
            "asset_server",
            "asset_data",
            "data_asset_meta",
            "rd_cost_ai",
            "vendor_cloud",
            "contract_cloud",
            "license_security",
            "rent_office",
            "utility_power",
            "cloud_provider",
            "cloud_resource",
            "cloud_bill",
            "software_subscription",
            "domain_subscription",
            "reminder_contract",
            "agent_file_attachment",
        }
        self.assertTrue(required.issubset(slugs))

        by_slug = {spec.slug: spec for spec in specs}
        self.assertIn("client_primary", by_slug["contact_primary"].depends_on)
        self.assertIn("client_primary", by_slug["deal_primary"].depends_on)
        self.assertIn("deal_primary", by_slug["quote_primary"].depends_on)
        self.assertIn("project_primary", by_slug["time_billable"].depends_on)
        self.assertIn("employee_engineer", by_slug["salary_engineer"].depends_on)
        self.assertIn("asset_data", by_slug["data_asset_meta"].depends_on)
        self.assertIn("cloud_provider", by_slug["cloud_resource"].depends_on)

    def test_catalog_covers_all_user_facing_data_surfaces(self):
        module = load_module()
        specs = module.build_dataset()
        slugs = {spec.slug for spec in specs}

        required = {
            "asset_category_rd",
            "branch_hangzhou",
            "kpi_metric_mrr",
            "kpi_config_mrr",
            "kpi_record_mrr",
            "tax_record_income_q2",
            "application_rd_subsidy",
            "application_material_business_plan",
            "material_library_business_plan",
            "material_template_rd_subsidy",
            "material_version_business_plan",
            "quote_item_platform",
            "recurring_invoice_support",
            "invoice_reminder_prepayment",
            "tax_jurisdiction_cn",
            "tax_deadline_cn_vat",
            "profile_ops",
            "user_role_owner",
            "ai_configuration_default",
            "email_configuration_default",
            "email_log_invoice_reminder",
            "exchange_rate_usd_cny",
            "error_log_sample",
            "bank_transaction_income",
            "api_key_agent",
            "webhook_accounting",
        }
        self.assertTrue(required.issubset(slugs), required - slugs)

        covered_endpoints = {spec.endpoint for spec in specs}
        required_endpoints = {
            "/api/v1/branches",
            "/api/v1/kpi",
            "/api/v1/kpi_configs",
            "/api/v1/kpi_records",
            "/api/v1/tax",
            "/api/v1/categories",
            "/api/v1/applications",
            "/api/v1/application_materials",
            "/api/v1/material_library",
            "/api/v1/material_templates",
            "/api/v1/material_versions",
            "/api/v1/quote_items",
            "/api/v1/recurring_invoices",
            "/api/v1/invoice_reminders",
            "/api/v1/tax_jurisdictions",
            "/api/v1/tax_deadlines",
            "/api/v1/profiles",
            "/api/v1/user_roles",
            "/api/v1/ai_configurations",
            "/api/v1/email_configurations",
            "/api/v1/email/send",
            "/api/v1/exchange_rates",
            "/api/v1/error_logs",
            "/api/bank/transactions",
            "/api/api-keys",
            "/api/v1/webhooks",
        }
        self.assertTrue(required_endpoints.issubset(covered_endpoints), required_endpoints - covered_endpoints)

        by_slug = {spec.slug: spec for spec in specs}
        self.assertIn("kpi_config_mrr", by_slug["kpi_record_mrr"].depends_on)
        self.assertIn("project_primary", by_slug["application_rd_subsidy"].depends_on)
        self.assertIn("application_rd_subsidy", by_slug["application_material_business_plan"].depends_on)
        self.assertIn("material_library_business_plan", by_slug["material_version_business_plan"].depends_on)
        self.assertIn("quote_primary", by_slug["quote_item_platform"].depends_on)
        self.assertIn("invoice_income", by_slug["invoice_reminder_prepayment"].depends_on)
        self.assertIn("tax_jurisdiction_cn", by_slug["tax_deadline_cn_vat"].depends_on)
        self.assertIn("bank_operating", by_slug["bank_transaction_income"].depends_on)

    def test_default_config_is_non_destructive(self):
        module = load_module()
        config = module.SeedConfig(base_url="http://127.0.0.1:8987", session="session")
        self.assertFalse(config.refresh_existing)
        self.assertFalse(config.cleanup_first)

    def test_dataset_is_topologically_ordered(self):
        module = load_module()
        seen = {"company", "current_user"}
        for spec in module.build_dataset():
            missing = [dep for dep in spec.depends_on if dep not in seen]
            self.assertEqual(missing, [], f"{spec.slug} is listed before dependencies")
            seen.add(spec.slug)

    def test_company_info_payload_uses_table_columns(self):
        module = load_module()
        spec = next(spec for spec in module.build_dataset() if spec.slug == "company_info_profile")
        self.assertIn("company_address", spec.payload)
        self.assertIn("contact_phone", spec.payload)
        self.assertIn("contact_email", spec.payload)
        self.assertNotIn("address", spec.payload)
        self.assertNotIn("phone", spec.payload)
        self.assertNotIn("email", spec.payload)
        self.assertNotIn("industry", spec.payload)

    def test_milestone_payload_uses_current_api_contract(self):
        module = load_module()
        spec = next(spec for spec in module.build_dataset() if spec.slug == "milestone_alpha")
        self.assertIn("title", spec.payload)
        self.assertIn("milestone_date", spec.payload)
        self.assertNotIn("name", spec.payload)
        self.assertNotIn("due_date", spec.payload)
        self.assertNotIn("status", spec.payload)
        self.assertNotIn("progress", spec.payload)

    def test_vendor_payload_uses_contact_email_column(self):
        module = load_module()
        spec = next(spec for spec in module.build_dataset() if spec.slug == "vendor_cloud")
        self.assertIn("contact_email", spec.payload)
        self.assertNotIn("email", spec.payload)

    def test_utility_payload_uses_notes_and_rent_reference(self):
        module = load_module()
        spec = next(spec for spec in module.build_dataset() if spec.slug == "utility_power")
        self.assertIn("notes", spec.payload)
        self.assertIn("rent_record_id", spec.payload)
        self.assertNotIn("description", spec.payload)

    def test_branch_payload_uses_numeric_registered_capital(self):
        module = load_module()
        spec = next(spec for spec in module.build_dataset() if spec.slug == "branch_hangzhou")
        self.assertIsInstance(spec.payload["registered_capital"], (int, float))

    def test_render_value_preserves_non_placeholder_json_strings(self):
        module = load_module()
        raw = '{"endpoint":"/api/v1/email/send","status":200}'
        self.assertEqual(module.render_value(raw, {"invoice_income": "inv-1"}), raw)
        self.assertEqual(module.render_value("{invoice_income}", {"invoice_income": "inv-1"}), "inv-1")

    def test_non_put_resources_are_not_refreshed(self):
        module = load_module()
        by_slug = {spec.slug: spec for spec in module.build_dataset()}
        for slug in {
            "agent_file_attachment",
            "bank_transaction_income",
            "email_log_invoice_reminder",
            "error_log_sample",
            "api_key_agent",
            "webhook_accounting",
        }:
            self.assertFalse(by_slug[slug].refresh_supported, slug)


if __name__ == "__main__":
    unittest.main()
