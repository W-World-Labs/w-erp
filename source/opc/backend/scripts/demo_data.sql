-- Synthetic demonstration records; personal/account identifiers are deliberately invalid.
-- Demo data for OPC (正确的表结构)

-- 1. 更新公司名为 demo公司
UPDATE companies SET 
  name = 'demo公司',
  legal_name = '演示科技有限公司',
  tax_id = '91110108MA01ABCD1X',
  registration_number = '91110108MA01ABCD1X',
  address = '北京市海淀区中关村大街1号',
  city = '北京',
  province = '北京',
  postal_code = '100080',
  phone = '010-88888888',
  email = 'contact@demo-company-com.example.invalid',
  website = 'https://demo-company.com',
  industry = '科技',
  founded_date = '2020-01-15',
  description = 'demo公司是一家专注于企业管理软件的科技公司'
WHERE id = (SELECT id FROM companies LIMIT 1);

-- 2. 员工数据
INSERT OR REPLACE INTO employees (id, company_id, name, email, phone, department, position, hire_date, status, created_at, updated_at)
SELECT 'emp_001', id, '张三', 'zhangsan@demo-com.example.invalid', '00000138001', '技术部', '技术总监', '2020-03-01', 'active', datetime('now'), datetime('now') FROM companies LIMIT 1;

INSERT OR REPLACE INTO employees (id, company_id, name, email, phone, department, position, hire_date, status, created_at, updated_at)
SELECT 'emp_002', id, '李四', 'lisi@demo-com.example.invalid', '00000138002', '财务部', '财务经理', '2020-04-15', 'active', datetime('now'), datetime('now') FROM companies LIMIT 1;

INSERT OR REPLACE INTO employees (id, company_id, name, email, phone, department, position, hire_date, status, created_at, updated_at)
SELECT 'emp_003', id, '王五', 'wangwu@demo-com.example.invalid', '00000138003', '销售部', '销售主管', '2021-01-10', 'active', datetime('now'), datetime('now') FROM companies LIMIT 1;

INSERT OR REPLACE INTO employees (id, company_id, name, email, phone, department, position, hire_date, status, created_at, updated_at)
SELECT 'emp_004', id, '赵六', 'zhaoliu@demo-com.example.invalid', '00000138004', '技术部', '高级工程师', '2021-06-01', 'active', datetime('now'), datetime('now') FROM companies LIMIT 1;

INSERT OR REPLACE INTO employees (id, company_id, name, email, phone, department, position, hire_date, status, created_at, updated_at)
SELECT 'emp_005', id, '孙七', 'sunqi@demo-com.example.invalid', '00000138005', '人事部', '人事专员', '2022-02-20', 'active', datetime('now'), datetime('now') FROM companies LIMIT 1;

-- 3. 资产数据 (asset_type 只能是 physical 或 virtual)
INSERT OR REPLACE INTO assets (id, company_id, name, asset_type, status, description, serial_number, location, purchase_date, purchase_price, current_value, created_at, updated_at)
SELECT 'ast_001', id, 'MacBook Pro 16寸', 'physical', 'active', '技术部办公电脑', 'MBP2023001', '技术部', '2023-06-15', 24999, 20000, datetime('now'), datetime('now') FROM companies LIMIT 1;

INSERT OR REPLACE INTO assets (id, company_id, name, asset_type, status, description, serial_number, location, purchase_date, purchase_price, current_value, created_at, updated_at)
SELECT 'ast_002', id, '戴尔服务器', 'physical', 'active', '机房主服务器', 'DELL2023001', '机房', '2023-03-10', 58000, 50000, datetime('now'), datetime('now') FROM companies LIMIT 1;

INSERT OR REPLACE INTO assets (id, company_id, name, asset_type, status, description, serial_number, location, purchase_date, purchase_price, current_value, created_at, updated_at)
SELECT 'ast_003', id, '会议室投影仪', 'physical', 'active', '大会议室投影设备', 'PJ2022001', '大会议室', '2022-08-20', 8500, 6000, datetime('now'), datetime('now') FROM companies LIMIT 1;

INSERT OR REPLACE INTO assets (id, company_id, name, asset_type, status, description, serial_number, location, purchase_date, purchase_price, current_value, created_at, updated_at)
SELECT 'ast_004', id, '云服务器套餐', 'virtual', 'active', '阿里云ECS年付', 'ALIYUN2026', '云端', '2026-01-01', 36000, 30000, datetime('now'), datetime('now') FROM companies LIMIT 1;

-- 4. 发票数据
INSERT OR REPLACE INTO invoices (id, company_id, invoice_number, invoice_type, amount, tax_amount, status, invoice_date, due_date, supplier, description, created_at, updated_at)
SELECT 'inv_001', id, 'INV2026001', '增值税专用发票', 100000, 13000, 'paid', '2026-01-15', '2026-02-15', '阿里云计算有限公司', '云服务器年费', datetime('now'), datetime('now') FROM companies LIMIT 1;

INSERT OR REPLACE INTO invoices (id, company_id, invoice_number, invoice_type, amount, tax_amount, status, invoice_date, due_date, supplier, description, created_at, updated_at)
SELECT 'inv_002', id, 'INV2026002', '增值税普通发票', 8500, 765, 'pending', '2026-01-20', '2026-02-20', '北京物业管理有限公司', '2月份物业费', datetime('now'), datetime('now') FROM companies LIMIT 1;

