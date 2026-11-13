-- ============================================================================
-- OPC 全球政策种子数据 (27条)
-- 用法: sqlite3 opc.db < sql/seed_policies.sql
-- 注意: 会先清空现有政策数据再插入
-- ============================================================================

DELETE FROM policies;

INSERT INTO policies (id, name, name_en, country, region, category, provider, description, eligibility, benefits, url, cycle, amount_min, amount_max, currency, equity_required, match_industries, match_countries, match_min_revenue, match_max_employees, match_company_age_max, match_founder_education, match_founder_is_expat, match_founder_is_overseas_returnee, is_active)
VALUES
-- ═══════════════════ 🇺🇸 US Policies ═══════════════════
(
  lower(hex(randomblob(4)) || '-' || hex(randomblob(2)) || '-4' || substr(hex(randomblob(2)),2) || '-' || substr('89ab',abs(random()) % 4 + 1, 1) || substr(hex(randomblob(2)),2) || '-' || hex(randomblob(6))),
  'Y Combinator', NULL, 'US', NULL, 'accelerator', 'Y Combinator',
  '全球顶级创业加速器，提供资金、导师辅导和全球网络。3个月密集培训，Demo Day面向全球投资人。已孵化Stripe、Airbnb、Dropbox等明星公司。',
  '1. 公司处于早期阶段（种子/Pre-A轮）
2. 创始团队至少2人
3. 全球任何国家的公司均可申请
4. 需全职投入
5. 培训期间需搬至旧金山湾区',
  '1. 投资50万美元（12.5万标准+37.5万MFN SAFE）
2. 换取7%股权
3. 3个月密集培训+导师辅导
4. YC校友网络（全球8000+创始人）
5. Demo Day面向顶级投资人
6. AWS/Google Cloud等合作伙伴福利（价值超50万美元）',
  'https://www.ycombinator.com/apply',
  '每年两批（春季3月/秋季9月）',
  500000.0, NULL, 'USD', 0.07,
  '["tech","biotech","fintech","ai"]', '["*"]',
  NULL, NULL, NULL, NULL, NULL, NULL, 1
),
(
  lower(hex(randomblob(4)) || '-' || hex(randomblob(2)) || '-4' || substr(hex(randomblob(2)),2) || '-' || substr('89ab',abs(random()) % 4 + 1, 1) || substr(hex(randomblob(2)),2) || '-' || hex(randomblob(6))),
  'Techstars', NULL, 'US', NULL, 'accelerator', 'Techstars',
  '全球化加速器网络，在40+城市运营。提供导师驱动的3个月加速项目，专注于特定垂直领域（金融科技、健康科技、零售科技等）。',
  '1. 早期创业公司
2. 有初步产品或原型
3. 全球任何国家可申请
4. 创始团队需全职参与
5. 需到指定城市参加培训',
  '1. 投资12万美元（可转债）
2. 换取约6%股权
3. 3个月导师驱动加速
4. 全球Techstars校友网络
5. 超过300位导师资源
6. Demo Day + 投资人对接',
  'https://www.techstars.com/accelerators',
  '全年滚动（各城市时间不同）',
  120000.0, NULL, 'USD', 0.06,
  '["tech"]', '["*"]',
  NULL, NULL, NULL, NULL, NULL, NULL, 1
),
(
  lower(hex(randomblob(4)) || '-' || hex(randomblob(2)) || '-4' || substr(hex(randomblob(2)),2) || '-' || substr('89ab',abs(random()) % 4 + 1, 1) || substr(hex(randomblob(2)),2) || '-' || hex(randomblob(6))),
  '500 Global', NULL, 'US', NULL, 'accelerator', '500 Global',
  '全球最活跃的早期风投之一，已投资80+国家2800+公司。提供4个月加速项目，聚焦新兴市场创业者。',
  '1. 有初步产品且有一定用户/收入
2. 全球任何国家可申请
3. 优先新兴市场创业者
4. 创始团队需全职投入',
  '1. 投资15万美元
2. 换取约6%股权
3. 4个月加速培训
4. 全球导师和投资人网络
5. 后续融资对接',
  'https://500.co/accelerators',
  '每年2-3批',
  150000.0, NULL, 'USD', 0.06,
  NULL, '["*"]',
  NULL, NULL, NULL, NULL, NULL, NULL, 1
),
(
  lower(hex(randomblob(4)) || '-' || hex(randomblob(2)) || '-4' || substr(hex(randomblob(2)),2) || '-' || substr('89ab',abs(random()) % 4 + 1, 1) || substr(hex(randomblob(2)),2) || '-' || hex(randomblob(6))),
  'SBIR Phase I', NULL, 'US', NULL, 'government_grant', 'US Government (11 federal agencies)',
  'Small Business Innovation Research 计划，美国联邦政府最大的创新资助项目之一。资助小型企业进行联邦感兴趣的研发活动，无需出让股权。',
  '1. 美国注册的小型企业（<500人）
2. 51%以上由美国公民/永久居民持有
3. 需针对联邦机构发布的具体课题提案
4. PI（首席研究员）须为该企业主要雇员',
  '1. Phase I: 最高27.5万美元（6-12个月可行性研究）
2. Phase II: 最高100万美元（原型开发，2年）
3. Phase III: 商业化（无额外SBIR资金，但可获政府合同）
4. 无需出让股权
5. 知识产权归企业所有',
  'https://www.sbir.gov/',
  '全年滚动（各机构时间不同）',
  NULL, 275000.0, 'USD', NULL,
  NULL, '["US"]',
  NULL, NULL, 120, NULL, NULL, NULL, 1
),
(
  lower(hex(randomblob(4)) || '-' || hex(randomblob(2)) || '-4' || substr(hex(randomblob(2)),2) || '-' || substr('89ab',abs(random()) % 4 + 1, 1) || substr(hex(randomblob(2)),2) || '-' || hex(randomblob(6))),
  'R&D Tax Credit (IRC §41)', NULL, 'US', NULL, 'tax_incentive', 'IRS (Internal Revenue Service)',
  '美国研发税收抵免，允许企业将合格研发支出的一定比例直接抵扣联邦所得税。初创企业可抵扣工资税。',
  '1. 美国纳税企业
2. 从事合格研发活动（技术不确定性、实验过程、技术性质）
3. 初创企业（<5年且收入<500万）可申请工资税抵免
4. 需有详细的研发活动记录和费用文档',
  '1. 合格研发费用的6-8%可直接抵扣所得税
2. 初创企业每年最高25万美元工资税抵免
3. 未用完的抵免可向前追溯1年或向后结转20年
4. 包括工资、供应品、合同研究费用等',
  'https://www.irs.gov/businesses/small-businesses-self-employed/research-credit',
  '年度纳税申报时申请',
  NULL, 250000.0, 'USD', NULL,
  NULL, '["US"]',
  NULL, NULL, NULL, NULL, NULL, NULL, 1
),

