#!/usr/bin/env python3
"""
OPC — 全栈自底向顶逐层测试
Layer 1: Health + DB
Layer 2: Auth Session
Layer 3: Dedicated Modules (assets, invoices, employees, budgets, bank)
Layer 4: Generic CRUD (payments, contracts, salary, rent, projects, etc.)
Layer 5: Cloud Cost Management
Layer 6: Dashboard Stats + System Status
Layer 7: Cleanup
"""
import requests
import json
import sys
import time

BASE_URL = "http://localhost:8987"
API = f"{BASE_URL}/api"
SESSION_ID = None  # Will be set from auth or env
HEADERS = {}
CREATED_IDS = {}  # Track IDs for cleanup

passed = 0
failed = 0
skipped = 0

# ── Helpers ─────────────────────────────────────────────────

def test(name, status, expected_range=(200, 299), body=None):
    global passed, failed
    ok = expected_range[0] <= status <= expected_range[1]
    symbol = "✅" if ok else "❌"
    if ok:
        passed += 1
    else:
        failed += 1
    detail = ""
    if not ok and body:
        detail = f" | {str(body)[:120]}"
    print(f"  {symbol} {name:<55} [{status}]{detail}")
    return ok

def skip(name, reason=""):
    global skipped
    skipped += 1
    print(f"  ⏭️  {name:<55} [SKIP] {reason}")

def crud_test(table_name, create_data, update_data=None, list_key="data"):
    """Test full CRUD cycle for a generic table."""
    print(f"\n  ── {table_name} ──")
    record_id = None

    # CREATE
    res = requests.post(f"{API}/{table_name}", headers=HEADERS, json=create_data)
    if test(f"POST /{table_name}", res.status_code):
        record_id = res.json().get("id")
        CREATED_IDS.setdefault(table_name, []).append(record_id)

    # LIST
    res = requests.get(f"{API}/{table_name}", headers=HEADERS)
    if test(f"GET /{table_name}", res.status_code):
        data = res.json()
        count = len(data.get(list_key, data if isinstance(data, list) else []))
        print(f"       → {count} records")

    # GET by ID
    if record_id:
        res = requests.get(f"{API}/{table_name}/{record_id}", headers=HEADERS)
        test(f"GET /{table_name}/{{id}}", res.status_code)

    # UPDATE
    if record_id and update_data:
        res = requests.put(f"{API}/{table_name}/{record_id}", headers=HEADERS, json=update_data)
        test(f"PUT /{table_name}/{{id}}", res.status_code)

    # DELETE
    if record_id:
        res = requests.delete(f"{API}/{table_name}/{record_id}", headers=HEADERS)
        test(f"DELETE /{table_name}/{{id}}", res.status_code, expected_range=(200, 299))
        # Remove from cleanup since already deleted
        CREATED_IDS.get(table_name, []).remove(record_id)

    return record_id

# ── Layer 1: Health + Connectivity ─────────────────────────

def test_layer1():
    print("\n" + "=" * 70)
    print("Layer 1: Health Check + Database Connectivity")
    print("=" * 70)

    try:
        res = requests.get(f"{BASE_URL}/health", timeout=5)
        if not test("GET /health", res.status_code):
            print("❌ Server not responding. Start with: cargo run")
            return False
        data = res.json()
        print(f"       → status={data.get('status')}, version={data.get('version')}")
        return True
    except requests.ConnectionError:
        print("❌ Cannot connect to backend at localhost:8987")
        print("   Please start: cd backend && cargo run")
        return False

# ── Layer 2: Auth ──────────────────────────────────────────

def test_layer2():
    global SESSION_ID, HEADERS
    print("\n" + "=" * 70)
    print("Layer 2: Authentication & Session")
    print("=" * 70)

    import os
    SESSION_ID = os.environ.get("OPC_SESSION")
    if not SESSION_ID:
        print("Set OPC_SESSION to a session from your local test instance")
        return False

    HEADERS = {
        "Authorization": f"Bearer {SESSION_ID}",
        "Content-Type": "application/json",
    }

    # Test auth/me
    res = requests.get(f"{BASE_URL}/auth/me", headers=HEADERS)
    if not test("GET /auth/me", res.status_code):
        print("  ❌ Session expired or invalid")
        return False
    user = res.json()
    print(f"       → user={user.get('name', user.get('email'))}, id={user.get('id', '')[:8]}...")

    # Test 401 without auth
    res = requests.get(f"{API}/companies", headers={"Content-Type": "application/json"})
    test("Reject unauthenticated request", res.status_code, expected_range=(401, 401))

    # Test company context
    res = requests.get(f"{API}/companies", headers=HEADERS)
    if test("GET /api/companies", res.status_code):
        companies = res.json().get("data", [])
        if companies:
            company = companies[0]
            print(f"       → company={company['name']}, id={company['id'][:8]}...")
            # Switch to first company
            res2 = requests.get(f"{API}/companies/{company['id']}/switch", headers=HEADERS)
            test("Switch company context", res2.status_code)
        else:
            print("  ⚠️  No companies found")

    return True

