#!/usr/bin/env python3
"""Seed a reusable, company-shaped E2E dataset for OPC.

The dataset models the test tenant as "语数科技". It is intentionally
idempotent and non-destructive by default: existing matching rows are reused,
missing rows are created, and a manifest of stable IDs is written for later E2E
tests.
"""

import argparse
import base64
import dataclasses
import json
import os
import re
import sqlite3
import sys
import urllib.parse
import uuid
from typing import Any, Optional

import requests


DEFAULT_COMPANY_NAME = "语数科技"
DEFAULT_EMAIL = "e2e@yushu-tech.example"
DEFAULT_USER_NAME = "语数科技 E2E 测试员"
PLACEHOLDER_RE = re.compile(r"\{([A-Za-z0-9_]+)\}")


@dataclasses.dataclass(frozen=True)
class RecordSpec:
    slug: str
    endpoint: str
    payload: dict[str, Any]
    match_field: str
    match_value: str
    depends_on: tuple[str, ...] = ()
    list_endpoint: Optional[str] = None
    result_path: tuple[str, ...] = ("id",)
    refresh_supported: bool = True


@dataclasses.dataclass
class SeedConfig:
    base_url: str
    session: str
    company_name: str = DEFAULT_COMPANY_NAME
    refresh_existing: bool = False
    cleanup_first: bool = False
    manifest_path: Optional[str] = None
    timeout: int = 15


class SeedClient:
    def __init__(self, config: SeedConfig) -> None:
        self.config = config
        self.base_url = config.base_url.rstrip("/")
        self.session = requests.Session()
        self.session.headers.update(
            {
                "Authorization": f"Bearer {config.session}",
                "Content-Type": "application/json",
            }
        )

    def url(self, endpoint: str) -> str:
        if endpoint.startswith("http://") or endpoint.startswith("https://"):
            return endpoint
        return f"{self.base_url}{endpoint}"

    def get(self, endpoint: str) -> requests.Response:
        return self.session.get(self.url(endpoint), timeout=self.config.timeout)

    def post(self, endpoint: str, payload: dict[str, Any]) -> requests.Response:
        return self.session.post(self.url(endpoint), json=payload, timeout=self.config.timeout)

    def put(self, endpoint: str, payload: dict[str, Any]) -> requests.Response:
        return self.session.put(self.url(endpoint), json=payload, timeout=self.config.timeout)