-- ═══════════════════ 🇨🇳 CN Policies ═══════════════════
(
  lower(hex(randomblob(4)) || '-' || hex(randomblob(2)) || '-4' || substr(hex(randomblob(2)),2) || '-' || substr('89ab',abs(random()) % 4 + 1, 1) || substr(hex(randomblob(2)),2) || '-' || hex(randomblob(6))),
  '高新技术企业认定', 'High-Tech Enterprise Certification', 'CN', NULL, 'tax_incentive', '科技部 / 财政部 / 国家税务总局',
  '国家级高新技术企业认定，通过后企业所得税税率由25%降至15%，是中国最具含金量的科技企业资质之一。',
  '1. 在中国注册满1年以上的居民企业
2. 近3年内通过自主研发/受让/受赠等方式拥有核心知识产权
3. 产品（服务）属于《国家重点支持的高新技术领域》
4. 研发人员占企业当年职工总数≥10%
5. 近3年研发费用占销售收入比例：
   - 最近一年收入<5000万：≥5%
   - 5000万-2亿：≥4%
   - 2亿以上：≥3%
6. 高新产品收入占总收入≥60%',
  '1. 企业所得税15%优惠税率（正常25%）
2. 研发费用加计扣除（100%）
3. 有效期3年，可续期
4. 享受地方配套补贴（各地5-50万不等）
5. 上市/融资的重要背书
6. 人才引进积分加分',
  'https://www.innofund.gov.cn/',
  '全年受理，各省批次不同',
  NULL, NULL, 'CNY', NULL,
  '["tech","biotech","manufacturing","ai"]', '["CN"]',
  NULL, NULL, NULL, NULL, NULL, NULL, 1
),
(
  lower(hex(randomblob(4)) || '-' || hex(randomblob(2)) || '-4' || substr(hex(randomblob(2)),2) || '-' || substr('89ab',abs(random()) % 4 + 1, 1) || substr(hex(randomblob(2)),2) || '-' || hex(randomblob(6))),
  '专精特新"小巨人"', 'Specialized, Refined, Differentiated & Novel SME', 'CN', NULL, 'government_grant', '工信部',
  '国家级"专精特新"小巨人企业认定，聚焦制造业补短板、产业链供应链自主可控。获评企业将获得财政奖补和政策支持。',
  '1. 在中国注册满3年，有独立法人资格
2. 属于制造业或与制造业紧密相关的生产性服务业
3. 年营业收入1000万以上
4. 近2年研发费用占比≥3%
5. 主营业务收入占比≥70%
6. 拥有核心自主知识产权
7. 细分市场占有率省内前3或全国前10',
  '1. 中央财政奖补100-600万元
2. 各省市配套补贴50-300万元
3. 优先获得政府采购和项目支持
4. 融资担保和信贷支持
5. 上市绿色通道（北交所）
6. 人才政策支持',
  'https://www.miit.gov.cn/',
  '每年1批（一般4-6月申报）',
  1000000.0, 6000000.0, 'CNY', NULL,
  NULL, '["CN"]',
  10000000.0, NULL, NULL, NULL, NULL, NULL, 1
),
(
  lower(hex(randomblob(4)) || '-' || hex(randomblob(2)) || '-4' || substr(hex(randomblob(2)),2) || '-' || substr('89ab',abs(random()) % 4 + 1, 1) || substr(hex(randomblob(2)),2) || '-' || hex(randomblob(6))),
  '科技型中小企业评价', 'Technology-Based SME Assessment', 'CN', NULL, 'tax_incentive', '科技部',
  '科技型中小企业评价入库，主要享受研发费用加计扣除优惠。门槛低于高新技术企业，适合初创科技公司。',
  '1. 在中国注册的居民企业
2. 职工总数≤500人
3. 年销售收入≤2亿元
4. 资产总额≤2亿元
5. 拥有一定的科技人员和研发投入
6. 产品/服务具有一定技术含量',
  '1. 研发费用100%加计扣除
2. 作为申报高新技术企业的过渡资质
3. 享受地方配套的科技贷款贴息
4. 优先参与政府科技项目',
  'https://www.innofund.gov.cn/',
  '全年受理',
  NULL, NULL, 'CNY', NULL,
  NULL, '["CN"]',
  NULL, 500, NULL, NULL, NULL, NULL, 1
),
(
  lower(hex(randomblob(4)) || '-' || hex(randomblob(2)) || '-4' || substr(hex(randomblob(2)),2) || '-' || substr('89ab',abs(random()) % 4 + 1, 1) || substr(hex(randomblob(2)),2) || '-' || hex(randomblob(6))),
  '深圳孔雀计划', 'Shenzhen Peacock Plan', 'CN', 'Shenzhen', 'government_grant', '深圳市人力资源和社会保障局',
  '深圳市高层次人才引进计划，面向海外高层次创新创业人才和团队。提供创业启动资金、安居补贴和科研资助。',
  '1. 海外留学归国人员或外籍高层次人才
2. 硕士及以上学历或等同专业资质
3. 在海外有3年以上工作/学习经历
4. 在深圳创办或即将创办企业
5. 团队核心成员不少于3人
6. 项目有明确的技术创新性',
  '1. A类人才：最高300万元奖励
2. B类人才：最高200万元奖励
3. C类人才：最高160万元奖励
4. 团队资助：最高1亿元
5. 项目资助：最高3000万元
6. 安居补贴、子女入学等配套',
  'https://hrss.sz.gov.cn/',
  '全年受理',
  NULL, 3000000.0, 'CNY', NULL,
  NULL, '["CN"]',
  NULL, NULL, NULL, '["masters","phd"]', NULL, 1, 1
),

