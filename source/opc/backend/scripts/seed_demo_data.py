import os
#!/usr/bin/env python3
"""
Seed realistic demo data: 固定资产、虚拟资产、数据资产、无形资产、研发费用
"""
import requests
import json
import uuid
import sys

BASE_URL = "http://localhost:8987/api"
SESSION_ID = os.environ["OPC_SESSION"]
HEADERS = {
    "Authorization": f"Bearer {SESSION_ID}",
    "Content-Type": "application/json",
}

created = {"assets": [], "meta": [], "rd_costs": []}

def post(endpoint, data):
    res = requests.post(f"{BASE_URL}/{endpoint}", headers=HEADERS, json=data)
    ok = 200 <= res.status_code < 300
    name = data.get("name", data.get("cost_type", ""))
    symbol = "✅" if ok else "❌"
    print(f"  {symbol} {endpoint}: {name} [{res.status_code}]")
    if ok:
        rid = res.json().get("id")
        return rid
    else:
        print(f"      {res.text[:120]}")
        return None

def main():
    # 0. Setup
    print("=== 连接 & 切换公司上下文 ===")
    res = requests.get(f"{BASE_URL}/companies", headers=HEADERS)
    if res.status_code != 200:
        print("❌ 无法连接后端"); return
    companies = res.json().get("data", [])
    if not companies:
        print("❌ 无公司"); return
    cid = companies[0]["id"]
    print(f"📌 {companies[0]['name']} ({cid})")
    requests.get(f"{BASE_URL}/companies/{cid}/switch", headers=HEADERS)

    # =============================================
    # 1. 固定资产 (physical)
    # =============================================
    print("\n=== 1. 固定资产 (服务器/电脑/显示器/网络设备) ===")

    physical_assets = [
        {
            "name": "Dell PowerEdge R750 服务器",
            "asset_type": "physical",
            "status": "in_use",
            "serial_number": "SRV-DELL-2024-001",
            "location": "机房A-Rack3-U12",
            "purchase_date": "2024-03-15",
            "purchase_price": 68000.00,
            "current_value": 68000.00,
            "useful_life": 60,
            "salvage_rate": 0.05,
            "depreciation_method": "straight_line",
            "monthly_depreciation": 1076.67,
            "accumulated_depreciation": 11843.33,
            "description": "双路至强 Gold 5418Y / 128GB DDR5 / 4x 1.92TB NVMe SSD / 双电源",
        },
        {
            "name": "Dell PowerEdge R750 服务器 #2",
            "asset_type": "physical",
            "status": "in_use",
            "serial_number": "SRV-DELL-2024-002",
            "location": "机房A-Rack3-U14",
            "purchase_date": "2024-03-15",
            "purchase_price": 68000.00,
            "current_value": 68000.00,
            "useful_life": 60,
            "salvage_rate": 0.05,
            "depreciation_method": "straight_line",
            "monthly_depreciation": 1076.67,
            "accumulated_depreciation": 11843.33,
            "description": "GPU 计算节点 / 双路至强 + 2x NVIDIA A100 80GB / 256GB DDR5",
        },
        {
            "name": "MacBook Pro 16\" M3 Max",
            "asset_type": "physical",
            "status": "in_use",
            "serial_number": "MBP-M3MAX-001",
            "location": "研发部-工位A12",
            "purchase_date": "2024-01-20",
            "purchase_price": 29999.00,
            "current_value": 29999.00,
            "useful_life": 36,
            "salvage_rate": 0.10,
            "depreciation_method": "straight_line",
            "monthly_depreciation": 749.98,
            "accumulated_depreciation": 9749.70,
            "description": "M3 Max 36核GPU / 48GB / 1TB SSD / 研发主力机",
        },
        {
            "name": "MacBook Pro 14\" M3 Pro",
            "asset_type": "physical",
            "status": "in_use",
            "serial_number": "MBP-M3PRO-002",
            "location": "研发部-工位A15",
            "purchase_date": "2024-02-10",
            "purchase_price": 18999.00,
            "current_value": 18999.00,
            "useful_life": 36,
            "salvage_rate": 0.10,
            "depreciation_method": "straight_line",
            "monthly_depreciation": 474.98,
            "accumulated_depreciation": 5699.70,
            "description": "M3 Pro 18核GPU / 36GB / 512GB SSD / 前端开发",
        },
        {
            "name": "Mac Mini M2 Pro",
            "asset_type": "physical",
            "status": "in_use",
            "serial_number": "MMINI-M2-003",
            "location": "机房B-CI/CD架",
            "purchase_date": "2023-08-01",
            "purchase_price": 9999.00,
            "current_value": 9999.00,
            "useful_life": 48,
            "salvage_rate": 0.05,
            "depreciation_method": "straight_line",
            "monthly_depreciation": 197.90,
            "accumulated_depreciation": 3561.15,
            "description": "CI/CD 构建服务器 / macOS Runner / 32GB / 512GB SSD",
        },
        {
            "name": "Dell UltraSharp U2723QE 4K 27\"",
            "asset_type": "physical",
            "status": "in_use",
            "serial_number": "MON-DELL-001",
            "location": "研发部-工位A12",
            "purchase_date": "2024-01-20",
            "purchase_price": 4299.00,
            "current_value": 4299.00,
            "useful_life": 60,
            "salvage_rate": 0.05,
            "depreciation_method": "straight_line",
            "monthly_depreciation": 68.07,
            "accumulated_depreciation": 884.88,
            "description": "4K IPS / 90W USB-C / HDR400 / 外接显示器",
        },
        {
            "name": "Dell UltraSharp U2723QE 4K 27\" #2",
            "asset_type": "physical",
            "status": "in_use",
            "serial_number": "MON-DELL-002",
            "location": "研发部-工位A15",
            "purchase_date": "2024-02-10",
            "purchase_price": 4299.00,
            "current_value": 4299.00,
            "useful_life": 60,
            "salvage_rate": 0.05,
            "depreciation_method": "straight_line",
            "monthly_depreciation": 68.07,
            "accumulated_depreciation": 816.81,
            "description": "4K IPS / 90W USB-C / HDR400 / 外接显示器",
        },
        {
            "name": "Ubiquiti Dream Machine Pro",
            "asset_type": "physical",
            "status": "in_use",
            "serial_number": "NET-UDM-001",
            "location": "机房A-网络柜",
            "purchase_date": "2023-06-01",
            "purchase_price": 5200.00,
            "current_value": 5200.00,
            "useful_life": 60,
            "salvage_rate": 0.05,
            "depreciation_method": "straight_line",
            "monthly_depreciation": 82.33,
            "accumulated_depreciation": 2552.33,
            "description": "企业路由 / IDS/IPS / VPN / 10G SFP+ / 全网管控",
        },
        {
            "name": "Synology DS1823xs+ NAS",
            "asset_type": "physical",
            "status": "in_use",
            "serial_number": "NAS-SYN-001",
            "location": "机房A-Rack2-U6",
            "purchase_date": "2024-05-20",
            "purchase_price": 18500.00,
            "current_value": 18500.00,
            "useful_life": 60,
            "salvage_rate": 0.05,
            "depreciation_method": "straight_line",
            "monthly_depreciation": 292.92,
            "accumulated_depreciation": 2636.25,
            "description": "8盘位 / 8x 16TB IronWolf Pro / RAID6 / 数据备份与归档",
        },
        {
            "name": "Kingston 64GB DDR5 ECC 内存条 (4条)",
            "asset_type": "physical",
            "status": "in_use",
            "serial_number": "MEM-KST-001",
            "location": "机房A-Rack3-U12 (服务器内)",
            "purchase_date": "2024-03-15",
            "purchase_price": 8800.00,
            "current_value": 8800.00,
            "useful_life": 60,
            "salvage_rate": 0.05,
            "depreciation_method": "straight_line",
            "monthly_depreciation": 139.33,
            "accumulated_depreciation": 1532.67,
            "description": "4x 64GB DDR5-4800 ECC RDIMM / 服务器扩容内存",
        },
        {
            "name": "APC Smart-UPS 3000VA",
            "asset_type": "physical",
            "status": "in_use",
            "serial_number": "UPS-APC-001",
            "location": "机房A-电源柜",
            "purchase_date": "2023-06-01",
            "purchase_price": 12000.00,
            "current_value": 12000.00,
            "useful_life": 60,
            "salvage_rate": 0.05,
            "depreciation_method": "straight_line",
            "monthly_depreciation": 190.00,
            "accumulated_depreciation": 5890.00,
            "description": "在线式 UPS / 3000VA/2700W / 8插口 / 网络管理卡",
        },
        {
            "name": "惠普 LaserJet Pro MFP 4101fdw",
            "asset_type": "physical",
            "status": "in_use",
            "serial_number": "PRT-HP-001",
            "location": "行政部-打印区",
            "purchase_date": "2024-06-01",
            "purchase_price": 3599.00,
            "current_value": 3599.00,
            "useful_life": 48,
            "salvage_rate": 0.05,
            "depreciation_method": "straight_line",
            "monthly_depreciation": 71.23,
            "accumulated_depreciation": 569.84,
            "description": "激光多功能一体机 / 打印/扫描/复印/传真 / WiFi / 双面打印",
        },
    ]

    for asset in physical_assets:
        aid = post("assets", asset)
        if aid: created["assets"].append(aid)

    # =============================================
    # 2. 虚拟资产 (virtual) — 订阅/域名/云服务
    # =============================================
    print("\n=== 2. 虚拟资产 (订阅服务/域名/云服务) ===")

    virtual_assets = [
        {
            "name": "Apple Developer Program 年费",
            "asset_type": "virtual",
            "status": "in_use",
            "purchase_date": "2024-06-15",
            "purchase_price": 688.00,
            "current_value": 688.00,
            "expiry_date": "2025-06-15",
            "registration_number": "APPLE-DEV-2024",
            "holder_name": "黑盒智能实验室",
            "description": "Apple Developer Program / iOS & macOS 应用发布 / TestFlight",
        },
        {
            "name": "Apple One 高级版 (家庭)",
            "asset_type": "virtual",
            "status": "in_use",
            "purchase_date": "2024-01-01",
            "purchase_price": 2388.00,
            "current_value": 2388.00,
            "expiry_date": "2025-01-01",
            "description": "iCloud+ 2TB / Apple Music / TV+ / Arcade / Fitness+ / News+ / 年付",
        },
        {
            "name": "Cloudflare Pro Plan",
            "asset_type": "virtual",
            "status": "in_use",
            "purchase_date": "2024-01-01",
            "purchase_price": 1680.00,
            "current_value": 1680.00,
            "expiry_date": "2025-01-01",
            "registration_number": "CF-PRO-2024",
            "holder_name": "黑盒智能实验室",
            "description": "Cloudflare Pro / WAF / DDoS 防护 / CDN / Workers 10万次/天 / 年付 $20/月",
        },
        {
            "name": "Cloudflare Workers Paid",
            "asset_type": "virtual",
            "status": "in_use",
            "purchase_date": "2024-01-01",
            "purchase_price": 360.00,
            "current_value": 360.00,
            "expiry_date": "2025-01-01",
            "description": "Workers Paid / 1000万请求/月 / KV / R2 / D1 / 年付 $5/月",
        },
        {
            "name": "GitHub Team 订阅",
            "asset_type": "virtual",
            "status": "in_use",
            "purchase_date": "2024-01-01",
            "purchase_price": 2016.00,
            "current_value": 2016.00,
            "expiry_date": "2025-01-01",
            "description": "GitHub Team / 6 seats / Actions 3000 min/月 / Packages / Codespaces / 年付",
        },
        {
            "name": "Figma Professional 订阅",
            "asset_type": "virtual",
            "status": "in_use",
            "purchase_date": "2024-03-01",
            "purchase_price": 864.00,
            "current_value": 864.00,
            "expiry_date": "2025-03-01",
            "description": "Figma Professional / 3 editor seats / 无限项目 / 版本历史 / 年付 $12/seat/月",
        },
        {
            "name": "Notion Team 订阅",
            "asset_type": "virtual",
            "status": "in_use",
            "purchase_date": "2024-01-01",
            "purchase_price": 576.00,
            "current_value": 576.00,
            "expiry_date": "2025-01-01",
            "description": "Notion Team / 6 members / 无限blocks / SAML SSO / 年付 $8/member/月",
        },
        {
            "name": "域名 blackbox.ai",
            "asset_type": "virtual",
            "status": "in_use",
            "purchase_date": "2023-11-15",
            "purchase_price": 580.00,
            "current_value": 580.00,
            "expiry_date": "2025-11-15",
            "registration_number": "D-BB-AI-2023",
            "holder_name": "黑盒智能实验室",
            "description": ".ai 域名 / Cloudflare 注册 / 2年续费",
        },
        {
            "name": "域名 http.vg",
            "asset_type": "virtual",
            "status": "in_use",
            "purchase_date": "2024-02-01",
            "purchase_price": 320.00,
            "current_value": 320.00,
            "expiry_date": "2026-02-01",
            "registration_number": "D-HTTP-VG-2024",
            "holder_name": "黑盒智能实验室",
            "description": ".vg 域名 / Star 隧道短域名 / 2年续费",
        },
        {
            "name": "域名 obscura.app",
            "asset_type": "virtual",
            "status": "in_use",
            "purchase_date": "2024-04-10",
            "purchase_price": 120.00,
            "current_value": 120.00,
            "expiry_date": "2025-04-10",
            "registration_number": "D-OBSCURA-APP-2024",
            "holder_name": "黑盒智能实验室",
            "description": ".app 域名 / Cloudflare 注册 / 自动续费",
        },
        {
            "name": "1Password Business 订阅",
            "asset_type": "virtual",
            "status": "in_use",
            "purchase_date": "2024-01-01",
            "purchase_price": 2304.00,
            "current_value": 2304.00,
            "expiry_date": "2025-01-01",
            "description": "1Password Business / 6 members / SSO / Advanced protection / 年付 $7.99/user/月",
        },
        {
            "name": "阿里云 ECS 年付 (生产环境)",
            "asset_type": "virtual",
            "status": "in_use",
            "purchase_date": "2024-04-01",
            "purchase_price": 9600.00,
            "current_value": 9600.00,
            "expiry_date": "2025-04-01",
            "registration_number": "ALIYUN-ECS-PROD-001",
            "description": "ecs.g7.2xlarge / 8vCPU 32GB / 500GB ESSD / 固定公网IP / 年付",
        },
    ]

    for asset in virtual_assets:
        aid = post("assets", asset)
        if aid: created["assets"].append(aid)

    # =============================================
    # 3. 数据资产 (data) — 软件/数据集
    # =============================================
    print("\n=== 3. 数据资产 (自研软件/数据集) ===")

    data_assets_and_meta = [
        {
            "asset": {
                "name": "Asset Nexus ERP 系统",
                "asset_type": "data",
                "status": "in_use",
                "purchase_price": 0,
                "current_value": 150000.00,
                "purchase_date": "2024-01-01",
                "description": "企业资源管理系统 / React + Rust (Axum) + SQLite / 自研",
            },
            "meta": {
                "data_type": "software",
                "tech_stack": "React 18, TypeScript, Vite, Rust, Axum, SQLite, TailwindCSS",
                "version": "v2.0.0",
                "license_type": "proprietary",
                "access_url": "https://github.com/example/opc",
            },
        },
        {
            "asset": {
                "name": "Star 隧道系统",
                "asset_type": "data",
                "status": "in_use",
                "purchase_price": 0,
                "current_value": 200000.00,
                "purchase_date": "2023-06-01",
                "description": "安全端口隧道 / 多协议转发 / Cloudflare Durable Objects / WebSocket Relay",
            },
            "meta": {
                "data_type": "software",
                "tech_stack": "Rust, Tokio, Cloudflare Workers, TypeScript, React",
                "version": "v4.2.0",
                "license_type": "proprietary",
                "access_url": "https://github.com/blackbox/star",
            },
        },
        {
            "asset": {
                "name": "Radar 行为分析平台",
                "asset_type": "data",
                "status": "in_use",
                "purchase_price": 0,
                "current_value": 120000.00,
                "purchase_date": "2024-01-15",
                "description": "遥测/审计/行为分析 / PV/Click/Scroll SDK / 非侵入式采集",
            },
            "meta": {
                "data_type": "software",
                "tech_stack": "Rust, Axum, JavaScript SDK, React Dashboard",
                "version": "v2026.1",
                "license_type": "proprietary",
            },
        },
        {
            "asset": {
                "name": "多语种艺术图文数据集",
                "asset_type": "data",
                "status": "in_use",
                "purchase_price": 50000.00,
                "current_value": 50000.00,
                "purchase_date": "2024-10-15",
                "description": "3M+ 多语种艺术作品图文对 / 涵盖11种小语种 / 无AI合成",
            },
            "meta": {
                "data_type": "dataset",
                "data_volume": "3,200,000 对",
                "data_format": "JSON + JPEG",
            },
        },
        {
            "asset": {
                "name": "NetProtect 网络安全套件",
                "asset_type": "data",
                "status": "in_use",
                "purchase_price": 0,
                "current_value": 80000.00,
                "purchase_date": "2024-06-01",
                "description": "跨平台网络流量监控 & 防火墙 / pcap + WinTUN / Tauri 2.0",
            },
            "meta": {
                "data_type": "software",
                "tech_stack": "Rust, Tauri 2.0, React, libpcap, WinTUN, SQLite",
                "version": "v1.3.0",
                "license_type": "proprietary",
            },
        },
    ]

    for item in data_assets_and_meta:
        aid = post("assets", item["asset"])
        if aid:
            created["assets"].append(aid)
            meta = item["meta"].copy()
            meta["asset_id"] = aid
            mid = post("data_asset_meta", meta)
            if mid: created["meta"].append(mid)

    # =============================================
    # 4. 无形资产 (intangible) — 专利/商标
    # =============================================
    print("\n=== 4. 无形资产 (专利/商标/著作权) ===")

    intangible_assets = [
        {
            "asset": {
                "name": "发明专利: 基于行为分析的异常检测方法",
                "asset_type": "intangible",
                "status": "in_use",
                "purchase_price": 15000.00,
                "current_value": 15000.00,
                "purchase_date": "2024-03-20",
                "registration_number": "CN202410123456.7",
                "holder_name": "黑盒智能实验室",
                "expiry_date": "2044-03-20",
                "description": "发明专利 / 基于深度学习的用户行为异常检测方法与系统",
            },
            "meta": {
                "data_type": "patent",
                "patent_number": "CN202410123456.7",
                "patent_type": "invention",
            },
        },
        {
            "asset": {
                "name": "软件著作权: Asset Nexus 企业管理系统",
                "asset_type": "intangible",
                "status": "in_use",
                "purchase_price": 2000.00,
                "current_value": 2000.00,
                "purchase_date": "2024-08-15",
                "registration_number": "2024SR1234567",
                "holder_name": "黑盒智能实验室",
                "description": "计算机软件著作权 / 登记号: 2024SR1234567",
            },
            "meta": {
                "data_type": "copyright",
            },
        },
        {
            "asset": {
                "name": "商标: Obscura Intelligence Lab",
                "asset_type": "intangible",
                "status": "in_use",
                "purchase_price": 5000.00,
                "current_value": 5000.00,
                "purchase_date": "2024-02-01",
                "registration_number": "TM-2024-OBSCURA",
                "holder_name": "黑盒智能实验室",
                "expiry_date": "2034-02-01",
                "description": "注册商标 / 第9类(计算机软件) + 第42类(技术服务)",
            },
            "meta": {
                "data_type": "trademark",
                "trademark_class": "第9类, 第42类",
            },
        },
    ]

    for item in intangible_assets:
        aid = post("assets", item["asset"])
        if aid:
            created["assets"].append(aid)
            meta = item["meta"].copy()
            meta["asset_id"] = aid
            mid = post("data_asset_meta", meta)
            if mid: created["meta"].append(mid)

    # =============================================
    # 5. 研发费用
    # =============================================
    print("\n=== 5. 研发费用 (按月归集) ===")

    # 找到一个项目
    res = requests.get(f"{BASE_URL}/projects", headers=HEADERS)
    projects = res.json().get("data", []) if res.status_code == 200 else []
    if not projects:
        print("⚠️ 无项目，跳过研发费用")
    else:
        pid = projects[0]["id"]
        pname = projects[0].get("name", "")
        print(f"  📌 项目: {pname} ({pid})")

        rd_costs = [
            # 2024年1-6月人员费用
            {"cost_type": "personnel", "amount": 45000, "cost_date": "2024-01-31", "description": "1月研发人员工资 (3人)", "project_id": pid},
            {"cost_type": "personnel", "amount": 45000, "cost_date": "2024-02-29", "description": "2月研发人员工资 (3人)", "project_id": pid},
            {"cost_type": "personnel", "amount": 48000, "cost_date": "2024-03-31", "description": "3月研发人员工资+绩效 (3人)", "project_id": pid},
            {"cost_type": "personnel", "amount": 45000, "cost_date": "2024-04-30", "description": "4月研发人员工资 (3人)", "project_id": pid},
            {"cost_type": "personnel", "amount": 45000, "cost_date": "2024-05-31", "description": "5月研发人员工资 (3人)", "project_id": pid},
            {"cost_type": "personnel", "amount": 50000, "cost_date": "2024-06-30", "description": "6月研发人员工资+季度奖 (3人)", "project_id": pid},
            # 设备费用
            {"cost_type": "equipment", "amount": 68000, "cost_date": "2024-03-15", "description": "Dell R750 GPU 服务器采购", "project_id": pid},
            {"cost_type": "equipment", "amount": 29999, "cost_date": "2024-01-20", "description": "MacBook Pro 16\" M3 Max 采购", "project_id": pid},
            {"cost_type": "equipment", "amount": 8800, "cost_date": "2024-03-15", "description": "DDR5 ECC 内存扩容 (4x64GB)", "project_id": pid},
            # 材料费用
            {"cost_type": "material", "amount": 3500, "cost_date": "2024-02-15", "description": "网线/光纤跳线/理线器等布线材料", "project_id": pid},
            {"cost_type": "material", "amount": 1200, "cost_date": "2024-04-10", "description": "硬盘托架/散热垫/螺丝等配件", "project_id": pid},
            # 外协费用
            {"cost_type": "outsourcing", "amount": 50000, "cost_date": "2024-10-15", "description": "多语种数据标注外包 (第一批)", "project_id": pid},
            {"cost_type": "outsourcing", "amount": 15000, "cost_date": "2024-05-20", "description": "安全渗透测试服务", "project_id": pid},
            # 折旧费用 (按月分摊)
            {"cost_type": "depreciation", "amount": 1076.67, "cost_date": "2024-04-30", "description": "R750服务器折旧-4月", "project_id": pid},
            {"cost_type": "depreciation", "amount": 1076.67, "cost_date": "2024-05-31", "description": "R750服务器折旧-5月", "project_id": pid},
            {"cost_type": "depreciation", "amount": 1076.67, "cost_date": "2024-06-30", "description": "R750服务器折旧-6月", "project_id": pid},
            # 其他费用
            {"cost_type": "other", "amount": 5800, "cost_date": "2024-03-25", "description": "ICLR 2024 学术会议注册+差旅", "project_id": pid},
            {"cost_type": "other", "amount": 2400, "cost_date": "2024-05-10", "description": "AWS/Azure GPU 临时算力租赁", "project_id": pid},
            {"cost_type": "other", "amount": 1500, "cost_date": "2024-06-15", "description": "技术书籍+在线课程采购", "project_id": pid},
        ]

        for cost in rd_costs:
            cid = post("rd_costs", cost)
            if cid: created["rd_costs"].append(cid)

    # =============================================
    # Summary
    # =============================================
    print("\n" + "=" * 60)
    print(f"🎉 Demo 数据录入完成！")
    print(f"   资产: {len(created['assets'])} 条")
    print(f"   元数据: {len(created['meta'])} 条")
    print(f"   研发费用: {len(created['rd_costs'])} 条")
    print("=" * 60)

    # Verify totals
    print("\n--- 验证统计 ---")
    res = requests.get(f"{BASE_URL}/assets", headers=HEADERS)
    if res.status_code == 200:
        all_assets = res.json().get("data", [])
        by_type = {}
        total_value = 0
        for a in all_assets:
            t = a.get("asset_type", "unknown")
            by_type[t] = by_type.get(t, 0) + 1
            total_value += a.get("current_value", 0)
        print(f"  资产总数: {len(all_assets)}")
        for t, c in sorted(by_type.items()):
            print(f"    {t}: {c} 项")
        print(f"  资产总值: ¥{total_value:,.2f}")

    res = requests.get(f"{BASE_URL}/rd_costs", headers=HEADERS)
    if res.status_code == 200:
        costs = res.json().get("data", [])
        total_rd = sum(c.get("amount", 0) for c in costs)
        print(f"  研发费用总额: ¥{total_rd:,.2f}")

if __name__ == "__main__":
    main()