def build_dataset() -> list[RecordSpec]:
    """Return the canonical 语数科技 E2E data catalog."""
    return [
        RecordSpec(
            slug="company_info_profile",
            endpoint="/api/v1/company_info",
            match_field="company_name",
            match_value=DEFAULT_COMPANY_NAME,
            payload={
                "company_name": DEFAULT_COMPANY_NAME,
                "legal_representative": "沈语辰",
                "unified_social_credit_code": "91330110YS20260529X",
                "taxpayer_type": "general",
                "vat_rate": 13,
                "registered_capital": 5000000,
                "business_scope": "AI 数据智能、企业软件、数据资产治理、智能运营系统研发与技术服务",
                "company_address": "杭州市余杭区未来科技城语数路 88 号",
                "contact_phone": "0571-8800-0529",
                "contact_email": "ops@yushu-tech.example",
            },
        ),
        RecordSpec(
            slug="social_insurance_shenzhen",
            endpoint="/api/v1/social_insurance_config",
            match_field="city",
            match_value="深圳",
            payload={
                "city": "深圳",
                "social_security_base_min": 2360.0,
                "social_security_base_max": 33891.0,
                "pension_personal_rate": 0.08,
                "pension_company_rate": 0.14,
                "medical_personal_rate": 0.02,
                "medical_company_rate": 0.052,
                "housing_fund_personal_rate": 0.07,
                "housing_fund_company_rate": 0.07,
            },
        ),
        RecordSpec(
            slug="asset_category_rd",
            endpoint="/api/v1/categories",
            match_field="name",
            match_value="数据与研发基础设施",
            payload={
                "name": "数据与研发基础设施",
                "category_type": "both",
                "description": "语数科技用于数据资产、研发设备和云资源归集的资产分类。",
            },
        ),
        RecordSpec(
            slug="branch_hangzhou",
            endpoint="/api/v1/branches",
            match_field="branch_code",
            match_value="YS-HZ-RD",
            payload={
                "branch_name": "语数科技杭州研发中心",
                "branch_code": "YS-HZ-RD",
                "branch_type": "headquarters",
                "legal_representative": "沈语辰",
                "unified_social_credit_code": "91330110YS20260529X",
                "registered_capital": 5000000,
                "contact_person": "周敏",
                "contact_phone": "0571-8800-0529",
                "contact_email": "ops@yushu-tech.example",
                "branch_address": "杭州市余杭区未来科技城语数路 88 号 7F",
                "business_scope": "AI 数据智能、企业软件、数据资产治理和智能运营系统研发",
                "registration_date": "2026-01-05",
                "employee_count": 18,
                "notes": "语数科技 E2E 主测试分支机构。",
            },
        ),
        RecordSpec(
            slug="kpi_metric_mrr",
            endpoint="/api/v1/kpi",
            match_field="name",
            match_value="语数科技月度经常性收入",
            payload={
                "name": "语数科技月度经常性收入",
                "category": "financial",
                "value": 512000,
                "target": 800000,
                "unit": "元",
                "period": "2026-05",
                "description": "用于 KPI Dashboard 的收入指标样本。",
            },
        ),
        RecordSpec(
            slug="kpi_config_mrr",
            endpoint="/api/v1/kpi_configs",
            match_field="name",
            match_value="月度经常性收入 MRR",
            payload={
                "name": "月度经常性收入 MRR",
                "category": "financial",
                "description": "衡量语数科技订阅和运维收入的月度指标。",
                "unit": "元",
                "target_value": 800000,
                "warning_threshold": 600000,
                "danger_threshold": 400000,
                "is_active": 1,
            },
        ),
        RecordSpec(
            slug="kpi_record_mrr",
            endpoint="/api/v1/kpi_records",
            depends_on=("kpi_config_mrr",),
            match_field="notes",
            match_value="语数科技 2026-05 MRR 指标样本",
            payload={
                "kpi_id": "{kpi_config_mrr}",
                "value": 512000,
                "period": "2026-05",
                "notes": "语数科技 2026-05 MRR 指标样本",
            },
        ),
        RecordSpec(
            slug="tax_record_income_q2",
            endpoint="/api/v1/tax",
            match_field="notes",
            match_value="语数科技 2026-Q2 企业所得税测算样本",
            payload={
                "tax_type": "企业所得税",
                "period": "2026-Q2",
                "declaration_type": "quarterly",
                "declaration_period": "2026-Q2",
                "period_start": "2026-04-01",
                "period_end": "2026-06-30",
                "taxable_amount": 680000,
                "tax_amount": 170000,
                "declared_amount": 0,
                "status": "draft",
                "due_date": "2026-07-15",
                "notes": "语数科技 2026-Q2 企业所得税测算样本",
            },
        ),
        RecordSpec(
            slug="client_primary",
            endpoint="/api/v1/clients",
            match_field="name",
            match_value="青岚制造集团",
            payload={
                "name": "青岚制造集团",
                "short_name": "青岚制造",
                "client_type": "active",
                "industry": "高端装备制造",
                "website": "https://qinglan.example",
                "email": "digital@qinglan.example",
                "phone": "021-6100-8899",
                "address": "上海市浦东新区张江高科园区",
                "city": "上海",
                "country": "CN",
                "source": "referral",
                "annual_revenue": 860000000,
                "employee_count": 1800,
                "tags": json.dumps(["enterprise", "manufacturing", "data-platform"], ensure_ascii=False),
                "notes": "语数科技核心制造业数字化客户，用于端到端销售、交付、计费链路。",
            },
        ),
        RecordSpec(
            slug="contact_primary",
            endpoint="/api/v1/contacts",
            depends_on=("client_primary",),
            match_field="email",
            match_value="lin.wei@qinglan.example",
            payload={
                "client_id": "{client_primary}",
                "name": "林微",
                "title": "数字化转型负责人",
                "email": "lin.wei@qinglan.example",
                "phone": "021-6100-8801",
                "mobile": "00000005291",
                "department": "信息化中心",
                "is_primary": 1,
                "notes": "负责语数智能运营平台一期项目。",
            },
        ),
        RecordSpec(
            slug="interaction_discovery",
            endpoint="/api/v1/interactions",
            depends_on=("client_primary", "contact_primary"),
            match_field="subject",
            match_value="语数智能运营平台需求澄清会",
            payload={
                "client_id": "{client_primary}",
                "contact_id": "{contact_primary}",
                "interaction_type": "meeting",
                "subject": "语数智能运营平台需求澄清会",
                "description": "确认数据资产台账、研发费用归集、云成本看板和合同审查范围。",
                "interaction_date": "2026-05-20",
                "duration_minutes": 95,
                "outcome": "positive",
                "next_action": "提交正式报价与实施计划",
                "next_action_date": "2026-05-27",
            },
        ),
        RecordSpec(
            slug="deal_primary",
            endpoint="/api/v1/deals",
            depends_on=("client_primary", "contact_primary"),
            match_field="title",
            match_value="青岚制造智能运营数据中台一期",
            payload={
                "client_id": "{client_primary}",
                "contact_id": "{contact_primary}",
                "title": "青岚制造智能运营数据中台一期",
                "description": "语数科技提供 ERP 经营分析、数据资产治理、AI 合同审查和财税看板。",
                "stage": "won",
                "amount": 1280000,
                "currency": "CNY",
                "probability": 100,
                "expected_close_date": "2026-06-10",
                "actual_close_date": "2026-05-28",
                "source": "partner",
                "assigned_to": "沈语辰",
                "tags": json.dumps(["won", "strategic"], ensure_ascii=False),
            },
        ),
        RecordSpec(
            slug="quote_primary",
            endpoint="/api/v1/quotes",
            depends_on=("client_primary", "contact_primary", "deal_primary"),
            match_field="quote_number",
            match_value="YS-QT-2026-0001",
            payload={
                "client_id": "{client_primary}",
                "contact_id": "{contact_primary}",
                "deal_id": "{deal_primary}",
                "quote_number": "YS-QT-2026-0001",
                "title": "语数智能运营数据中台一期报价",
                "status": "accepted",
                "subtotal": 1132743.36,
                "tax_rate": 0.13,
                "tax_amount": 147256.64,
                "discount": 0,
                "total": 1280000,
                "currency": "CNY",
                "valid_until": "2026-06-30",
                "terms": "40% 预付款，40% 验收款，20% 质保款。",
            },
        ),
        RecordSpec(
            slug="quote_item_platform",
            endpoint="/api/v1/quote_items",
            depends_on=("quote_primary",),
            match_field="description",
            match_value="智能运营平台一期实施服务",
            payload={
                "quote_id": "{quote_primary}",
                "description": "智能运营平台一期实施服务",
                "quantity": 1,
                "unit": "项",
                "unit_price": 980000,
                "discount_percent": 0,
                "amount": 980000,
                "sort_order": 1,
            },
        ),
        RecordSpec(
            slug="recurring_invoice_support",
            endpoint="/api/v1/recurring_invoices",
            depends_on=("client_primary",),
            match_field="title",
            match_value="青岚制造年度运维支持月费",
            payload={
                "client_id": "{client_primary}",
                "title": "青岚制造年度运维支持月费",
                "description": "青岚制造一期上线后的月度运维、模型监控和报表支持。",
                "amount": 38000,
                "currency": "CNY",
                "tax_rate": 0.13,
                "frequency": "monthly",
                "start_date": "2026-06-01",
                "next_invoice_date": "2026-06-01",
                "day_of_month": 1,
                "auto_send": 0,
                "payment_terms_days": 15,
                "status": "active",
                "notes": "语数科技 E2E 周期发票样本。",
            },
        ),
        RecordSpec(
            slug="project_primary",
            endpoint="/api/v1/projects",
            match_field="name",
            match_value="语数智能运营平台研发项目",
            payload={
                "name": "语数智能运营平台研发项目",
                "description": "围绕企业经营数据、文档融合、AI 审查和运营看板的核心研发项目。",
                "project_type": "rd",
                "leader_name": "沈语辰",
                "status": "active",
                "priority": "high",
                "budget": 1850000,
                "spent": 642000,
                "start_date": "2026-04-01",
                "end_date": "2026-12-31",
                "client": "青岚制造集团",
                "progress": 38,
                "team_members": 8,
                "goals": "形成可复用的智能 ERP 经营数据资产底座。",
            },
        ),
        RecordSpec(
            slug="milestone_alpha",
            endpoint="/api/v1/milestones",
            depends_on=("project_primary",),
            match_field="title",
            match_value="Alpha 数据资产台账可用",
            payload={
                "project_id": "{project_primary}",
                "title": "Alpha 数据资产台账可用",
                "description": "完成数据资产、文件附件、研发费用归集和云账单联动。",
                "milestone_type": "delivery",
                "importance": "high",
                "milestone_date": "2026-06-30",
                "notes": "E2E 测试数据中的项目阶段节点。",
            },
        ),
        RecordSpec(
            slug="application_rd_subsidy",
            endpoint="/api/v1/applications",
            depends_on=("project_primary",),
            match_field="application_no",
            match_value="YS-APP-2026-HZ-001",
            payload={
                "application_no": "YS-APP-2026-HZ-001",
                "title": "杭州市人工智能研发补助申报",
                "type": "government_subsidy",
                "category": "杭州市级",
                "deadline": "2026-07-31",
                "status": "in_progress",
                "progress": 65,
                "project_id": "{project_primary}",
                "notes": "围绕语数智能运营平台研发投入申请地方补助。",
            },
        ),
        RecordSpec(
            slug="material_library_business_plan",
            endpoint="/api/v1/material_library",
            depends_on=("application_rd_subsidy",),
            match_field="name",
            match_value="语数科技研发项目商业计划书",
            payload={
                "name": "语数科技研发项目商业计划书",
                "category": "项目申报",
                "upload_date": "2026-05-26",
                "file_size": "2.4 MB",
                "file_path": "/e2e/materials/yushu-business-plan.pdf",
                "used_count": 1,
                "source_type": "application",
                "source_id": "{application_rd_subsidy}",
            },
        ),
        RecordSpec(
            slug="material_template_rd_subsidy",
            endpoint="/api/v1/material_templates",
            match_field="template_name",
            match_value="AI 研发补助材料清单",
            payload={
                "template_type": "rd_subsidy",
                "template_name": "AI 研发补助材料清单",
                "template_category": "杭州市级",
                "material_name": "商业计划书",
                "required": 1,
                "description": "用于杭州人工智能研发补助申报的标准材料模板。",
                "display_order": 1,
            },
        ),
        RecordSpec(
            slug="application_material_business_plan",
            endpoint="/api/v1/application_materials",
            depends_on=("application_rd_subsidy", "material_library_business_plan"),
            match_field="name",
            match_value="语数科技研发项目商业计划书",
            payload={
                "application_id": "{application_rd_subsidy}",
                "name": "语数科技研发项目商业计划书",
                "required": 1,
                "status": "completed",
                "description": "说明产品定位、研发投入、数据资产和商业化路径。",
                "upload_date": "2026-05-26",
                "file_size": "2.4 MB",
                "file_path": "/e2e/materials/yushu-business-plan.pdf",
                "can_reuse": 1,
                "reuse_source": "{material_library_business_plan}",
            },
        ),
        RecordSpec(
            slug="material_version_business_plan",
            endpoint="/api/v1/material_versions",
            depends_on=("material_library_business_plan",),
            match_field="file_name",
            match_value="yushu-business-plan-v1.pdf",
            payload={
                "material_id": "{material_library_business_plan}",
                "version_number": 1,
                "file_path": "/e2e/materials/yushu-business-plan-v1.pdf",
                "file_name": "yushu-business-plan-v1.pdf",
                "file_size": "2.4 MB",
                "description": "语数科技研发补助申报商业计划书首版。",
                "change_notes": "E2E 初始版本。",
                "created_by": "沈语辰",
                "is_current": 1,
            },
        ),
        RecordSpec(
            slug="employee_engineer",
            endpoint="/api/v1/employees",
            match_field="email",
            match_value="chen.rui@yushu-tech.example",
            payload={
                "name": "陈睿",
                "full_name": "陈睿",
                "employee_number": "YS-ENG-001",
                "email": "chen.rui@yushu-tech.example",
                "phone": "00000005292",
                "department": "研发部",
                "position": "后端平台工程师",
                "hire_date": "2025-11-03",
                "status": "active",
                "employment_type": "full_time",
                "base_salary": 32000,
                "social_security_base": 30000,
                "housing_fund_base": 30000,
                "bank_name": "招商银行杭州分行",
                "bank_account": "0004830000000529001",
                "notes": "负责 Rust 后端、数据资产和 AI 网关。",
            },
        ),
        RecordSpec(
            slug="employee_accountant",
            endpoint="/api/v1/employees",
            match_field="email",
            match_value="zhou.min@yushu-tech.example",
            payload={
                "name": "周敏",
                "full_name": "周敏",
                "employee_number": "YS-FIN-001",
                "email": "zhou.min@yushu-tech.example",
                "department": "财务与运营",
                "position": "财务运营经理",
                "hire_date": "2026-01-15",
                "status": "active",
                "employment_type": "full_time",
                "base_salary": 26000,
                "social_security_base": 26000,
                "housing_fund_base": 26000,
            },
        ),
        RecordSpec(
            slug="time_billable",
            endpoint="/api/v1/time_entries",
            depends_on=("project_primary", "client_primary", "employee_engineer"),
            match_field="description",
            match_value="青岚制造数据资产模型设计",
            payload={
                "project_id": "{project_primary}",
                "client_id": "{client_primary}",
                "employee_id": "{employee_engineer}",
                "description": "青岚制造数据资产模型设计",
                "date": "2026-05-24",
                "duration_minutes": 360,
                "is_billable": 1,
                "hourly_rate": 950,
                "amount": 5700,
                "status": "logged",
                "tags": json.dumps(["rd", "client-delivery"], ensure_ascii=False),
            },
        ),
        RecordSpec(
            slug="salary_engineer",
            endpoint="/api/v1/salary",
            depends_on=("employee_engineer",),
            match_field="notes",
            match_value="语数科技 E2E 研发工资样本",
            payload={
                "employee_id": "{employee_engineer}",
                "employee_name": "陈睿",
                "department": "研发部",
                "position": "后端平台工程师",
                "period": "2026-05",
                "base_salary": 32000,
                "bonus": 5000,
                "deductions": 1200,
                "social_security": 3100,
                "social_security_company": 5200,
                "housing_fund": 2100,
                "housing_fund_company": 2100,
                "net_salary": 30600,
                "status": "approved",
                "notes": "语数科技 E2E 研发工资样本",
            },
        ),
        RecordSpec(
            slug="budget_rd",
            endpoint="/api/v1/budgets",
            depends_on=("project_primary",),
            match_field="notes",
            match_value="语数科技 E2E 研发预算样本",
            payload={
                "category": "研发费用",
                "budget_type": "monthly",
                "period": "2026-05",
                "budget_amount": 420000,
                "actual_amount": 286000,
                "status": "active",
                "project_id": "{project_primary}",
                "notes": "语数科技 E2E 研发预算样本",
            },
        ),
        RecordSpec(
            slug="bank_operating",
            endpoint="/api/bank/accounts",
            match_field="account_number",
            match_value="0004830000000529999",
            payload={
                "bank_name": "招商银行杭州未来科技城支行",
                "account_name": "语数科技基本户",
                "account_number": "0004830000000529999",
                "account_type": "checking",
                "balance": 3860000,
                "currency": "CNY",
                "status": "active",
                "notes": "语数科技 E2E 运营资金账户。",
            },
        ),
        RecordSpec(
            slug="vendor_cloud",
            endpoint="/api/v1/vendors",
            match_field="name",
            match_value="灵云计算服务有限公司",
            payload={
                "name": "灵云计算服务有限公司",
                "vendor_type": "cloud",
                "contact_person": "陆远",
                "contact_phone": "00000005293",
                "contact_email": "support@lingcloud.example",
                "status": "active",
                "notes": "语数科技云基础设施主供应商。",
            },
        ),
        RecordSpec(
            slug="contract_cloud",
            endpoint="/api/v1/contracts",
            depends_on=("project_primary", "vendor_cloud"),
            match_field="contract_number",
            match_value="YS-CT-2026-0001",
            payload={
                "contract_name": "青岚制造智能运营数据中台一期服务合同",
                "contract_number": "YS-CT-2026-0001",
                "contract_type": "service",
                "counterparty": "青岚制造集团",
                "party_a": DEFAULT_COMPANY_NAME,
                "party_b": "青岚制造集团",
                "amount": 1280000,
                "start_date": "2026-05-29",
                "end_date": "2027-05-28",
                "status": "active",
                "project_id": "{project_primary}",
                "content": "语数科技提供智能 ERP、数据资产治理、AI 审查和运维支持。",
                "counterparty_type": "client",
            },
        ),
        RecordSpec(
            slug="invoice_income",
            endpoint="/api/v1/invoices",
            depends_on=("contract_cloud",),
            match_field="invoice_number",
            match_value="YS-INV-2026-0001",
            payload={
                "invoice_number": "YS-INV-2026-0001",
                "invoice_type": "output",
                "amount": 512000,
                "tax_amount": 58902.65,
                "status": "approved",
                "invoice_date": "2026-05-29",
                "due_date": "2026-06-12",
                "supplier": "青岚制造集团",
                "description": "语数智能运营数据中台一期 40% 预付款发票。",
                "contract_id": "{contract_cloud}",
            },
        ),
        RecordSpec(
            slug="payment_income",
            endpoint="/api/v1/payments",
            depends_on=("invoice_income", "bank_operating"),
            match_field="payment_no",
            match_value="YS-PAY-2026-0001",
            payload={
                "payment_no": "YS-PAY-2026-0001",
                "payment_type": "收入",
                "payment_method": "bank_transfer",
                "amount": 512000,
                "currency": "CNY",
                "category": "项目预付款",
                "status": "completed",
                "payer": "青岚制造集团",
                "payee": DEFAULT_COMPANY_NAME,
                "bank_account_id": "{bank_operating}",
                "invoice_id": "{invoice_income}",
                "description": "青岚制造一期项目预付款到账。",
                "payment_date": "2026-05-30",
            },
        ),
        RecordSpec(
            slug="bank_transaction_income",
            endpoint="/api/bank/transactions",
            list_endpoint="/api/bank/transactions",
            depends_on=("bank_operating",),
            match_field="description",
            match_value="青岚制造一期项目预付款银行流水",
            refresh_supported=False,
            payload={
                "account_id": "{bank_operating}",
                "transaction_type": "deposit",
                "amount": 512000,
                "counterparty": "青岚制造集团",
                "description": "青岚制造一期项目预付款银行流水",
                "transaction_date": "2026-05-30",
            },
        ),
        RecordSpec(
            slug="invoice_reminder_prepayment",
            endpoint="/api/v1/invoice_reminders",
            depends_on=("invoice_income", "client_primary"),
            match_field="subject",
            match_value="语数科技发票付款提醒：YS-INV-2026-0001",
            payload={
                "invoice_id": "{invoice_income}",
                "client_id": "{client_primary}",
                "reminder_type": "before_due",
                "scheduled_date": "2026-06-10",
                "status": "pending",
                "subject": "语数科技发票付款提醒：YS-INV-2026-0001",
                "message": "青岚制造一期项目预付款发票即将到期，请确认付款安排。",
                "channel": "email",
            },
        ),
        RecordSpec(
            slug="asset_server",
            endpoint="/api/assets",
            depends_on=("asset_category_rd",),
            match_field="name",
            match_value="语数科技推理与文档融合服务器",
            payload={
                "name": "语数科技推理与文档融合服务器",
                "asset_type": "physical",
                "status": "in_use",
                "serial_number": "YS-SRV-2026-0001",
                "category_id": "{asset_category_rd}",
                "location": "杭州未来科技城机房 A-03",
                "purchase_date": "2026-04-18",
                "purchase_price": 168000,
                "current_value": 156200,
                "useful_life": 60,
                "salvage_rate": 0.05,
                "depreciation_method": "straight_line",
                "monthly_depreciation": 2660,
                "accumulated_depreciation": 5320,
                "description": "用于 DaaS 文档解析、向量化和 AI 审查任务。",
            },
        ),
        RecordSpec(
            slug="asset_data",
            endpoint="/api/assets",
            depends_on=("asset_category_rd",),
            match_field="name",
            match_value="语数企业经营语义数据集",
            payload={
                "name": "语数企业经营语义数据集",
                "asset_type": "data",
                "status": "in_use",
                "category_id": "{asset_category_rd}",
                "purchase_price": 0,
                "current_value": 980000,
                "description": "包含匿名化合同条款、财务科目、项目工时和云成本标签的经营语义数据集。",
            },
        ),
        RecordSpec(
            slug="asset_record_depreciation",
            endpoint="/api/v1/asset_records",
            depends_on=("asset_server",),
            match_field="description",
            match_value="2026-05 服务器折旧归集",
            payload={
                "asset_id": "{asset_server}",
                "record_type": "depreciation",
                "description": "2026-05 服务器折旧归集",
                "amount": 2660,
                "record_date": "2026-05-31",
            },
        ),
        RecordSpec(
            slug="data_asset_meta",
            endpoint="/api/v1/data_asset_meta",
            depends_on=("asset_data",),
            match_field="asset_id",
            match_value="{asset_data}",
            payload={
                "asset_id": "{asset_data}",
                "data_type": "dataset",
                "tech_stack": "Rust, SQLite, Parquet, Embedding Pipeline",
                "version": "2026.05",
                "license_type": "自研",
                "data_volume": "约 120GB 结构化与半结构化经营样本",
                "data_format": "SQLite/JSONL/Parquet",
                "access_url": "s3://yushu-e2e-dataset/business-semantic-2026-05",
            },
        ),
        RecordSpec(
            slug="rd_cost_ai",
            endpoint="/api/v1/rd_costs",
            depends_on=("project_primary", "employee_engineer", "asset_server", "invoice_income"),
            match_field="description",
            match_value="陈睿 2026-05 文档融合研发工时归集",
            payload={
                "project_id": "{project_primary}",
                "cost_type": "personnel",
                "amount": 37000,
                "description": "陈睿 2026-05 文档融合研发工时归集",
                "cost_date": "2026-05-31",
                "employee_id": "{employee_engineer}",
                "asset_id": "{asset_server}",
                "invoice_id": "{invoice_income}",
            },
        ),
        RecordSpec(
            slug="license_security",
            endpoint="/api/v1/licenses",
            match_field="license_number",
            match_value="YS-ICP-2026-E2E",
            payload={
                "license_name": "语数科技增值电信业务备案样本",
                "license_type": "icp",
                "license_number": "YS-ICP-2026-E2E",
                "issuing_authority": "浙江省通信管理局",
                "issue_date": "2026-03-01",
                "expiry_date": "2031-03-01",
                "status": "valid",
                "notes": "E2E 合规证照样本。",
            },
        ),
        RecordSpec(
            slug="rent_office",
            endpoint="/api/v1/rent",
            depends_on=("contract_cloud",),
            match_field="location_name",
            match_value="语数科技杭州研发办公室",
            payload={
                "location_name": "语数科技杭州研发办公室",
                "property_type": "office",
                "address": "杭州市余杭区未来科技城语数路 88 号 7F",
                "area": 680,
                "monthly_rent": 91800,
                "deposit": 183600,
                "contract_start_date": "2026-01-01",
                "contract_end_date": "2028-12-31",
                "next_payment_date": "2026-06-01",
                "payment_cycle": "monthly",
                "landlord": "未来科创园运营有限公司",
                "landlord_contact": "0571-8899-0529",
                "contract_id": "{contract_cloud}",
                "status": "active",
            },
        ),
        RecordSpec(
            slug="utility_power",
            endpoint="/api/v1/utilities",
            depends_on=("rent_office",),
            match_field="notes",
            match_value="语数科技 2026-05 机房电费",
            payload={
                "utility_type": "电费",
                "amount": 12800,
                "location_name": "语数科技杭州研发办公室",
                "period": "2026-05",
                "status": "unpaid",
                "rent_record_id": "{rent_office}",
                "notes": "语数科技 2026-05 机房电费",
            },
        ),
        RecordSpec(
            slug="tax_declaration_vat",
            endpoint="/api/v1/tax_declarations",
            depends_on=("invoice_income", "payment_income"),
            match_field="notes",
            match_value="语数科技 2026-05 增值税申报样本",
            payload={
                "tax_type": "增值税",
                "period": "2026-05",
                "declaration_type": "monthly",
                "taxable_amount": 453097.35,
                "tax_amount": 58902.65,
                "declared_amount": 58902.65,
                "status": "pending",
                "due_date": "2026-06-15",
                "invoice_id": "{invoice_income}",
                "payment_id": "{payment_income}",
                "notes": "语数科技 2026-05 增值税申报样本",
            },
        ),
        RecordSpec(
            slug="tax_jurisdiction_cn",
            endpoint="/api/v1/tax_jurisdictions",
            match_field="code",
            match_value="CN",
            payload={
                "code": "CN",
                "name": "China",
                "name_local": "中国",
                "country_code": "CN",
                "jurisdiction_type": "country",
                "currency": "CNY",
                "fiscal_year_start": "01-01",
                "corporate_tax_rate": 25.0,
                "personal_tax_rate": 45.0,
                "vat_rate": 13.0,
                "notes": "语数科技中国大陆税务主体。",
                "is_active": 1,
            },
        ),
        RecordSpec(
            slug="tax_deadline_cn_vat",
            endpoint="/api/v1/tax_deadlines",
            depends_on=("tax_jurisdiction_cn",),
            match_field="title",
            match_value="语数科技增值税月度申报截止",
            payload={
                "jurisdiction_id": "{tax_jurisdiction_cn}",
                "jurisdiction_code": "CN",
                "tax_type": "vat",
                "title": "语数科技增值税月度申报截止",
                "title_local": "增值税月度申报",
                "description": "每月 15 日前完成上月增值税申报。",
                "deadline_month": 6,
                "deadline_day": 15,
                "recurrence": "monthly",
                "applies_to": "all",
                "penalty_info": "逾期可能产生滞纳金和信用扣分。",
                "filing_url": "https://etax.chinatax.gov.cn",
                "is_system": 0,
                "is_active": 1,
                "notes": "语数科技 E2E 税务日历样本。",
            },
        ),
        RecordSpec(
            slug="cloud_provider",
            endpoint="/api/v1/cloud_providers",
            match_field="provider_name",
            match_value="灵云 Cloud",
            payload={
                "provider_name": "灵云 Cloud",
                "provider_type": "public_cloud",
                "account_name": "yushu-prod-main",
                "account_id": "ys-cloud-prod-001",
                "notes": "语数科技生产与测试云资源统一供应商。",
            },
        ),
        RecordSpec(
            slug="cloud_resource",
            endpoint="/api/v1/cloud_resources",
            depends_on=("cloud_provider",),
            match_field="resource_name",
            match_value="ys-doc-fusion-prod-01",
            payload={
                "provider_id": "{cloud_provider}",
                "resource_type": "gpu-instance",
                "resource_name": "ys-doc-fusion-prod-01",
                "specification": "8 vCPU / 64GB RAM / L40S 48GB",
                "region": "cn-east-1",
                "start_date": "2026-04-20",
                "monthly_cost": 28600,
                "status": "running",
            },
        ),
        RecordSpec(
            slug="cloud_bill",
            endpoint="/api/v1/cloud_bills",
            depends_on=("cloud_provider",),
            match_field="billing_period",
            match_value="2026-05",
            payload={
                "provider_id": "{cloud_provider}",
                "billing_period": "2026-05",
                "total_amount": 64280,
                "currency": "CNY",
                "status": "pending",
                "notes": "语数科技 2026-05 云账单，包含 GPU 推理、对象存储和数据库资源。",
            },
        ),
        RecordSpec(
            slug="software_subscription",
            endpoint="/api/v1/software_subscriptions",
            match_field="software_name",
            match_value="语数科技 JetBrains 团队订阅",
            payload={
                "software_name": "语数科技 JetBrains 团队订阅",
                "vendor": "JetBrains",
                "subscription_type": "SaaS",
                "billing_cycle": "yearly",
                "start_date": "2026-01-01",
                "next_billing_date": "2027-01-01",
                "total_price": 42000,
                "status": "active",
            },
        ),
        RecordSpec(
            slug="domain_subscription",
            endpoint="/api/v1/domain_subscriptions",
            match_field="domain_name",
            match_value="yushu-tech.example",
            payload={
                "domain_name": "yushu-tech.example",
                "registrar": "Cloudflare",
                "registration_date": "2026-01-01",
                "expiry_date": "2027-01-01",
                "annual_fee": 88,
                "status": "active",
            },
        ),
        RecordSpec(
            slug="insurance_policy_cyber",
            endpoint="/api/v1/insurance_policies",
            match_field="policy_name",
            match_value="语数科技网络安全责任险",
            payload={
                "policy_name": "语数科技网络安全责任险",
                "insurance_type": "cyber",
                "provider": "中国平安",
                "premium_amount": 36000,
                "premium_frequency": "yearly",
                "coverage_amount": 10000000,
                "deductible": 20000,
                "start_date": "2026-01-01",
                "end_date": "2026-12-31",
                "renewal_date": "2026-12-01",
                "country": "CN",
                "status": "active",
            },
        ),
        RecordSpec(
            slug="pension_plan_cn",
            endpoint="/api/v1/pension_plans",
            match_field="plan_name",
            match_value="语数科技企业年金计划",
            payload={
                "plan_name": "语数科技企业年金计划",
                "plan_type": "enterprise_annuity",
                "country": "CN",
                "provider": "招商银行养老金中心",
                "contribution_employer": 120000,
                "contribution_employee": 60000,
                "contribution_frequency": "monthly",
                "status": "active",
            },
        ),
        RecordSpec(
            slug="profile_ops",
            endpoint="/api/v1/profiles",
            depends_on=("current_user",),
            match_field="email",
            match_value=DEFAULT_EMAIL,
            payload={
                "user_id": "{current_user}",
                "display_name": DEFAULT_USER_NAME,
                "email": DEFAULT_EMAIL,
                "phone": "00000005299",
                "department": "QA 与运营",
                "position": "E2E 测试员",
                "bio": "语数科技完整 E2E 数据集的测试账号档案。",
            },
        ),
        RecordSpec(
            slug="user_role_owner",
            endpoint="/api/v1/user_roles",
            depends_on=("current_user",),
            match_field="role",
            match_value="owner",
            payload={
                "user_id": "{current_user}",
                "role": "owner",
                "permissions": json.dumps(
                    ["admin", "finance", "operations", "integrations"],
                    ensure_ascii=False,
                ),
            },
        ),
        RecordSpec(
            slug="ai_configuration_default",
            endpoint="/api/v1/ai_configurations",
            match_field="provider",
            match_value="DeepSeek",
            payload={
                "provider": "DeepSeek",
                "model": "deepseek-v4-flash",
                "api_key": "w-id://workspace/example-workspace/secret/yushu-e2e-deepseek",
                "endpoint": "https://api.deepseek.com/v1/chat/completions",
                "enabled": 0,
            },
        ),
        RecordSpec(
            slug="email_configuration_default",
            endpoint="/api/v1/email_configurations",
            match_field="smtp_host",
            match_value="smtp.yushu-tech.example",
            payload={
                "smtp_host": "smtp.yushu-tech.example",
                "smtp_port": 587,
                "smtp_user": "mailer@yushu-tech.example",
                "smtp_password": "w-id://workspace/example-workspace/secret/yushu-e2e-smtp",
                "sender_name": "语数科技运营",
                "sender_email": "ops@yushu-tech.example",
                "enabled": 1,
            },
        ),
        RecordSpec(
            slug="email_log_invoice_reminder",
            endpoint="/api/v1/email/send",
            list_endpoint="/api/v1/email/logs",
            depends_on=("email_configuration_default", "invoice_income"),
            match_field="subject",
            match_value="语数科技 E2E 发票提醒邮件",
            result_path=("log_id",),
            refresh_supported=False,
            payload={
                "to": "finance@qinglan.example",
                "subject": "语数科技 E2E 发票提醒邮件",
                "template": "payment_reminder",
                "entity_type": "invoices",
                "entity_id": "{invoice_income}",
                "body": "青岚制造一期项目预付款发票已生成，请在到期日前完成付款。",
            },
        ),
        RecordSpec(
            slug="exchange_rate_usd_cny",
            endpoint="/api/v1/exchange_rates",
            match_field="from_currency",
            match_value="USD",
            payload={
                "from_currency": "USD",
                "to_currency": "CNY",
                "rate": 7.18,
                "effective_date": "2026-05-29",
                "source": "e2e",
            },
        ),
        RecordSpec(
            slug="error_log_sample",
            endpoint="/api/v1/error_logs",
            match_field="error_message",
            match_value="语数科技 E2E 前端错误样本：无害日志",
            refresh_supported=False,
            payload={
                "error_type": "e2e_seed_warning",
                "error_message": "语数科技 E2E 前端错误样本：无害日志",
                "stack_trace": "SyntheticError: generated by seed_complete_e2e_dataset",
                "component_stack": "Settings > EmailConfigDialog",
                "page_url": "http://127.0.0.1:18987/settings",
                "user_agent": "OPC E2E Seed",
                "metadata": json.dumps(
                    {"endpoint": "/api/v1/email/send", "method": "POST", "status": 200},
                    ensure_ascii=False,
                ),
            },
        ),
        RecordSpec(
            slug="api_key_agent",
            endpoint="/api/api-keys",
            list_endpoint="/api/api-keys",
            depends_on=("current_user",),
            match_field="name",
            match_value="语数科技 Agent E2E Key",
            refresh_supported=False,
            payload={
                "name": "语数科技 Agent E2E Key",
                "scopes": "read,write",
                "expires_in_days": 30,
            },
        ),
        RecordSpec(
            slug="webhook_accounting",
            endpoint="/api/v1/webhooks",
            depends_on=("invoice_income", "payment_income"),
            match_field="url",
            match_value="https://hooks.yushu-tech.example/opc/e2e",
            refresh_supported=False,
            payload={
                "url": "https://hooks.yushu-tech.example/opc/e2e",
                "events": "invoice.created,payment.received",
                "description": "语数科技 E2E 财务事件 Webhook。",
                "secret": "w-id://workspace/example-workspace/secret/yushu-e2e-webhook",
            },
        ),
        RecordSpec(
            slug="reminder_contract",
            endpoint="/api/v1/reminders",
            depends_on=("contract_cloud",),
            match_field="title",
            match_value="青岚制造一期合同回款节点",
            payload={
                "title": "青岚制造一期合同回款节点",
                "description": "检查 40% 验收款回款条件和客户验收单状态。",
                "reminder_type": "payment",
                "priority": "high",
                "due_date": "2026-07-15",
                "remind_date": "2026-07-08",
                "status": "pending",
                "entity_type": "contracts",
                "entity_id": "{contract_cloud}",
            },
        ),
        RecordSpec(
            slug="agent_file_attachment",
            endpoint="/api/v1/agent/execute",
            list_endpoint="/api/v1/files/list/payments/{payment_income}",
            depends_on=("payment_income",),
            match_field="original_name",
            match_value="yushu-payment-evidence.txt",
            result_path=("result", "id"),
            refresh_supported=False,
            payload={
                "tool_name": "files.upload",
                "agent_name": "codex-e2e-seed",
                "arguments": {
                    "original_name": "yushu-payment-evidence.txt",
                    "content_type": "text/plain",
                    "content_base64": base64.b64encode(
                        b"Yushu Tech E2E payment evidence for OPC."
                    ).decode("ascii"),
                    "entity_type": "payments",
                    "entity_id": "{payment_income}",
                    "description": "语数科技 E2E 付款凭证附件。",
                },
            },
        ),
    ]