-- ═══════════════════ 🇯🇵 JP Policies ═══════════════════
(
  lower(hex(randomblob(4)) || '-' || hex(randomblob(2)) || '-4' || substr(hex(randomblob(2)),2) || '-' || substr('89ab',abs(random()) % 4 + 1, 1) || substr(hex(randomblob(2)),2) || '-' || hex(randomblob(6))),
  'J-Startup', 'J-Startup Program', 'JP', NULL, 'accelerator', 'METI（経済産業省）',
  '日本经济产业省主导的国家级创业支援项目，选拔有全球潜力的日本创业公司并提供政策支持和全球推广。',
  '1. 在日本注册的创业企业
2. 有明确的全球化发展意愿
3. 技术或商业模式具有创新性
4. 获得知名VC投资或有一定营收
5. 由J-Startup推荐委员推荐',
  '1. 国家级品牌背书
2. 政府机构优先推荐
3. 海外展会参展支援
4. 海外市场开拓支援
5. 规制改革优先对应
6. 税务/法务/知财等专家支援',
  'https://www.j-startup.go.jp/',
  '每年征集（不定期）',
  NULL, NULL, 'JPY', NULL,
  NULL, '["JP"]',
  NULL, NULL, NULL, NULL, NULL, NULL, 1
),
(
  lower(hex(randomblob(4)) || '-' || hex(randomblob(2)) || '-4' || substr(hex(randomblob(2)),2) || '-' || substr('89ab',abs(random()) % 4 + 1, 1) || substr(hex(randomblob(2)),2) || '-' || hex(randomblob(6))),
  'NEDO 研发补贴', 'NEDO R&D Subsidy', 'JP', NULL, 'rd_subsidy', 'NEDO（国立研究開発法人新エネルギー・産業技術総合開発機構）',
  '日本最大的研发补贴机构之一，面向能源、环境、产业技术等领域的研发项目提供大额资助。',
  '1. 在日本注册的企业或研究机构
2. 项目属于NEDO重点支持技术领域
3. 有明确的技术开发计划和实施体制
4. 具有技术可行性和市场前景',
  '1. 研发补贴最高5亿日元（约2500万人民币）
2. 补贴比例通常为2/3
3. 无需出让股权
4. NEDO技术专家支援
5. 成果推广支援',
  'https://www.nedo.go.jp/',
  '每年多次公募',
  NULL, 500000000.0, 'JPY', NULL,
  NULL, '["JP"]',
  NULL, NULL, NULL, NULL, NULL, NULL, 1
),
(
  lower(hex(randomblob(4)) || '-' || hex(randomblob(2)) || '-4' || substr(hex(randomblob(2)),2) || '-' || substr('89ab',abs(random()) % 4 + 1, 1) || substr(hex(randomblob(2)),2) || '-' || hex(randomblob(6))),
  'IT導入補助金', 'IT Introduction Subsidy', 'JP', NULL, 'government_grant', '経済産業省 / 中小企業庁',
  '帮助中小企业引入IT工具和软件以提升生产效率的补助金。覆盖软件购买费、云服务费、导入顾问费等。',
  '1. 日本中小企业·小规模事业者
2. 引入经认定的IT工具
3. 通过认定的IT导入支援事业者申请
4. 需制定具体的IT导入计划',
  '1. A类型：30-150万日元（补贴率1/2）
2. B类型：150-450万日元（补贴率1/2）
3. 安全对策推进枠：5-100万日元
4. 数字化基盘导入枠：最高350万日元
5. 复数社连携IT导入：最高3000万日元',
  'https://www.it-hojo.jp/',
  '每年多次公募（通常3-5次）',
  NULL, 4500000.0, 'JPY', NULL,
  NULL, '["JP"]',
  NULL, NULL, NULL, NULL, NULL, NULL, 1
),
(
  lower(hex(randomblob(4)) || '-' || hex(randomblob(2)) || '-4' || substr(hex(randomblob(2)),2) || '-' || substr('89ab',abs(random()) % 4 + 1, 1) || substr(hex(randomblob(2)),2) || '-' || hex(randomblob(6))),
  '事業再構築補助金', 'Business Restructuring Subsidy', 'JP', NULL, 'government_grant', '経済産業省 / 中小企業庁',
  '支持中小企业进行事业转型、业种转换、新分野展开等业务重构的大型补助金制度。',
  '1. 日本中小企业·中堅企業
2. 受疫情或经济环境变化影响
3. 需有明确的事业再构筑计划
4. 获得认定经营革新等支援机关的确认
5. 补助对象经费需与再构筑计划直接相关',
  '1. 通常枠：100-8000万日元（补贴率2/3）
2. 大规模赁金引下促进枠：最高1.5亿日元
3. 成长枠：最高7000万日元
4. 圧缩型：最高5000万日元
5. 覆盖建筑费、机械装置费、技术导入费等',
  'https://jigyou-saikouchiku.go.jp/',
  '每年多次公募',
  NULL, 150000000.0, 'JPY', NULL,
  NULL, '["JP"]',
  NULL, NULL, NULL, NULL, NULL, NULL, 1
),
(
  lower(hex(randomblob(4)) || '-' || hex(randomblob(2)) || '-4' || substr(hex(randomblob(2)),2) || '-' || substr('89ab',abs(random()) % 4 + 1, 1) || substr(hex(randomblob(2)),2) || '-' || hex(randomblob(6))),
  '外国人起業活動促進事業', 'Foreign Entrepreneurship Promotion', 'JP', 'Tokyo', 'government_grant', '東京都 / 入国管理局',
  '为外国创业者提供在日本创业的特别签证和支援制度。通过认定后可获得最长1年的「经营·管理」在留资格准备期间。',
  '1. 外国籍（非日本国籍）
2. 有明确的在日本创业计划
3. 创业计划具有可行性和创新性
4. 有一定的创业资金（通常500万日元以上）
5. 在东京都内设立据点',
  '1. 最长1年的创业准备期签证
2. 创业支援设施（联合办公空间）
3. 行政手续代行支援
4. 经营顾问支援
5. 日语商务培训
6. 投资人·合作伙伴对接',
  'https://www.seisakukikaku.metro.tokyo.lg.jp/',
  '全年受理',
  NULL, NULL, 'JPY', NULL,
  NULL, '["JP"]',
  NULL, NULL, NULL, NULL, 1, NULL, 1
),

