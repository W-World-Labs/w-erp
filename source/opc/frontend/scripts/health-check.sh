#!/bin/bash
# OPC - 全端口自动健康检测脚本
# 用法: bash frontend/scripts/health-check.sh [BASE_URL]
# 示例: bash frontend/scripts/health-check.sh http://localhost:3000

set -euo pipefail

BASE_URL="${1:-http://localhost:8987}"
PASS=0
FAIL=0
WARN=0
ERRORS=()

# 颜色
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${CYAN}╔══════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║   OPC - 全自动健康检测                            ║${NC}"
echo -e "${CYAN}║   Base URL: ${BASE_URL}${NC}"
echo -e "${CYAN}╚══════════════════════════════════════════════════╝${NC}"
echo ""

# 检测单个 API 端点
check_endpoint() {
    local endpoint="$1"
    local description="$2"
    local expect_data="${3:-true}"

    local url="${BASE_URL}/api${endpoint}"
    local response
    local http_code

    # 发送请求并获取HTTP状态码和响应体
    response=$(curl -s -w "\n%{http_code}" --connect-timeout 5 --max-time 10 "$url" 2>/dev/null || echo -e "\n000")
    http_code=$(echo "$response" | tail -1)
    body=$(echo "$response" | sed '$d')

    if [ "$http_code" = "000" ]; then
        echo -e "  ${RED}✗ FAIL${NC}  ${endpoint}  (${description}) - 连接超时/失败"
        FAIL=$((FAIL + 1))
        ERRORS+=("${endpoint}: 连接失败")
        return
    fi

    if [ "$http_code" = "200" ]; then
        if [ "$expect_data" = "true" ]; then
            # 检查返回体是否有效 JSON 且包含 data 字段
            local has_data=$(echo "$body" | python3 -c "
import json, sys
try:
    d = json.load(sys.stdin)
    if 'data' in d:
        count = d.get('count', len(d['data']) if isinstance(d['data'], list) else 0)
        print(f'OK:{count}')
    else:
        print('NO_DATA')
except:
    print('BAD_JSON')
" 2>/dev/null || echo "PARSE_ERR")

            if [[ "$has_data" == OK:* ]]; then
                local count="${has_data#OK:}"
                echo -e "  ${GREEN}✓ PASS${NC}  ${endpoint}  (${description}) - ${count} 条记录"
                PASS=$((PASS + 1))
            elif [ "$has_data" = "NO_DATA" ]; then
                echo -e "  ${YELLOW}⚠ WARN${NC}  ${endpoint}  (${description}) - 200 但无 data 字段"
                WARN=$((WARN + 1))
            else
                echo -e "  ${YELLOW}⚠ WARN${NC}  ${endpoint}  (${description}) - 响应格式异常"
                WARN=$((WARN + 1))
            fi
        else
            echo -e "  ${GREEN}✓ PASS${NC}  ${endpoint}  (${description}) - HTTP 200"
            PASS=$((PASS + 1))
        fi
    elif [ "$http_code" = "401" ]; then
        echo -e "  ${YELLOW}⚠ AUTH${NC}  ${endpoint}  (${description}) - 需要登录 (401)"
        WARN=$((WARN + 1))
    elif [ "$http_code" = "404" ]; then
        echo -e "  ${RED}✗ MISS${NC}  ${endpoint}  (${description}) - 端点不存在 (404)"
        FAIL=$((FAIL + 1))
        ERRORS+=("${endpoint}: 404 Not Found")
    else
        echo -e "  ${RED}✗ FAIL${NC}  ${endpoint}  (${description}) - HTTP ${http_code}"
        FAIL=$((FAIL + 1))
        ERRORS+=("${endpoint}: HTTP ${http_code}")
    fi
}

# ===== 1. 后端连通性检查 =====
echo -e "${CYAN}[1/4] 后端连通性检查${NC}"
echo "───────────────────────────────────────"

backend_ok=true
backend_response=$(curl -s -o /dev/null -w "%{http_code}" --connect-timeout 3 "${BASE_URL}/api/company_info" 2>/dev/null || echo "000")
if [ "$backend_response" = "000" ]; then
    echo -e "  ${RED}✗ 后端服务未启动或无法连接 (${BASE_URL})${NC}"
    echo -e "  ${YELLOW}  请先启动后端: cd backend && cargo run${NC}"
    backend_ok=false
else
    echo -e "  ${GREEN}✓ 后端服务连接正常${NC}"
fi
echo ""

if [ "$backend_ok" = false ]; then
    echo -e "${RED}后端不可用，跳过 API 检测${NC}"
    exit 1
fi

# ===== 2. 核心业务 API 端点检查 =====
echo -e "${CYAN}[2/4] 核心业务 API 端点${NC}"
echo "───────────────────────────────────────"

# 公司管理
check_endpoint "/company_info" "公司信息"
check_endpoint "/branches" "分支机构"

# 人事管理
check_endpoint "/employees" "员工管理"
check_endpoint "/salary" "工资管理"
check_endpoint "/contracts" "合同管理"

# 财务管理
check_endpoint "/invoices" "发票管理"
check_endpoint "/budgets" "预算管理"
check_endpoint "/payments" "支付管理"
check_endpoint "/tax" "税务管理"

# 资产管理
check_endpoint "/assets" "固定资产"
check_endpoint "/categories" "资产分类"

# 场地管理
check_endpoint "/rent" "场地租赁"
check_endpoint "/licenses" "证照管理"

# 项目管理
check_endpoint "/projects" "项目管理"
check_endpoint "/milestones" "里程碑"

# KPI 管理
check_endpoint "/kpi" "KPI 指标"
check_endpoint "/kpi_configs" "KPI 配置"
check_endpoint "/kpi_records" "KPI 记录"

# 其他
check_endpoint "/reminders" "提醒管理"
check_endpoint "/logs" "操作日志"
check_endpoint "/rd_costs" "研发成本"
check_endpoint "/data_asset_meta" "数据资产"

echo ""

# ===== 3. 聚合 API 检查 =====
echo -e "${CYAN}[3/4] 聚合统计 API${NC}"
echo "───────────────────────────────────────"

check_endpoint "/stats/dashboard" "仪表盘统计" "false"

echo ""

# ===== 4. 前端页面可访问性检测 =====
echo -e "${CYAN}[4/4] 前端页面可访问性 (端口 8080)${NC}"
echo "───────────────────────────────────────"

FRONTEND_URL="${FRONTEND_URL:-http://localhost:8080}"

pages=(
    "/ 首页/仪表盘"
    "/company-info 公司信息"
    "/branches 分支机构"
    "/employees 员工管理"
    "/salary 工资管理"
    "/invoices 发票管理"
    "/assets 资产管理"
    "/rent 场地租赁"
    "/licenses 证照管理"
    "/projects 项目管理"
    "/budget-management 预算管理"
    "/settings 系统设置"
    "/tax-management 税务管理"
)

frontend_ok=true
frontend_code=$(curl -s -o /dev/null -w "%{http_code}" --connect-timeout 3 "$FRONTEND_URL" 2>/dev/null || echo "000")
if [ "$frontend_code" = "000" ]; then
    echo -e "  ${RED}✗ 前端服务未启动 (${FRONTEND_URL})${NC}"
    frontend_ok=false
else
    echo -e "  ${GREEN}✓ 前端服务连接正常${NC}"
    for page_info in "${pages[@]}"; do
        path=$(echo "$page_info" | awk '{print $1}')
        name=$(echo "$page_info" | awk '{$1=""; print $0}' | sed 's/^ //')
        page_code=$(curl -s -o /dev/null -w "%{http_code}" --connect-timeout 3 "${FRONTEND_URL}${path}" 2>/dev/null || echo "000")
        if [ "$page_code" = "200" ]; then
            echo -e "  ${GREEN}✓ PASS${NC}  ${path}  (${name})"
            PASS=$((PASS + 1))
        else
            echo -e "  ${RED}✗ FAIL${NC}  ${path}  (${name}) - HTTP ${page_code}"
            FAIL=$((FAIL + 1))
        fi
    done
fi

echo ""

# ===== 汇总 =====
echo -e "${CYAN}══════════════════════════════════════════════════${NC}"
TOTAL=$((PASS + FAIL + WARN))
echo -e "  ${GREEN}通过: ${PASS}${NC}  ${RED}失败: ${FAIL}${NC}  ${YELLOW}警告: ${WARN}${NC}  总计: ${TOTAL}"
echo ""

if [ ${#ERRORS[@]} -gt 0 ]; then
    echo -e "${RED}失败详情:${NC}"
    for err in "${ERRORS[@]}"; do
        echo -e "  ${RED}•${NC} ${err}"
    done
    echo ""
fi

if [ "$FAIL" -eq 0 ]; then
    echo -e "${GREEN}🎉 所有检测通过！系统运行正常。${NC}"
    exit 0
else
    echo -e "${RED}⚠️  发现 ${FAIL} 个问题，请检查上述失败项。${NC}"
    exit 1
fi