def render_value(value: Any, ids: dict[str, str]) -> Any:
    if isinstance(value, str):
        def replace(match: re.Match[str]) -> str:
            key = match.group(1)
            if key not in ids:
                raise KeyError(key)
            return ids[key]

        return PLACEHOLDER_RE.sub(replace, value)
    if isinstance(value, list):
        return [render_value(item, ids) for item in value]
    if isinstance(value, dict):
        return {key: render_value(item, ids) for key, item in value.items()}
    return value


def field_value(row: Any, field: str) -> Any:
    current = row
    for part in field.split("."):
        if not isinstance(current, dict):
            return None
        current = current.get(part)
    return current


def extract_rows(body: Any) -> list[dict[str, Any]]:
    if isinstance(body, list):
        return [item for item in body if isinstance(item, dict)]
    if isinstance(body, dict):
        for key in ("data", "items", "rows", "files", "keys"):
            value = body.get(key)
            if isinstance(value, list):
                return [item for item in value if isinstance(item, dict)]
        if isinstance(body.get("result"), list):
            return [item for item in body["result"] if isinstance(item, dict)]
    return []


def extract_id(body: Any, path: tuple[str, ...] = ("id",)) -> Optional[str]:
    current = body
    for part in path:
        if not isinstance(current, dict):
            current = None
            break
        current = current.get(part)
    if isinstance(current, str) and current:
        return current

    if isinstance(body, dict):
        for key in ("id",):
            if isinstance(body.get(key), str):
                return body[key]
        for key in ("data", "result"):
            nested = body.get(key)
            if isinstance(nested, dict) and isinstance(nested.get("id"), str):
                return nested["id"]
    return None