# ── Layer 3: Dedicated Modules ─────────────────────────────

def test_layer3():
    print("\n" + "=" * 70)
    print("Layer 3: Dedicated Modules (CRUD)")
    print("=" * 70)

    # ── Assets ──
    print("\n  ── assets ──")
    asset_id = None
    res = requests.post(f"{API}/assets", headers=HEADERS, json={
        "name": "E2E测试笔记本",
        "asset_type": "physical",
        "status": "in_use",
        "purchase_price": 8999,
        "current_value": 7000,
        "location": "办公室A",
        "description": "自底向上测试创建",
    })
    if test("POST /assets", res.status_code):
        asset_id = res.json().get("id")
        CREATED_IDS.setdefault("assets", []).append(asset_id)

    res = requests.get(f"{API}/assets", headers=HEADERS)
    test("GET /assets", res.status_code)

    if asset_id:
        res = requests.get(f"{API}/assets/{asset_id}", headers=HEADERS)
        test("GET /assets/{id}", res.status_code)

        res = requests.put(f"{API}/assets/{asset_id}", headers=HEADERS, json={
            "name": "E2E测试笔记本", "asset_type": "physical", "status": "in_use",
            "current_value": 6500, "description": "已更新"
        })
        test("PUT /assets/{id}", res.status_code)

    # ── Invoices ──
    print("\n  ── invoices ──")
    inv_id = None
    res = requests.post(f"{API}/invoices", headers=HEADERS, json={
        "invoice_number": f"E2E-INV-{int(time.time())}",
        "amount": 5000,
        "tax_amount": 750,
        "invoice_date": "2024-06-15",
        "supplier": "E2E测试供应商",
        "status": "pending",
        "invoice_type": "input",
    })
    if test("POST /invoices", res.status_code):
        inv_id = res.json().get("id")
        CREATED_IDS.setdefault("invoices", []).append(inv_id)

    res = requests.get(f"{API}/invoices", headers=HEADERS)
    test("GET /invoices", res.status_code)

    if inv_id:
        res = requests.put(f"{API}/invoices/{inv_id}", headers=HEADERS, json={
            "invoice_number": f"E2E-INV-{int(time.time())}",
            "amount": 5000, "invoice_date": "2024-06-15",
            "supplier": "E2E测试供应商", "status": "approved",
            "description": "E2E已审批",
        })
        test("PUT /invoices/{id}", res.status_code)

    # ── Employees ──
    print("\n  ── employees ──")
    emp_id = None
    res = requests.post(f"{API}/employees", headers=HEADERS, json={
        "name": "E2E测试员工",
        "email": "e2e@test-com.example.invalid",
        "department": "技术部",
        "position": "工程师",
        "hire_date": "2024-01-15",
        "status": "active",
        "base_salary": 15000,
        "bank_name": "中国银行",
        "bank_account": "0002000000000001",
    })
    if test("POST /employees", res.status_code):
        emp_id = res.json().get("id")
        CREATED_IDS.setdefault("employees", []).append(emp_id)

    res = requests.get(f"{API}/employees", headers=HEADERS)
    test("GET /employees", res.status_code)

    if emp_id:
        res = requests.put(f"{API}/employees/{emp_id}", headers=HEADERS, json={
            "name": "E2E测试员工", "email": "e2e@test-com.example.invalid",
            "department": "技术部", "position": "高级工程师",
            "status": "active", "base_salary": 18000,
        })
        test("PUT /employees/{id}", res.status_code)

    # ── Budgets ──
    print("\n  ── budgets ──")
    budget_id = None
    res = requests.post(f"{API}/budgets", headers=HEADERS, json={
        "category": "研发费用",
        "budget_type": "monthly",
        "period": "2024-06",
        "budget_amount": 100000,
        "notes": "E2E测试预算",
    })
    if test("POST /budgets", res.status_code):
        budget_id = res.json().get("id")
        CREATED_IDS.setdefault("budgets", []).append(budget_id)

    res = requests.get(f"{API}/budgets", headers=HEADERS)
    test("GET /budgets", res.status_code)

    if budget_id:
        res = requests.put(f"{API}/budgets/{budget_id}", headers=HEADERS, json={
            "category": "研发费用", "budget_type": "monthly",
            "period": "2024-06", "budget_amount": 100000,
            "actual_amount": 85000,
        })
        test("PUT /budgets/{id}", res.status_code)

        res = requests.delete(f"{API}/budgets/{budget_id}", headers=HEADERS)
        test("DELETE /budgets/{id}", res.status_code, expected_range=(200, 299))
        CREATED_IDS.get("budgets", []).remove(budget_id)

    # ── Bank Accounts ──
    print("\n  ── bank/accounts ──")
    bank_id = None
    res = requests.post(f"{API}/bank/accounts", headers=HEADERS, json={
        "bank_name": "E2E测试银行",
        "account_name": "主账户",
        "account_number": "1234567890",
        "account_type": "checking",
        "balance": 500000,
        "currency": "CNY",
    })
    if test("POST /bank/accounts", res.status_code):
        bank_id = res.json().get("id")
        CREATED_IDS.setdefault("bank_accounts", []).append(bank_id)

    res = requests.get(f"{API}/bank/accounts", headers=HEADERS)
    test("GET /bank/accounts", res.status_code)

    if bank_id:
        res = requests.put(f"{API}/bank/accounts/{bank_id}", headers=HEADERS, json={
            "bank_name": "E2E测试银行", "account_name": "主账户",
            "account_number": "1234567890", "account_type": "checking",
            "balance": 480000, "currency": "CNY",
        })
        test("PUT /bank/accounts/{id}", res.status_code)

