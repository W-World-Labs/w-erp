import requests
import json
import os

BASE_URL = "http://localhost:8987/api"
SESSION_ID = os.environ["OPC_SESSION"]

HEADERS = {
    "Authorization": f"Bearer {SESSION_ID}",
    # Content-Type not set globally because file upload needs multipart
}

def print_result(name, status, response=None):
    symbol = "✅" if 200 <= status < 300 else "❌"
    print(f"{name:<40} | {status:<6} | {symbol} {response if status >= 400 else ''}")

def main():
    print("Using the operator-provided session")
    print("-" * 80)

    # 1. Get User Companies
    print("\n--- 1. Company Context ---")
    try:
        res = requests.get(f"{BASE_URL}/companies", headers=HEADERS)
        print_result("List User Companies", res.status_code)
        
        if res.status_code != 200:
            print("Failed to list companies. Aborting.")
            return

        data = res.json().get("data", [])
        if not data:
            print("No companies found for user. Aborting.")
            return
            
        company = data[0]
        company_id = company["id"]
        print(f"Selected Company: {company['name']} ({company_id})")

        # 2. Switch Context
        res = requests.get(f"{BASE_URL}/companies/{company_id}/switch", headers=HEADERS)
        print_result("Switch Company Context", res.status_code)

    except Exception as e:
        print(f"Error in Company section: {e}")
        return

    # 3. Files Test
    print("\n--- 2. Files API ---")
    file_id = None
    try:
        # Upload
        files = {'file': ('test_file.txt', b'Hello World Content')}
        # headers must not have Content-Type for multipart
        upload_headers = {"Authorization": f"Bearer {SESSION_ID}"}
        
        # Upload for entity (Company)
        res = requests.post(
            f"{BASE_URL}/files/upload/company/{company_id}", 
            headers=upload_headers, 
            files=files
        )
        print_result("Upload File (Entity: Company)", res.status_code, res.text)
        
        if res.status_code == 200:
            file_data = res.json()
            file_id = file_data.get("id")
            print(f"Uploaded File ID: {file_id}")

        # List
        res = requests.get(
            f"{BASE_URL}/files/list/company/{company_id}", 
            headers=HEADERS
        )
        print_result("List Files (Entity: Company)", res.status_code)
        
        # Delete
        if file_id:
            res = requests.delete(f"{BASE_URL}/files/{file_id}", headers=HEADERS)
            print_result("Delete File", res.status_code)

    except Exception as e:
        print(f"Error in Files section: {e}")

    # 4. AI Test
    print("\n--- 3. AI API ---")
    try:
        ai_headers = HEADERS.copy()
        ai_headers["Content-Type"] = "application/json"
        
        payload = {"message": "Hello, are you working?"}
        res = requests.post(f"{BASE_URL}/ai/chat", headers=ai_headers, json=payload)
        print_result("AI Chat", res.status_code, res.text if res.status_code != 200 else "OK")
        
        if res.status_code == 200:
            print(f"AI Reply: {res.json().get('reply')[:50]}...")

    except Exception as e:
        print(f"Error in AI section: {e}")

    # 5. Budgets Test (Company Wide)
    print("\n--- 4. Budgets API (Company Wide) ---")
    try:
        budget_headers = HEADERS.copy()
        budget_headers["Content-Type"] = "application/json"

        # List
        res = requests.get(f"{BASE_URL}/budgets", headers=budget_headers)
        print_result("List Budgets", res.status_code)
        
        # Create
        new_budget = {
            "category": "Test Category",
            "budget_type": "expense",
            "period": "2026-02",
            "budget_amount": 1000.0,
            "notes": "E2E Test Budget"
        }
        res = requests.post(f"{BASE_URL}/budgets", headers=budget_headers, json=new_budget)
        print_result("Create Budget", res.status_code, res.text if res.status_code != 200 else "OK")

    except Exception as e:
        print(f"Error in Budgets section: {e}")

if __name__ == "__main__":
    main()