def get_json(response: requests.Response) -> Any:
    try:
        return response.json()
    except ValueError:
        return {"_text": response.text}


def fail_for_response(action: str, response: requests.Response) -> None:
    if 200 <= response.status_code < 300:
        return
    body = response.text[:500]
    raise RuntimeError(f"{action} failed: HTTP {response.status_code}: {body}")


def find_existing(client: SeedClient, spec: RecordSpec, ids: dict[str, str]) -> Optional[str]:
    endpoint = render_value(spec.list_endpoint or spec.endpoint, ids)
    response = client.get(endpoint)
    if response.status_code == 404:
        return None
    fail_for_response(f"list {endpoint}", response)
    body = get_json(response)
    target = render_value(spec.match_value, ids)
    for row in extract_rows(body):
        if str(field_value(row, spec.match_field)) == str(target):
            found = extract_id(row, spec.result_path) or extract_id(row)
            if found:
                return found
    return None


def create_record(client: SeedClient, spec: RecordSpec, ids: dict[str, str]) -> str:
    payload = render_value(spec.payload, ids)
    response = client.post(spec.endpoint, payload)
    fail_for_response(f"create {spec.slug}", response)
    body = get_json(response)
    record_id = extract_id(body, spec.result_path)
    if not record_id:
        raise RuntimeError(f"create {spec.slug} returned no id: {body}")
    return record_id