# ── Layer 4: Generic CRUD modules ──────────────────────────

def test_layer4():
    print("\n" + "=" * 70)
    print("Layer 4: Generic CRUD Modules")
    print("=" * 70)

    # Payments
    crud_test("payments", {
        "payment_type": "支出",
        "amount": 3000,
        "payment_date": "2024-06-20",
        "payer": "我方公司",
        "payee": "E2E测试收款方",
        "status": "pending",
        "description": "E2E测试付款",
    }, {"description": "E2E测试付款（已更新）"})

    # Contracts
    crud_test("contracts", {
        "contract_name": "E2E测试合同",
        "contract_type": "service",
        "party_a": "我方公司",
        "party_b": "E2E测试乙方",
        "counterparty": "E2E测试乙方",
        "amount": 50000,
        "start_date": "2024-01-01",
        "end_date": "2024-12-31",
        "status": "active",
    }, {"status": "completed"})

    # Salary (needs employee_id from existing data)
    emp_res = requests.get(f"{API}/employees", headers=HEADERS)
    emp_list = emp_res.json().get("data", []) if emp_res.status_code == 200 else []
    sal_emp_id = emp_list[0]["id"] if emp_list else None
    if sal_emp_id:
        crud_test("salary", {
            "employee_id": sal_emp_id,
            "period": "2024-06",
            "base_salary": 15000,
            "net_salary": 14000,
            "status": "pending",
        }, {"status": "paid"})
    else:
        skip("salary CRUD", "no employees to reference")

    # Rent
    crud_test("rent", {
        "location_name": "E2E测试办公室",
        "monthly_rent": 8000,
        "contract_start_date": "2024-01-01",
        "contract_end_date": "2025-12-31",
        "property_type": "office",
        "status": "active",
    }, {"monthly_rent": 8500})

    # Projects
    crud_test("projects", {
        "name": "E2E测试项目",
        "start_date": "2024-01-01",
        "end_date": "2024-12-31",
        "status": "ongoing",
        "budget": 200000,
    }, {"status": "completed"})

    # Vendors
    crud_test("vendors", {
        "name": "E2E测试供应商有限公司",
        "vendor_type": "supplier",
        "contact_person": "张经理",
        "contact_phone": "00000138000",
        "status": "active",
    }, {"contact_phone": "00000139000"})

    # Licenses
    crud_test("licenses", {
        "license_type": "business_license",
        "license_name": "E2E营业执照",
        "license_number": "91330100MATEST001",
        "issue_date": "2024-01-01",
        "expiry_date": "2034-01-01",
    }, {"notes": "E2E测试更新"})

    # Categories
    crud_test("categories", {
        "name": "E2E测试分类",
        "category_type": "expense",
        "description": "自底向上测试",
    }, {"description": "已更新"})

    # Reminders
    crud_test("reminders", {
        "title": "E2E测试提醒",
        "remind_date": "2024-07-01",
        "status": "pending",
    }, {"status": "completed"})

    # Tax
    crud_test("tax", {
        "tax_type": "增值税",
        "period": "2024-06",
        "tax_amount": 15000,
        "taxable_amount": 100000,
        "status": "pending",
    }, {"status": "filed"})

    # Utilities
    crud_test("utilities", {
        "utility_type": "电费",
        "amount": 3500,
        "location_name": "E2E测试办公室",
        "period": "2024-06",
        "status": "unpaid",
    }, {"status": "paid"})

