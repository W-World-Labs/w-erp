import os
import requests
import json

BASE_URL = "http://localhost:8987/api"
SESSION_ID = os.environ["OPC_SESSION"]

HEADERS = {
    "Authorization": f"Bearer {SESSION_ID}",
    "Content-Type": "application/json"
}

# List of GET endpoints to test
endpoints = [
    # Generic routes (merged at /api)
    "/company",
    "/projects",
    "/payments",
    "/contracts",
    "/salary",
    "/rent",
    "/licenses",
    "/reminders",
    "/kpi",
    "/tax",
    "/categories",
    "/logs",
    "/stats/dashboard",
    "/company_info",
    "/branches",
    "/milestones",

    # Nested routes (try with and without trailing slash)
    "/assets",
    "/invoices",
    "/budgets",
    "/employees",
    "/bank/accounts",
    "/companies",
    "/files",
    "/ai",
]

print("Using the operator-provided session")
print(f"{'ENDPOINT':<30} | {'STATUS':<6} | {'RESULT'}")
print("-" * 60)

passed = 0
failed = 0

for ep in endpoints:
    url = f"{BASE_URL}{ep}"
    try:
        response = requests.get(url, headers=HEADERS, timeout=5)
        status = response.status_code
        
        result = "✅ PASS" if status == 200 else f"❌ FAIL ({status})"
        if status == 200:
            passed += 1
        else:
            failed += 1
            
        print(f"{ep:<30} | {status:<6} | {result}")
        if status != 200:
             print(f"   Response: {response.text[:100]}...")

    except Exception as e:
        print(f"{ep:<30} | ERR    | ❌ FAIL ({str(e)})")
        failed += 1

print("-" * 60)
print(f"Total: {len(endpoints)} | Passed: {passed} | Failed: {failed}")