def update_record(client: SeedClient, spec: RecordSpec, record_id: str, ids: dict[str, str]) -> None:
    if not spec.refresh_supported:
        return
    if spec.endpoint == "/api/v1/agent/execute":
        return
    payload = render_value(spec.payload, ids)
    response = client.put(f"{spec.endpoint}/{record_id}", payload)
    fail_for_response(f"refresh {spec.slug}", response)


def ensure_user_context(client: SeedClient) -> str:
    response = client.get("/auth/me")
    fail_for_response("get current user", response)
    body = get_json(response)
    user_id = body.get("id") if isinstance(body, dict) else None
    if not user_id:
        raise RuntimeError(f"auth/me returned no id: {body}")
    return str(user_id)


def ensure_company_context(client: SeedClient) -> str:
    response = client.get("/api/companies")
    fail_for_response("list companies", response)
    companies = extract_rows(get_json(response))
    for company in companies:
        if company.get("name") == client.config.company_name:
            company_id = str(company["id"])
            switch = client.get(f"/api/companies/{urllib.parse.quote(company_id)}/switch")
            fail_for_response("switch company", switch)
            return company_id

    payload = {
        "name": client.config.company_name,
        "legal_name": f"{client.config.company_name}有限公司",
        "country": "CN",
        "locale": "zh",
        "base_currency": "CNY",
        "industry": "AI 数据智能与企业软件",
    }
    response = client.post("/api/companies", payload)
    fail_for_response("create company", response)
    company_id = extract_id(get_json(response))
    if not company_id:
        raise RuntimeError("create company returned no id")
    switch = client.get(f"/api/companies/{urllib.parse.quote(company_id)}/switch")
    fail_for_response("switch company", switch)
    return company_id