# ── Layer 5: Cloud Cost Management ─────────────────────────

def test_layer5():
    print("\n" + "=" * 70)
    print("Layer 5: Cloud Cost Management")
    print("=" * 70)

    # Cloud Providers (account_id FK exclusion fix verified)
    crud_test("cloud_providers", {
        "provider_name": "E2E阿里云",
        "provider_type": "IaaS",
        "account_name": "主账户",
        "account_id": "e2e-cloud-001",
    }, {"notes": "E2E测试更新"})

    # Cloud Resources
    crud_test("cloud_resources", {
        "resource_type": "ECS",
        "resource_name": "E2E测试服务器",
        "specification": "ecs.c6.xlarge",
        "region": "cn-hangzhou",
        "start_date": "2024-01-01",
        "monthly_cost": 500,
        "status": "running",
    }, {"monthly_cost": 550})

    # Domain Subscriptions
    crud_test("domain_subscriptions", {
        "domain_name": "e2e-test.com",
        "registrar": "阿里云",
        "registration_date": "2024-01-01",
        "expiry_date": "2025-01-01",
        "annual_fee": 69,
        "status": "active",
    }, {"annual_fee": 79})

    # Software Subscriptions
    crud_test("software_subscriptions", {
        "software_name": "E2E测试软件",
        "vendor": "E2E厂商",
        "subscription_type": "SaaS",
        "billing_cycle": "monthly",
        "start_date": "2024-01-01",
        "next_billing_date": "2024-07-01",
        "total_price": 999,
        "status": "active",
    }, {"total_price": 1099})

# ── Layer 6: Dashboard + System Status ─────────────────────

def test_layer6():
    print("\n" + "=" * 70)
    print("Layer 6: Dashboard Stats + System Status")
    print("=" * 70)

    res = requests.get(f"{API}/stats/dashboard", headers=HEADERS)
    if test("GET /stats/dashboard", res.status_code):
        stats = res.json()
        for key in sorted(stats.keys()):
            val = stats[key]
            if isinstance(val, (int, float)):
                print(f"       → {key}: {val}")

    res = requests.get(f"{API}/system/status", headers=HEADERS)
    if test("GET /system/status", res.status_code):
        status = res.json()
        print(f"       → tables: {status.get('table_count', '?')}")

    # Logs
    res = requests.get(f"{API}/logs", headers=HEADERS)
    test("GET /logs", res.status_code)

# ── Layer 7: Cleanup ───────────────────────────────────────

def test_layer7():
    print("\n" + "=" * 70)
    print("Layer 7: Cleanup")
    print("=" * 70)

    # Clean up dedicated module records
    cleanup_map = {
        "assets": "/assets",
        "invoices": "/invoices",
        "employees": "/employees",
        "budgets": "/budgets",
        "bank_accounts": "/bank/accounts",
    }
    for table, path in cleanup_map.items():
        for record_id in CREATED_IDS.get(table, []):
            if record_id:
                res = requests.delete(f"{API}{path}/{record_id}", headers=HEADERS)
                test(f"Cleanup {table} {record_id[:8]}...", res.status_code, expected_range=(200, 299))

    # Clean up generic CRUD records (any remaining)
    for table in list(CREATED_IDS.keys()):
        if table in cleanup_map:
            continue
        for record_id in CREATED_IDS.get(table, []):
            if record_id:
                res = requests.delete(f"{API}/{table}/{record_id}", headers=HEADERS)
                test(f"Cleanup {table} {record_id[:8]}...", res.status_code, expected_range=(200, 299))

# ── Main ───────────────────────────────────────────────────

def main():
    print("╔══════════════════════════════════════════════════════════════════════╗")
    print("║           OPC — 自底向顶全栈集成测试                        ║")
    print("╚══════════════════════════════════════════════════════════════════════╝")

    # Layer 1: Health
    if not test_layer1():
        return

    # Layer 2: Auth
    if not test_layer2():
        return

    # Layer 3: Dedicated Modules
    test_layer3()

    # Layer 4: Generic CRUD
    test_layer4()

    # Layer 5: Cloud Cost
    test_layer5()

    # Layer 6: Dashboard
    test_layer6()

    # Layer 7: Cleanup
    test_layer7()

    # ── Summary ──
    total = passed + failed
    print("\n" + "═" * 70)
    print(f"  Results: {passed}/{total} passed, {failed} failed, {skipped} skipped")
    if failed == 0:
        print("  🎉 全部测试通过!")
    else:
        print("  ⚠️  部分测试未通过!")
    print("═" * 70)
    sys.exit(0 if failed == 0 else 1)

if __name__ == "__main__":
    main()