-- ═══════════════════ 🇸🇬 SG Policies ═══════════════════
(
  lower(hex(randomblob(4)) || '-' || hex(randomblob(2)) || '-4' || substr(hex(randomblob(2)),2) || '-' || substr('89ab',abs(random()) % 4 + 1, 1) || substr(hex(randomblob(2)),2) || '-' || hex(randomblob(6))),
  'Startup SG Founder', 'Startup SG Founder Grant', 'SG', NULL, 'government_grant', 'Enterprise Singapore',
  '新加坡政府为首次创业者提供的启动资金，通过认可的孵化器合作伙伴申请。',
  '1. 新加坡注册的私人有限公司
2. 公司注册不超过6个月
3. 创始人为新加坡公民或永久居民
4. 创始人持股≥30%
5. 首次创业（未曾获得此补贴）
6. 通过认可的合作孵化器申请',
  '1. 政府出资5万新币（约27万人民币）
2. 创始人需自付1万新币（1:5配比）
3. 孵化器导师辅导
4. 创业培训和资源对接
5. 无需出让股权（政府部分）',
  'https://www.startupsg.gov.sg/programmes/founder',
  '全年受理',
  NULL, 50000.0, 'SGD', NULL,
  NULL, '["SG"]',
  NULL, NULL, NULL, NULL, NULL, NULL, 1
),
(
  lower(hex(randomblob(4)) || '-' || hex(randomblob(2)) || '-4' || substr(hex(randomblob(2)),2) || '-' || substr('89ab',abs(random()) % 4 + 1, 1) || substr(hex(randomblob(2)),2) || '-' || hex(randomblob(6))),
  'Enterprise Development Grant', 'EDG', 'SG', NULL, 'government_grant', 'Enterprise Singapore',
  '帮助新加坡企业升级能力、创新和国际化的综合补贴计划。覆盖核心能力、创新与生产力、市场准入三大方向。',
  '1. 新加坡注册并运营的企业
2. 至少30%本地股权
3. 有具体的升级或创新项目计划
4. 企业必须在相关行业有实质运营',
  '1. 补贴率最高50%（SME可达70%）
2. 覆盖顾问费、软件/设备采购、培训费
3. 三大方向：
   - 核心能力：人力资源、财务管理
   - 创新与生产力：自动化、产品开发
   - 市场准入：海外市场拓展',
  'https://www.enterprisesg.gov.sg/financial-support/enterprise-development-grant',
  '全年受理',
  NULL, NULL, 'SGD', NULL,
  NULL, '["SG"]',
  NULL, NULL, NULL, NULL, NULL, NULL, 1
),