INSERT OR REPLACE INTO invoices (id, company_id, invoice_number, invoice_type, amount, tax_amount, status, invoice_date, due_date, supplier, description, created_at, updated_at)
SELECT 'inv_003', id, 'INV2026003', '增值税专用发票', 25000, 3250, 'pending', '2026-02-01', '2026-03-01', '联想集团', '办公电脑采购', datetime('now'), datetime('now') FROM companies LIMIT 1;

-- 5. 预算数据
INSERT OR REPLACE INTO budgets (id, company_id, category, budget_type, period, budget_amount, actual_amount, status, notes, created_at, updated_at)
SELECT 'bgt_001', id, '技术', 'quarterly', '2026-Q1', 200000, 85000, 'active', '技术部研发预算', datetime('now'), datetime('now') FROM companies LIMIT 1;

INSERT OR REPLACE INTO budgets (id, company_id, category, budget_type, period, budget_amount, actual_amount, status, notes, created_at, updated_at)
SELECT 'bgt_002', id, '市场', 'quarterly', '2026-Q1', 150000, 45000, 'active', '市场推广预算', datetime('now'), datetime('now') FROM companies LIMIT 1;

INSERT OR REPLACE INTO budgets (id, company_id, category, budget_type, period, budget_amount, actual_amount, status, notes, created_at, updated_at)
SELECT 'bgt_003', id, '行政', 'quarterly', '2026-Q1', 80000, 32000, 'active', '日常行政开支', datetime('now'), datetime('now') FROM companies LIMIT 1;

-- 6. 银行账户
INSERT OR REPLACE INTO bank_accounts (id, company_id, bank_name, account_name, account_number, account_type, balance, currency, status, notes, created_at, updated_at)
SELECT 'bnk_001', id, '中国工商银行', '基本存款账户', '0002021234567890123', 'checking', 2850000, 'CNY', 'active', '公司主账户', datetime('now'), datetime('now') FROM companies LIMIT 1;

INSERT OR REPLACE INTO bank_accounts (id, company_id, bank_name, account_name, account_number, account_type, balance, currency, status, notes, created_at, updated_at)
SELECT 'bnk_002', id, '中国建设银行', '一般存款账户', '0008481234567890456', 'checking', 580000, 'CNY', 'active', '备用账户', datetime('now'), datetime('now') FROM companies LIMIT 1;

-- 7. 提醒事项
INSERT OR REPLACE INTO reminders (id, company_id, title, description, reminder_type, priority, due_date, is_completed, entity_type, created_at, updated_at)
SELECT 'rmd_001', id, '软件企业认证续期', '软件企业认证将于5月到期，需提前准备续期材料', 'license', 'high', '2026-04-01', 0, 'license', datetime('now'), datetime('now') FROM companies LIMIT 1;

INSERT OR REPLACE INTO reminders (id, company_id, title, description, reminder_type, priority, due_date, is_completed, entity_type, created_at, updated_at)
SELECT 'rmd_002', id, '季度财务报表', 'Q1季度财务报表编制', 'financial', 'high', '2026-04-15', 0, 'report', datetime('now'), datetime('now') FROM companies LIMIT 1;

INSERT OR REPLACE INTO reminders (id, company_id, title, description, reminder_type, priority, due_date, is_completed, entity_type, created_at, updated_at)
SELECT 'rmd_003', id, '员工体检安排', '年度员工健康体检', 'general', 'medium', '2026-03-15', 0, 'hr', datetime('now'), datetime('now') FROM companies LIMIT 1;

-- 8. KPI 指标
INSERT OR REPLACE INTO kpi_metrics (id, company_id, name, category, value, target, unit, period, description, created_at, updated_at)
SELECT 'kpi_001', id, '月度营收', '财务', 420000, 500000, '元', '2026-01', '月度销售收入', datetime('now'), datetime('now') FROM companies LIMIT 1;

INSERT OR REPLACE INTO kpi_metrics (id, company_id, name, category, value, target, unit, period, description, created_at, updated_at)
SELECT 'kpi_002', id, '客户满意度', '服务', 92, 95, '%', '2026-01', '客户满意度调查', datetime('now'), datetime('now') FROM companies LIMIT 1;

INSERT OR REPLACE INTO kpi_metrics (id, company_id, name, category, value, target, unit, period, description, created_at, updated_at)
SELECT 'kpi_003', id, '项目交付率', '运营', 85, 100, '%', '2026-01', '按时交付的项目比例', datetime('now'), datetime('now') FROM companies LIMIT 1;

-- 9. 税务记录
INSERT OR REPLACE INTO tax_records (id, company_id, tax_type, period, taxable_amount, tax_amount, status, due_date, payment_date, notes, created_at, updated_at)
SELECT 'tax_001', id, '增值税', '2026-01', 350000, 45500, 'paid', '2026-02-15', '2026-02-10', '1月增值税申报', datetime('now'), datetime('now') FROM companies LIMIT 1;

INSERT OR REPLACE INTO tax_records (id, company_id, tax_type, period, taxable_amount, tax_amount, status, due_date, notes, created_at, updated_at)
SELECT 'tax_002', id, '企业所得税', '2026-Q1', 500000, 125000, 'pending', '2026-04-20', 'Q1企业所得税预缴', datetime('now'), datetime('now') FROM companies LIMIT 1;

INSERT OR REPLACE INTO tax_records (id, company_id, tax_type, period, taxable_amount, tax_amount, status, due_date, payment_date, notes, created_at, updated_at)
SELECT 'tax_003', id, '个人所得税', '2026-01', 125000, 8500, 'paid', '2026-02-15', '2026-02-12', '1月员工个税代扣', datetime('now'), datetime('now') FROM companies LIMIT 1;

-- 完成提示
SELECT '✅ Demo数据已成功导入！' as message;