def seed_dataset(config: SeedConfig) -> dict[str, Any]:
    if config.cleanup_first:
        raise RuntimeError("cleanup_first is not implemented; default seeding is non-destructive")

    client = SeedClient(config)
    user_id = ensure_user_context(client)
    company_id = ensure_company_context(client)
    ids: dict[str, str] = {"company": company_id, "current_user": user_id}
    created: list[str] = []
    reused: list[str] = []

    for spec in build_dataset():
        missing = [dep for dep in spec.depends_on if dep not in ids]
        if missing:
            raise RuntimeError(f"{spec.slug} dependencies missing: {missing}")

        existing = find_existing(client, spec, ids)
        if existing:
            if config.refresh_existing:
                update_record(client, spec, existing, ids)
            ids[spec.slug] = existing
            reused.append(spec.slug)
            action = "refresh" if config.refresh_existing else "reuse"
            print(f"{action:<8}{spec.slug:<26} {existing}")
            continue

        record_id = create_record(client, spec, ids)
        ids[spec.slug] = record_id
        created.append(spec.slug)
        print(f"create {spec.slug:<26} {record_id}")

    manifest = {
        "dataset": "yushu-tech-complete-e2e",
        "company_name": config.company_name,
        "company_id": company_id,
        "user_id": user_id,
        "base_url": config.base_url,
        "created": created,
        "reused": reused,
        "ids": ids,
    }
    if config.manifest_path:
        with open(config.manifest_path, "w", encoding="utf-8") as handle:
            json.dump(manifest, handle, ensure_ascii=False, indent=2)
            handle.write("\n")
        print(f"manifest {config.manifest_path}")
    return manifest


