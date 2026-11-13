#!/bin/bash
SESSION="${OPC_SESSION:?set OPC_SESSION to a local test session}"
BASE="http://localhost:8987"

PATHS=(
  "/api/ai_configurations"
  "/api/application_materials"
  "/api/applications"
  "/api/asset_records"
  "/api/assets"
  "/api/bank/accounts"
  "/api/bank/transactions"
  "/api/bank_transactions"
  "/api/branches"
  "/api/budgets"
  "/api/categories"
  "/api/cloud_bills"
  "/api/company_info"
  "/api/contracts"
  "/api/email_configurations"
  "/api/employees"
  "/api/invoices"
  "/api/kpi_configs"
  "/api/kpi_records"
  "/api/licenses"
  "/api/logs"
  "/api/material_library"
  "/api/material_templates"
  "/api/milestones"
  "/api/payments"
  "/api/profiles"
  "/api/projects"
  "/api/rd_costs"
  "/api/reminders"
  "/api/rent"
  "/api/rents"
  "/api/salaries"
  "/api/salary"
  "/api/social_insurance_config"
  "/api/software_subscriptions"
  "/api/tax"
  "/api/tax_declarations"
  "/api/user_roles"
  "/api/utilities"
  "/api/cloud_providers"
  "/api/cloud_resources"
  "/api/domain_subscriptions"
  "/api/data_asset_meta"
  "/api/stats/dashboard"
  "/api/system/status"
)

echo "=== ALL API ENDPOINT STATUS ==="
PASS=0
FAIL=0
EMPTY=0
for path in "${PATHS[@]}"; do
  status=$(curl -s -o /tmp/_api_out.json -w "%{http_code}" "${BASE}${path}" -H "Authorization: Bearer ${SESSION}")
  count=$(python3 -c "
import json
try:
  d=json.load(open('/tmp/_api_out.json'))
  if isinstance(d, dict) and 'data' in d:
    print(len(d['data']))
  elif isinstance(d, list):
    print(len(d))
  else:
    print('obj')
except: print('err')
" 2>/dev/null)

  if [ "$status" = "200" ]; then
    if [ "$count" = "0" ]; then
      echo "⚠️  $status | $count rows | $path"
      EMPTY=$((EMPTY+1))
    else
      echo "✅ $status | $count rows | $path"
    fi
    PASS=$((PASS+1))
  else
    echo "❌ $status | $path"
    FAIL=$((FAIL+1))
  fi
done

echo ""
echo "=== SUMMARY ==="
echo "Total: ${#PATHS[@]}"
echo "Pass: $PASS"
echo "Fail: $FAIL"
echo "Empty (0 rows): $EMPTY"