-- ═══════════════════ 🇭🇰 HK Policies ═══════════════════
(
  lower(hex(randomblob(4)) || '-' || hex(randomblob(2)) || '-4' || substr(hex(randomblob(2)),2) || '-' || substr('89ab',abs(random()) % 4 + 1, 1) || substr(hex(randomblob(2)),2) || '-' || hex(randomblob(6))),
  '科技券 TVP', 'Technology Voucher Programme', 'HK', NULL, 'government_grant', '香港创新科技署',
  '香港科技券计划，资助企业使用科技服务和方案来提高生产力或升级业务流程。',
  '1. 在香港注册的非上市企业
2. 在提交申请时实质业务运营不少于1年
3. 在港拥有实质业务（不限行业）
4. 不是政府资助机构或下属机构
5. 项目须以提高生产力或业务流程升级为目的',
  '1. 每个项目最高40万港币
2. 企业累计最高60万港币（可申请最多6个项目）
3. 政府资助比例：项目成本的75%
4. 覆盖：科技方案/订制设备、现成软硬件
5. 覆盖：项目审计、顾问服务',
  'https://www.itf.gov.hk/en/funding-programmes/facilitating-technology/technology-voucher-programme/',
  '全年受理',
  NULL, 600000.0, 'HKD', NULL,
  NULL, '["HK"]',
  NULL, NULL, NULL, NULL, NULL, NULL, 1
),
(
  lower(hex(randomblob(4)) || '-' || hex(randomblob(2)) || '-4' || substr(hex(randomblob(2)),2) || '-' || substr('89ab',abs(random()) % 4 + 1, 1) || substr(hex(randomblob(2)),2) || '-' || hex(randomblob(6))),
  'Cyberport 培育计划', 'Cyberport Incubation Programme', 'HK', NULL, 'accelerator', '香港数码港 Cyberport',
  '数码港的旗舰创业培育计划，为数码科技初创企业提供资金、办公空间、导师辅导和商业网络支持。',
  '1. 业务与数码科技相关
2. 香港注册公司（或收到录取通知后注册）
3. 公司成立不超过7年
4. 每个创始人只能参与一次
5. 全球任何国籍的创始人可申请',
  '1. 资助金额最高50万港币
2. 24个月培育期
3. 免费共享办公空间
4. 云服务和技术资源（Azure/AWS等）
5. 导师辅导和商业配对
6. 优先参与数码港投资者网络活动
7. 无需出让股权',
  'https://www.cyberport.hk/en/about_cyberport/cyberport_incubation_programme',
  '全年滚动受理',
  NULL, 500000.0, 'HKD', NULL,
  NULL, '["HK","*"]',
  NULL, NULL, NULL, NULL, NULL, NULL, 1
),

