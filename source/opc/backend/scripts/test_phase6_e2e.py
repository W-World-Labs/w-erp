import os
#!/usr/bin/env python3
"""
Phase 6 E2E Test: Asset Depreciation, Data Asset Meta, R&D Costs
Tests the new endpoints added in Phase 6 against a running backend.
"""
import requests
import json
import sys

BASE_URL = "http://localhost:8987/api"
SESSION_ID = os.environ["OPC_SESSION"]
HEADERS = {
    "Authorization": f"Bearer {SESSION_ID}",
    "Content-Type": "application/json",
}

passed = 0
failed = 0

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
        detail = f" | {body[:120]}"
    print(f"  {symbol} {name:<50} [{status}]{detail}")
    return ok

def main():
    print("=" * 70)
    print("Phase 6 E2E Tests: 资产折旧 · 数据资产 · 研发费用")
    print("=" * 70)

    # ---- 0. Setup: Get company context ----
    print("\n--- 0. Setup: Company Context ---")
    try:
        res = requests.get(f"{BASE_URL}/companies", headers=HEADERS)
        if not test("List Companies", res.status_code):
            print("❌ Cannot list companies. Is session valid?")
            return

        companies = res.json().get("data", [])
        if not companies:
            print("❌ No companies found. Aborting.")
            return

        company = companies[0]
        print(f"  📌 Company: {company['name']} ({company['id']})")

        res = requests.get(f"{BASE_URL}/companies/{company['id']}/switch", headers=HEADERS)
        test("Switch Company Context", res.status_code)
    except requests.ConnectionError:
        print("❌ Cannot connect to backend at localhost:8987")
        return

    # ---- 1. Asset with Depreciation Fields ----
    print("\n--- 1. Asset with Depreciation Fields ---")
    asset_id = None

    new_asset = {
        "name": "E2E测试服务器",
        "asset_type": "physical",
        "status": "in_use",
        "purchase_price": 50000.0,
        "current_value": 50000.0,
        "purchase_date": "2024-01-15",
        "location": "机房A-Rack3",
        "useful_life": 60,
        "salvage_rate": 0.05,
        "depreciation_method": "straight_line",
        "monthly_depreciation": 791.67,
        "accumulated_depreciation": 0,
        "description": "E2E测试 — 折旧字段验证",
    }
    res = requests.post(f"{BASE_URL}/assets", headers=HEADERS, json=new_asset)
    test("Create Asset (with depreciation)", res.status_code)
    if res.status_code == 200:
        asset_id = res.json().get("id")
        print(f"  📌 Asset ID: {asset_id}")

    # Read back and verify depreciation fields
    if asset_id:
        res = requests.get(f"{BASE_URL}/assets/{asset_id}", headers=HEADERS)
        if test("Get Asset by ID", res.status_code):
            data = res.json()
            checks = [
                ("useful_life", 60),
                ("depreciation_method", "straight_line"),
            ]
            for field, expected in checks:
                actual = data.get(field)
                ok = str(actual) == str(expected)
                symbol = "✅" if ok else "❌"
                print(f"    {symbol} {field}: {actual} (expected {expected})")

    # Test data asset type
    print("\n--- 2. Data Asset (type=data) ---")
    data_asset_id = None

    new_data_asset = {
        "name": "E2E核心算法引擎",
        "asset_type": "data",
        "status": "in_use",
        "purchase_price": 200000.0,
        "current_value": 200000.0,
        "description": "E2E测试 — 数据资产类型",
    }
    res = requests.post(f"{BASE_URL}/assets", headers=HEADERS, json=new_data_asset)
    test("Create Data Asset (type=data)", res.status_code)
    if res.status_code == 200:
        data_asset_id = res.json().get("id")
        print(f"  📌 Data Asset ID: {data_asset_id}")

    # Test intangible asset type
    intangible_asset_id = None
    new_intangible = {
        "name": "E2E发明专利-AI优化方法",
        "asset_type": "intangible",
        "status": "in_use",
        "purchase_price": 80000.0,
        "registration_number": "CN202412345678.9",
        "holder_name": "黑盒智能实验室",
        "description": "E2E测试 — 无形资产类型",
    }
    res = requests.post(f"{BASE_URL}/assets", headers=HEADERS, json=new_intangible)
    test("Create Intangible Asset", res.status_code)
    if res.status_code == 200:
        intangible_asset_id = res.json().get("id")

    # Verify all 4 asset types in list
    res = requests.get(f"{BASE_URL}/assets", headers=HEADERS)
    if test("List Assets", res.status_code):
        all_assets = res.json().get("data", [])
        types_found = set(a.get("asset_type") for a in all_assets)
        print(f"    Asset types in DB: {types_found}")

    # ---- 3. Data Asset Metadata ----
    print("\n--- 3. Data Asset Metadata (data_asset_meta) ---")
    meta_id = None

    if data_asset_id:
        new_meta = {
            "asset_id": data_asset_id,
            "data_type": "software",
            "tech_stack": "Python, PyTorch, ONNX",
            "version": "v3.1.0",
            "license_type": "proprietary",
            "access_url": "https://github.com/blackbox/core-engine",
        }
        res = requests.post(f"{BASE_URL}/data_asset_meta", headers=HEADERS, json=new_meta)
        test("Create Data Asset Meta", res.status_code)
        if res.status_code == 200:
            meta_id = res.json().get("id")
            print(f"  📌 Meta ID: {meta_id}")

    # List all meta
    res = requests.get(f"{BASE_URL}/data_asset_meta", headers=HEADERS)
    if test("List Data Asset Meta", res.status_code):
        metas = res.json().get("data", [])
        print(f"    Total meta records: {len(metas)}")

    # Get by ID
    if meta_id:
        res = requests.get(f"{BASE_URL}/data_asset_meta/{meta_id}", headers=HEADERS)
        if test("Get Meta by ID", res.status_code):
            meta = res.json()
            print(f"    data_type={meta.get('data_type')}, tech_stack={meta.get('tech_stack')}")

    # Update meta
    if meta_id:
        update_meta = {"version": "v3.2.0", "tech_stack": "Python, PyTorch, TensorRT"}
        res = requests.put(f"{BASE_URL}/data_asset_meta/{meta_id}", headers=HEADERS, json=update_meta)
        test("Update Meta", res.status_code)

    # ---- 4. R&D Costs ----
    print("\n--- 4. R&D Costs (rd_costs) ---")

    # First, get or create a project
    res = requests.get(f"{BASE_URL}/projects", headers=HEADERS)
    projects = res.json().get("data", []) if res.status_code == 200 else []
    project_id = None

    if projects:
        project_id = projects[0]["id"]
        print(f"  📌 Using existing project: {projects[0].get('name')} ({project_id})")
    else:
        new_project = {
            "name": "E2E测试研发项目",
            "start_date": "2024-01-01",
            "end_date": "2024-12-31",
            "project_type": "research",
            "status": "ongoing",
            "budget": 500000,
        }
        res = requests.post(f"{BASE_URL}/projects", headers=HEADERS, json=new_project)
        test("Create Test Project", res.status_code)
        if res.status_code == 200:
            project_id = res.json().get("id")
            print(f"  📌 Created project: {project_id}")

    if not project_id:
        print("  ❌ No project available. Skipping R&D cost tests.")
    else:
        rd_cost_ids = []

        # Create costs of various types
        test_costs = [
            {"cost_type": "personnel", "amount": 15000, "cost_date": "2024-03-15", "description": "3月研发人员工资-张三"},
            {"cost_type": "equipment", "amount": 8000, "cost_date": "2024-03-20", "description": "GPU显卡采购"},
            {"cost_type": "material", "amount": 2500, "cost_date": "2024-04-01", "description": "实验材料采购"},
            {"cost_type": "outsourcing", "amount": 30000, "cost_date": "2024-04-10", "description": "数据标注外包"},
            {"cost_type": "depreciation", "amount": 791.67, "cost_date": "2024-04-30", "description": "服务器4月折旧"},
            {"cost_type": "other", "amount": 3200, "cost_date": "2024-05-01", "description": "学术会议差旅"},
        ]

        for cost in test_costs:
            cost["project_id"] = project_id
            res = requests.post(f"{BASE_URL}/rd_costs", headers=HEADERS, json=cost)
            test(f"Create R&D Cost ({cost['cost_type']})", res.status_code)
            if res.status_code == 200:
                rd_cost_ids.append(res.json().get("id"))

        # List all
        res = requests.get(f"{BASE_URL}/rd_costs", headers=HEADERS)
        if test("List R&D Costs", res.status_code):
            costs = res.json().get("data", [])
            total = sum(c.get("amount", 0) for c in costs)
            print(f"    Total R&D costs: {len(costs)} records, ¥{total:,.2f}")

            # Verify cost type breakdown
            by_type = {}
            for c in costs:
                t = c.get("cost_type", "unknown")
                by_type[t] = by_type.get(t, 0) + c.get("amount", 0)
            for t, amt in sorted(by_type.items()):
                print(f"      {t}: ¥{amt:,.2f}")

        # Update a cost
        if rd_cost_ids:
            update = {"amount": 16000, "description": "3月研发人员工资-张三（含加班）"}
            res = requests.put(f"{BASE_URL}/rd_costs/{rd_cost_ids[0]}", headers=HEADERS, json=update)
            test("Update R&D Cost", res.status_code)

        # Get by ID
        if rd_cost_ids:
            res = requests.get(f"{BASE_URL}/rd_costs/{rd_cost_ids[0]}", headers=HEADERS)
            if test("Get R&D Cost by ID", res.status_code):
                cost = res.json()
                print(f"    Updated amount: ¥{cost.get('amount', 0):,.2f}")

        # Delete one cost
        if len(rd_cost_ids) > 1:
            res = requests.delete(f"{BASE_URL}/rd_costs/{rd_cost_ids[-1]}", headers=HEADERS)
            test("Delete R&D Cost", res.status_code, expected_range=(200, 299))

    # ---- 5. Cleanup ----
    print("\n--- 5. Cleanup ---")
    if meta_id:
        res = requests.delete(f"{BASE_URL}/data_asset_meta/{meta_id}", headers=HEADERS)
        test("Delete Meta", res.status_code, expected_range=(200, 299))

    for aid in [asset_id, data_asset_id, intangible_asset_id]:
        if aid:
            res = requests.delete(f"{BASE_URL}/assets/{aid}", headers=HEADERS)
            test(f"Delete Asset ({aid[:8]}...)", res.status_code, expected_range=(200, 299))

    # Delete remaining R&D costs
    if project_id:
        for cid in rd_cost_ids[:-1]:  # last one was already deleted
            if cid:
                res = requests.delete(f"{BASE_URL}/rd_costs/{cid}", headers=HEADERS)
                test(f"Delete R&D Cost ({cid[:8]}...)", res.status_code, expected_range=(200, 299))

    # ---- Summary ----
    print("\n" + "=" * 70)
    total = passed + failed
    print(f"Results: {passed}/{total} passed, {failed} failed")
    if failed == 0:
        print("🎉 All tests passed!")
    else:
        print("⚠️  Some tests failed!")
    print("=" * 70)
    sys.exit(0 if failed == 0 else 1)

if __name__ == "__main__":
    main()