def bootstrap_sqlite_session(db_path: str, company_name: str) -> str:
    session_id = str(uuid.uuid4())
    user_id = str(uuid.uuid4())
    company_id = str(uuid.uuid4())
    member_id = str(uuid.uuid4())
    profile_id = str(uuid.uuid4())
    conn = sqlite3.connect(db_path)
    try:
        conn.executescript("PRAGMA foreign_keys=ON;")
        conn.execute(
            "INSERT INTO users (id, email, name, created_at, updated_at) VALUES (?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
            (user_id, DEFAULT_EMAIL, DEFAULT_USER_NAME),
        )
        conn.execute(
            """INSERT INTO companies (
                id, name, legal_name, country, locale, base_currency, industry,
                city, province, address, created_at, updated_at
            ) VALUES (?, ?, ?, 'CN', 'zh', 'CNY', ?, '杭州', '浙江', ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)""",
            (
                company_id,
                company_name,
                f"{company_name}有限公司",
                "AI 数据智能与企业软件",
                "杭州市余杭区未来科技城语数路 88 号",
            ),
        )
        conn.execute(
            "INSERT INTO company_members (id, user_id, company_id, role, is_default, created_at) VALUES (?, ?, ?, 'owner', 1, CURRENT_TIMESTAMP)",
            (member_id, user_id, company_id),
        )
        conn.execute(
            "INSERT INTO sessions (id, user_id, access_token, current_company_id, expires_at) VALUES (?, ?, 'e2e-token', ?, datetime('now', '+1 day'))",
            (session_id, user_id, company_id),
        )
        conn.execute(
            """INSERT INTO profiles (
                id, company_id, user_id, display_name, email, department, position, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, 'QA', 'E2E Tester', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)""",
            (profile_id, company_id, user_id, DEFAULT_USER_NAME, DEFAULT_EMAIL),
        )
        conn.commit()
    finally:
        conn.close()
    return session_id


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Seed the complete 语数科技 E2E dataset.")
    parser.add_argument("--base-url", default=os.environ.get("ONEMAN_BASE_URL", "http://127.0.0.1:8987"))
    parser.add_argument("--session", default=os.environ.get("SESSION_ID"))
    parser.add_argument("--company-name", default=os.environ.get("ONEMAN_E2E_COMPANY", DEFAULT_COMPANY_NAME))
    parser.add_argument("--manifest", default=os.environ.get("ONEMAN_E2E_MANIFEST"))
    parser.add_argument("--refresh-existing", action="store_true")
    parser.add_argument(
        "--bootstrap-sqlite",
        help="Create a test user/company/session directly in this isolated SQLite database before API seeding.",
    )
    return parser.parse_args(argv)


def main(argv: list[str]) -> int:
    args = parse_args(argv)
    session = args.session
    if args.bootstrap_sqlite:
        session = bootstrap_sqlite_session(args.bootstrap_sqlite, args.company_name)
        print(f"bootstrap session {session}")

    if not session:
        print("SESSION_ID is required unless --bootstrap-sqlite is used", file=sys.stderr)
        return 2

    config = SeedConfig(
        base_url=args.base_url,
        session=session,
        company_name=args.company_name,
        refresh_existing=args.refresh_existing,
        manifest_path=args.manifest,
    )
    manifest = seed_dataset(config)
    print(
        f"seeded {len(manifest['created'])} created, {len(manifest['reused'])} reused "
        f"for {manifest['company_name']}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