-- ═══════════════════ 🇬🇧 UK Policies ═══════════════════
(
  lower(hex(randomblob(4)) || '-' || hex(randomblob(2)) || '-4' || substr(hex(randomblob(2)),2) || '-' || substr('89ab',abs(random()) % 4 + 1, 1) || substr(hex(randomblob(2)),2) || '-' || hex(randomblob(6))),
  'SEIS/EIS', 'Seed Enterprise Investment Scheme / Enterprise Investment Scheme', 'UK', NULL, 'tax_incentive', 'HMRC (英国税务海关总署)',
  '英国最重要的创业投资税收优惠政策。通过为投资人提供税收减免来激励天使投资进入初创企业。',
  '1. 英国注册的有限公司
2. SEIS：公司成立<3年，资产<35万英镑，员工<25人
3. EIS：公司成立<7年（知识密集型<10年），员工<250人
4. 未在认可的证券交易所上市
5. 从事合格的商业活动',
  '1. SEIS投资人：50%所得税减免（最高15万英镑/年）
2. EIS投资人：30%所得税减免（最高200万英镑/年）
3. 资本利得税豁免（持有3年以上）
4. 损失救济：投资损失可抵扣所得税
5. 对创业公司：极大提高对天使投资人的吸引力',
  'https://www.gov.uk/guidance/venture-capital-schemes-apply-for-the-enterprise-investment-scheme',
  '全年受理（事先保证申请）',
  NULL, NULL, 'GBP', NULL,
  NULL, '["UK"]',
  NULL, NULL, NULL, NULL, NULL, NULL, 1
),
(
  lower(hex(randomblob(4)) || '-' || hex(randomblob(2)) || '-4' || substr(hex(randomblob(2)),2) || '-' || substr('89ab',abs(random()) % 4 + 1, 1) || substr(hex(randomblob(2)),2) || '-' || hex(randomblob(6))),
  'Innovate UK Smart Grant', 'Innovate UK Smart Grant', 'UK', NULL, 'government_grant', 'Innovate UK (UKRI)',
  '英国创新署的旗舰创新资助项目，面向具有高创新性和市场潜力的研发项目。不限行业、技术或地区。',
  '1. 英国注册的企业
2. 项目具有颠覆性创新潜力
3. 项目有明确的市场化路径
4. 需有详细的项目计划和成本预算
5. 中小企业优先',
  '1. 单个项目最高50万英镑
2. 中小企业：最高70%成本补贴
3. 大企业：最高50%成本补贴
4. 项目周期6-36个月
5. 无需出让股权
6. 附带Innovate UK创新导师支持',
  'https://apply-for-innovation-funding.service.gov.uk/',
  '全年滚动受理',
  NULL, 500000.0, 'GBP', NULL,
  NULL, '["UK"]',
  NULL, NULL, NULL, NULL, NULL, NULL, 1
),

-- ═══════════════════ 🇩🇪 DE / 🇰🇷 KR / 🇮🇱 IL ═══════════════════
(
  lower(hex(randomblob(4)) || '-' || hex(randomblob(2)) || '-4' || substr(hex(randomblob(2)),2) || '-' || substr('89ab',abs(random()) % 4 + 1, 1) || substr(hex(randomblob(2)),2) || '-' || hex(randomblob(6))),
  'EXIST', 'EXIST Business Startup Grant', 'DE', NULL, 'government_grant', 'BMWi（德国联邦经济和能源部）',
  '德国联邦政府面向高校科研人员和学生的创业资助计划。资助从科研到创业的转化过程。',
  '1. 与德国大学/科研机构关联
2. 创始团队不超过3人
3. 技术型或知识密集型创业项目
4. 公司尚未注册（资助期间注册）
5. 通过所在大学/机构申请',
  '1. 创始人生活费补贴：每月最高3000欧元/人
2. 儿童津贴：每月150欧元/孩
3. 材料费：最高3万欧元
4. 教练辅导：最高5000欧元
5. 资助期限：1年
6. 无需出让股权',
  'https://www.exist.de/',
  '全年受理',
  NULL, 150000.0, 'EUR', NULL,
  NULL, '["DE"]',
  NULL, NULL, NULL, NULL, NULL, NULL, 1
),
(
  lower(hex(randomblob(4)) || '-' || hex(randomblob(2)) || '-4' || substr(hex(randomblob(2)),2) || '-' || substr('89ab',abs(random()) % 4 + 1, 1) || substr(hex(randomblob(2)),2) || '-' || hex(randomblob(6))),
  'K-Startup Grand Challenge', 'K-Startup Grand Challenge', 'KR', NULL, 'accelerator', 'NIPA（韩国情報通信産業振興院）/ MSIT（科学技术情報通信部）',
  '韩国政府面向全球创业者的旗舰加速项目。入选团队在首尔接受3个月加速培训，获得资金和市场进入支持。',
  '1. 非韩国籍创始人的外国创业公司
2. 公司成立7年以内
3. 具有创新技术或商业模式
4. 创始团队至少1人全程参加（需去韩国）
5. 全球任何行业可申请',
  '1. 每支团队最高10万美元资助
2. 3.5个月加速培训（含Demo Day）
3. 免费办公空间和住宿
4. 韩国市场进入支援
5. 投资人和合作伙伴对接
6. 无需出让股权
7. 韩国往返机票报销',
  'https://www.k-startupgc.org/',
  '每年1批（通常4月开始申请）',
  NULL, 100000.0, 'USD', NULL,
  NULL, '["KR","*"]',
  NULL, NULL, NULL, NULL, NULL, NULL, 1
),
(
  lower(hex(randomblob(4)) || '-' || hex(randomblob(2)) || '-4' || substr(hex(randomblob(2)),2) || '-' || substr('89ab',abs(random()) % 4 + 1, 1) || substr(hex(randomblob(2)),2) || '-' || hex(randomblob(6))),
  'Israel Innovation Authority Seed', 'Israel Innovation Authority Early Stage Fund', 'IL', NULL, 'rd_subsidy', 'Israel Innovation Authority',
  '以色列创新局的早期研发资助计划，面向初创企业的概念验证和早期研发阶段。以色列是全球人均创业密度最高的国家。',
  '1. 以色列注册的创业公司
2. 处于早期阶段（概念验证/原型开发）
3. 项目具有技术创新性
4. 在以色列进行主要研发活动
5. 有清晰的商业化路径',
  '1. 补贴研发支出的最高50%-85%
2. 无需出让股权
3. 只需在商业化后以特许权使用费方式偿还
4. 创新局技术专家指导
5. 对接全球合作伙伴和投资人',
  'https://innovationisrael.org.il/en/',
  '全年受理',
  NULL, NULL, 'ILS', NULL,
  NULL, '["IL"]',
  NULL, NULL, NULL, NULL, NULL, NULL, 1
),

-- ═══════════════════ 🇨🇳 CN Grassroots Policies ═══════════════════
(
  lower(hex(randomblob(4)) || '-' || hex(randomblob(2)) || '-4' || substr(hex(randomblob(2)),2) || '-' || substr('89ab',abs(random()) % 4 + 1, 1) || substr(hex(randomblob(2)),2) || '-' || hex(randomblob(6))),
  '大学生创业无息贷款', 'College Student Interest-Free Startup Loan', 'CN', NULL, 'loan', '人社部 / 各地市人力资源局',
  '应届或毕业两年内大学生可申请10-50万元无息创业贷款，3年内免息，政府贴息。部分地区（如深圳）个人最高可贷60万元。',
  '1. 大专及以上学历的高校毕业生
2. 毕业2年以内（部分地区放宽至5年）
3. 在中国境内创办小微企业或从事个体经营
4. 无不良信用记录
5. 创业项目在经营范围内
6. 需提供创业计划书',
  '1. 个人贷款最高10-50万元（各地标准不同）
2. 合伙创业可叠加（最高300万元）
3. 3年内政府全额贴息
4. 还款方式灵活
5. 创业失败可享受就业援助',
  'http://www.mohrss.gov.cn/',
  '全年受理',
  100000.0, 500000.0, 'CNY', NULL,
  NULL, '["CN"]',
  NULL, NULL, 36, '["bachelor","masters","phd"]', NULL, NULL, 1
),
(
  lower(hex(randomblob(4)) || '-' || hex(randomblob(2)) || '-4' || substr(hex(randomblob(2)),2) || '-' || substr('89ab',abs(random()) % 4 + 1, 1) || substr(hex(randomblob(2)),2) || '-' || hex(randomblob(6))),
  '初创期一次性创业补贴', 'First-Time Startup One-Time Subsidy', 'CN', NULL, 'government_grant', '各地市人社局',
  '初次创办小微企业1年以上且正常经营即可申请5000-10000元一次性补贴。各城市标准不同，深圳为1万元。',
  '1. 首次创办小微企业或个体工商户
2. 正常经营1年以上
3. 至少带动1人就业（缴纳社保6个月以上）
4. 在当地社保处于正常缴纳状态
5. 未享受过同类补贴',
  '1. 一次性创业补贴5000-10000元
2. 创业带动就业补贴（每增1人补2000-3000元）
3. 创业场租补贴
4. 社会保险补贴
5. 各地具体标准和叠加政策不同',
  'http://www.mohrss.gov.cn/',
  '全年受理',
  5000.0, 10000.0, 'CNY', NULL,
  NULL, '["CN"]',
  NULL, NULL, 24, NULL, NULL, NULL, 1
),
(
  lower(hex(randomblob(4)) || '-' || hex(randomblob(2)) || '-4' || substr(hex(randomblob(2)),2) || '-' || substr('89ab',abs(random()) % 4 + 1, 1) || substr(hex(randomblob(2)),2) || '-' || hex(randomblob(6))),
  '小微企业社保全额补贴', 'SME Social Security Full Subsidy', 'CN', NULL, 'tax_incentive', '各地市人社局',
  '初创小微企业聘用人员的社保企业部分由政府承担补贴，最长3年。大幅降低初创公司用工成本。',
  '1. 小微企业（员工<300人，工业营业收入<2000万，软件<1000万）
2. 招用登记失业人员或应届毕业生
3. 签订1年以上劳动合同并缴纳社保
4. 企业正常经营
5. 不同城市资格条件略有差异',
  '1. 社保企业部分全额补贴（养老+医疗+失业+工伤+生育）
2. 补贴期限最长3年
3. 适用于每个新招录的符合条件员工
4. 大幅降低初创企业用工成本
5. 部分城市对大学生创业有额外加成',
  'http://www.mohrss.gov.cn/',
  '全年受理',
  NULL, NULL, 'CNY', NULL,
  NULL, '["CN"]',
  NULL, 20, 36, NULL, NULL, NULL, 1
),
(
  lower(hex(randomblob(4)) || '-' || hex(randomblob(2)) || '-4' || substr(hex(randomblob(2)),2) || '-' || substr('89ab',abs(random()) % 4 + 1, 1) || substr(hex(randomblob(2)),2) || '-' || hex(randomblob(6))),
  '高校毕业生落户安居补贴', 'Graduate Settlement & Housing Allowance', 'CN', NULL, 'government_grant', '各地市人才办',
  '高校毕业生首次在城市购房/租房可享受人才补贴。各城市标准差异较大，深圳本科1.5万、硕士2.5万、博士3万。',
  '1. 全日制本科及以上学历毕业生
2. 毕业后首次到该城市就业或创业
3. 年龄一般要求35周岁以下
4. 在该城市缴纳社保
5. 在该城市有合法住所（购房或租房）
6. 未享受过同类补贴',
  '1. 本科：1-3万元
2. 硕士：2.5-5万元
3. 博士：3-10万元
4. 部分城市额外提供购房补贴（10-30万不等）
5. 部分城市提供租房补贴（每月几百至几千元）
6. 子女入学等配套政策',
  'http://www.mohrss.gov.cn/',
  '全年受理',
  10000.0, 30000.0, 'CNY', NULL,
  NULL, '["CN"]',
  NULL, NULL, NULL, '["bachelor","masters","phd"]', NULL, NULL, 1
);

-- 验证
SELECT '✅ 已导入 ' || COUNT(*) || ' 条全球政策' FROM policies;
