use serde_json::json;
use sqlx::SqlitePool;
use uuid::Uuid;

pub async fn seed_global_policies(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Check if policies already have eligibility data (upgrade check)
    let has_eligibility: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM policies WHERE eligibility IS NOT NULL AND eligibility != ''",
    )
    .fetch_one(pool)
    .await
    .unwrap_or((0,));

    if has_eligibility.0 > 0 {
        return Ok(()); // Already have complete data
    }

    // Delete old incomplete seeds and re-seed with complete data
    sqlx::query("DELETE FROM policies").execute(pool).await?;

    tracing::info!("🌱 Seeding global tracking policies (complete data)...");

    let policies = vec![
        // ═══════════════════ 🇺🇸 US Policies ═══════════════════
        json!({
            "name": "Y Combinator",
            "country": "US",
            "category": "accelerator",
            "provider": "Y Combinator",
            "description": "全球顶级创业加速器，提供资金、导师辅导和全球网络。3个月密集培训，Demo Day面向全球投资人。已孵化Stripe、Airbnb、Dropbox等明星公司。",
            "eligibility": "1. 公司处于早期阶段（种子/Pre-A轮）\n2. 创始团队至少2人\n3. 全球任何国家的公司均可申请\n4. 需全职投入\n5. 培训期间需搬至旧金山湾区",
            "benefits": "1. 投资50万美元（12.5万标准+37.5万MFN SAFE）\n2. 换取7%股权\n3. 3个月密集培训+导师辅导\n4. YC校友网络（全球8000+创始人）\n5. Demo Day面向顶级投资人\n6. AWS/Google Cloud等合作伙伴福利（价值超50万美元）",
            "amount_min": 500000.0,
            "currency": "USD",
            "equity_required": 0.07,
            "url": "https://www.ycombinator.com/apply",
            "cycle": "每年两批（春季3月/秋季9月）",
            "match_countries": json!(["*"]).to_string(),
            "match_industries": json!(["tech", "biotech", "fintech", "ai"]).to_string()
        }),
        json!({
            "name": "Techstars",
            "country": "US",
            "category": "accelerator",
            "provider": "Techstars",
            "description": "全球化加速器网络，在40+城市运营。提供导师驱动的3个月加速项目，专注于特定垂直领域（金融科技、健康科技、零售科技等）。",
            "eligibility": "1. 早期创业公司\n2. 有初步产品或原型\n3. 全球任何国家可申请\n4. 创始团队需全职参与\n5. 需到指定城市参加培训",
            "benefits": "1. 投资12万美元（可转债）\n2. 换取约6%股权\n3. 3个月导师驱动加速\n4. 全球Techstars校友网络\n5. 超过300位导师资源\n6. Demo Day + 投资人对接",
            "amount_min": 120000.0,
            "currency": "USD",
            "equity_required": 0.06,
            "url": "https://www.techstars.com/accelerators",
            "cycle": "全年滚动（各城市时间不同）",
            "match_countries": json!(["*"]).to_string(),
            "match_industries": json!(["tech"]).to_string()
        }),
        json!({
            "name": "500 Global",
            "country": "US",
            "category": "accelerator",
            "provider": "500 Global",
            "description": "全球最活跃的早期风投之一，已投资80+国家2800+公司。提供4个月加速项目，聚焦新兴市场创业者。",
            "eligibility": "1. 有初步产品且有一定用户/收入\n2. 全球任何国家可申请\n3. 优先新兴市场创业者\n4. 创始团队需全职投入",
            "benefits": "1. 投资15万美元\n2. 换取约6%股权\n3. 4个月加速培训\n4. 全球导师和投资人网络\n5. 后续融资对接",
            "amount_min": 150000.0,
            "currency": "USD",
            "equity_required": 0.06,
            "url": "https://500.co/accelerators",
            "cycle": "每年2-3批",
            "match_countries": json!(["*"]).to_string()
        }),
        json!({
            "name": "SBIR Phase I",
            "country": "US",
            "category": "government_grant",
            "provider": "US Government (11 federal agencies)",
            "description": "Small Business Innovation Research 计划，美国联邦政府最大的创新资助项目之一。资助小型企业进行联邦感兴趣的研发活动，无需出让股权。",
            "eligibility": "1. 美国注册的小型企业（<500人）\n2. 51%以上由美国公民/永久居民持有\n3. 需针对联邦机构发布的具体课题提案\n4. PI（首席研究员）须为该企业主要雇员",
            "benefits": "1. Phase I: 最高27.5万美元（6-12个月可行性研究）\n2. Phase II: 最高100万美元（原型开发，2年）\n3. Phase III: 商业化（无额外SBIR资金，但可获政府合同）\n4. 无需出让股权\n5. 知识产权归企业所有",
            "amount_max": 275000.0,
            "currency": "USD",
            "url": "https://www.sbir.gov/",
            "cycle": "全年滚动（各机构时间不同）",
            "match_countries": json!(["US"]).to_string(),
            "match_company_age_max": 120
        }),
        json!({
            "name": "R&D Tax Credit (IRC §41)",
            "country": "US",
            "category": "tax_incentive",
            "provider": "IRS (Internal Revenue Service)",
            "description": "美国研发税收抵免，允许企业将合格研发支出的一定比例直接抵扣联邦所得税。初创企业可抵扣工资税。",
            "eligibility": "1. 美国纳税企业\n2. 从事合格研发活动（技术不确定性、实验过程、技术性质）\n3. 初创企业（<5年且收入<500万）可申请工资税抵免\n4. 需有详细的研发活动记录和费用文档",
            "benefits": "1. 合格研发费用的6-8%可直接抵扣所得税\n2. 初创企业每年最高25万美元工资税抵免\n3. 未用完的抵免可向前追溯1年或向后结转20年\n4. 包括工资、供应品、合同研究费用等",
            "amount_max": 250000.0,
            "currency": "USD",
            "url": "https://www.irs.gov/businesses/small-businesses-self-employed/research-credit",
            "cycle": "年度纳税申报时申请",
            "match_countries": json!(["US"]).to_string()
        }),
        // ═══════════════════ 🇨🇳 CN Policies ═══════════════════
        json!({
            "name": "高新技术企业认定",
            "name_en": "High-Tech Enterprise Certification",
            "country": "CN",
            "category": "tax_incentive",
            "provider": "科技部 / 财政部 / 国家税务总局",
            "description": "国家级高新技术企业认定，通过后企业所得税税率由25%降至15%，是中国最具含金量的科技企业资质之一。",
            "eligibility": "1. 在中国注册满1年以上的居民企业\n2. 近3年内通过自主研发/受让/受赠等方式拥有核心知识产权\n3. 产品（服务）属于《国家重点支持的高新技术领域》\n4. 研发人员占企业当年职工总数≥10%\n5. 近3年研发费用占销售收入比例：\n   - 最近一年收入<5000万：≥5%\n   - 5000万-2亿：≥4%\n   - 2亿以上：≥3%\n6. 高新产品收入占总收入≥60%",
            "benefits": "1. 企业所得税15%优惠税率（正常25%）\n2. 研发费用加计扣除（100%）\n3. 有效期3年，可续期\n4. 享受地方配套补贴（各地5-50万不等）\n5. 上市/融资的重要背书\n6. 人才引进积分加分",
            "currency": "CNY",
            "url": "https://www.innofund.gov.cn/",
            "cycle": "全年受理，各省批次不同",
            "match_countries": json!(["CN"]).to_string(),
            "match_industries": json!(["tech", "biotech", "manufacturing", "ai"]).to_string()
        }),
        json!({
            "name": "专精特新\"小巨人\"",
            "name_en": "Specialized, Refined, Differentiated & Novel SME",
            "country": "CN",
            "category": "government_grant",
            "provider": "工信部",
            "description": "国家级\"专精特新\"小巨人企业认定，聚焦制造业补短板、产业链供应链自主可控。获评企业将获得财政奖补和政策支持。",
            "eligibility": "1. 在中国注册满3年，有独立法人资格\n2. 属于制造业或与制造业紧密相关的生产性服务业\n3. 年营业收入1000万以上\n4. 近2年研发费用占比≥3%\n5. 主营业务收入占比≥70%\n6. 拥有核心自主知识产权\n7. 细分市场占有率省内前3或全国前10",
            "benefits": "1. 中央财政奖补100-600万元\n2. 各省市配套补贴50-300万元\n3. 优先获得政府采购和项目支持\n4. 融资担保和信贷支持\n5. 上市绿色通道（北交所）\n6. 人才政策支持",
            "amount_min": 1000000.0,
            "amount_max": 6000000.0,
            "currency": "CNY",
            "url": "https://www.miit.gov.cn/",
            "cycle": "每年1批（一般4-6月申报）",
            "match_countries": json!(["CN"]).to_string(),
            "match_min_revenue": 10000000.0
        }),
        json!({
            "name": "科技型中小企业评价",
            "name_en": "Technology-Based SME Assessment",
            "country": "CN",
            "category": "tax_incentive",
            "provider": "科技部",
            "description": "科技型中小企业评价入库，主要享受研发费用加计扣除优惠。门槛低于高新技术企业，适合初创科技公司。",
            "eligibility": "1. 在中国注册的居民企业\n2. 职工总数≤500人\n3. 年销售收入≤2亿元\n4. 资产总额≤2亿元\n5. 拥有一定的科技人员和研发投入\n6. 产品/服务具有一定技术含量",
            "benefits": "1. 研发费用100%加计扣除\n2. 作为申报高新技术企业的过渡资质\n3. 享受地方配套的科技贷款贴息\n4. 优先参与政府科技项目",
            "currency": "CNY",
            "url": "https://www.innofund.gov.cn/",
            "cycle": "全年受理",
            "match_countries": json!(["CN"]).to_string(),
            "match_max_employees": 500
        }),
        json!({
            "name": "深圳孔雀计划",
            "name_en": "Shenzhen Peacock Plan",
            "country": "CN",
            "region": "Shenzhen",
            "category": "government_grant",
            "provider": "深圳市人力资源和社会保障局",
            "description": "深圳市高层次人才引进计划，面向海外高层次创新创业人才和团队。提供创业启动资金、安居补贴和科研资助。",
            "eligibility": "1. 海外留学归国人员或外籍高层次人才\n2. 硕士及以上学历或等同专业资质\n3. 在海外有3年以上工作/学习经历\n4. 在深圳创办或即将创办企业\n5. 团队核心成员不少于3人\n6. 项目有明确的技术创新性",
            "benefits": "1. A类人才：最高300万元奖励\n2. B类人才：最高200万元奖励\n3. C类人才：最高160万元奖励\n4. 团队资助：最高1亿元\n5. 项目资助：最高3000万元\n6. 安居补贴、子女入学等配套",
            "amount_max": 3000000.0,
            "currency": "CNY",
            "url": "https://hrss.sz.gov.cn/",
            "cycle": "全年受理",
            "match_countries": json!(["CN"]).to_string(),
            "match_founder_is_overseas_returnee": 1,
            "match_founder_education": json!(["masters", "phd"]).to_string()
        }),
        // ═══════════════════ 🇯🇵 JP Policies ═══════════════════
        json!({
            "name": "J-Startup",
            "name_en": "J-Startup Program",
            "country": "JP",
            "category": "accelerator",
            "provider": "METI（経済産業省）",
            "description": "日本经济产业省主导的国家级创业支援项目，选拔有全球潜力的日本创业公司并提供政策支持和全球推广。",
            "eligibility": "1. 在日本注册的创业企业\n2. 有明确的全球化发展意愿\n3. 技术或商业模式具有创新性\n4. 获得知名VC投资或有一定营收\n5. 由J-Startup推荐委员推荐",
            "benefits": "1. 国家级品牌背书\n2. 政府机构优先推荐\n3. 海外展会参展支援\n4. 海外市场开拓支援\n5. 规制改革优先对应\n6. 税务/法务/知财等专家支援",
            "currency": "JPY",
            "url": "https://www.j-startup.go.jp/",
            "cycle": "每年征集（不定期）",
            "match_countries": json!(["JP"]).to_string()
        }),
        json!({
            "name": "NEDO 研发补贴",
            "name_en": "NEDO R&D Subsidy",
            "country": "JP",
            "category": "rd_subsidy",
            "provider": "NEDO（国立研究開発法人新エネルギー・産業技術総合開発機構）",
            "description": "日本最大的研发补贴机构之一，面向能源、环境、产业技术等领域的研发项目提供大额资助。",
            "eligibility": "1. 在日本注册的企业或研究机构\n2. 项目属于NEDO重点支持技术领域\n3. 有明确的技术开发计划和实施体制\n4. 具有技术可行性和市场前景",
            "benefits": "1. 研发补贴最高5亿日元（约2500万人民币）\n2. 补贴比例通常为2/3\n3. 无需出让股权\n4. NEDO技术专家支援\n5. 成果推广支援",
            "amount_max": 500000000.0,
            "currency": "JPY",
            "url": "https://www.nedo.go.jp/",
            "cycle": "每年多次公募",
            "match_countries": json!(["JP"]).to_string()
        }),
        json!({
            "name": "IT導入補助金",
            "name_en": "IT Introduction Subsidy",
            "country": "JP",
            "category": "government_grant",
            "provider": "経済産業省 / 中小企業庁",
            "description": "帮助中小企业引入IT工具和软件以提升生产效率的补助金。覆盖软件购买费、云服务费、导入顾问费等。",
            "eligibility": "1. 日本中小企业·小规模事业者\n2. 引入经认定的IT工具\n3. 通过认定的IT导入支援事业者申请\n4. 需制定具体的IT导入计划",
            "benefits": "1. A类型：30-150万日元（补贴率1/2）\n2. B类型：150-450万日元（补贴率1/2）\n3. 安全对策推进枠：5-100万日元\n4. 数字化基盘导入枠：最高350万日元\n5. 复数社连携IT导入：最高3000万日元",
            "amount_max": 4500000.0,
            "currency": "JPY",
            "url": "https://www.it-hojo.jp/",
            "cycle": "每年多次公募（通常3-5次）",
            "match_countries": json!(["JP"]).to_string()
        }),
        json!({
            "name": "事業再構築補助金",
            "name_en": "Business Restructuring Subsidy",
            "country": "JP",
            "category": "government_grant",
            "provider": "経済産業省 / 中小企業庁",
            "description": "支持中小企业进行事业转型、业种转换、新分野展开等业务重构的大型补助金制度。",
            "eligibility": "1. 日本中小企业·中堅企業\n2. 受疫情或经济环境变化影响\n3. 需有明确的事业再构筑计划\n4. 获得认定经营革新等支援机关的确认\n5. 补助对象经费需与再构筑计划直接相关",
            "benefits": "1. 通常枠：100-8000万日元（补贴率2/3）\n2. 大规模赁金引下促进枠：最高1.5亿日元\n3. 成长枠：最高7000万日元\n4. 圧缩型：最高5000万日元\n5. 覆盖建筑费、机械装置费、技术导入费等",
            "amount_max": 150000000.0,
            "currency": "JPY",
            "url": "https://jigyou-saikouchiku.go.jp/",
            "cycle": "每年多次公募",
            "match_countries": json!(["JP"]).to_string()
        }),
        json!({
            "name": "外国人起業活動促進事業",
            "name_en": "Foreign Entrepreneurship Promotion",
            "country": "JP",
            "region": "Tokyo",
            "category": "government_grant",
            "provider": "東京都 / 入国管理局",
            "description": "为外国创业者提供在日本创业的特别签证和支援制度。通过认定后可获得最长1年的「经营·管理」在留资格准备期间。",
            "eligibility": "1. 外国籍（非日本国籍）\n2. 有明确的在日本创业计划\n3. 创业计划具有可行性和创新性\n4. 有一定的创业资金（通常500万日元以上）\n5. 在东京都内设立据点",
            "benefits": "1. 最长1年的创业准备期签证\n2. 创业支援设施（联合办公空间）\n3. 行政手续代行支援\n4. 经营顾问支援\n5. 日语商务培训\n6. 投资人·合作伙伴对接",
            "currency": "JPY",
            "url": "https://www.seisakukikaku.metro.tokyo.lg.jp/",
            "cycle": "全年受理",
            "match_countries": json!(["JP"]).to_string(),
            "match_founder_is_expat": 1
        }),
        // ═══════════════════ 🇸🇬 SG Policies ═══════════════════
        json!({
            "name": "Startup SG Founder",
            "name_en": "Startup SG Founder Grant",
            "country": "SG",
            "category": "government_grant",
            "provider": "Enterprise Singapore",
            "description": "新加坡政府为首次创业者提供的启动资金，通过认可的孵化器合作伙伴申请。",
            "eligibility": "1. 新加坡注册的私人有限公司\n2. 公司注册不超过6个月\n3. 创始人为新加坡公民或永久居民\n4. 创始人持股≥30%\n5. 首次创业（未曾获得此补贴）\n6. 通过认可的合作孵化器申请",
            "benefits": "1. 政府出资5万新币（约27万人民币）\n2. 创始人需自付1万新币（1:5配比）\n3. 孵化器导师辅导\n4. 创业培训和资源对接\n5. 无需出让股权（政府部分）",
            "amount_max": 50000.0,
            "currency": "SGD",
            "url": "https://www.startupsg.gov.sg/programmes/founder",
            "cycle": "全年受理",
            "match_countries": json!(["SG"]).to_string()
        }),
        json!({
            "name": "Enterprise Development Grant",
            "name_en": "EDG",
            "country": "SG",
            "category": "government_grant",
            "provider": "Enterprise Singapore",
            "description": "帮助新加坡企业升级能力、创新和国际化的综合补贴计划。覆盖核心能力、创新与生产力、市场准入三大方向。",
            "eligibility": "1. 新加坡注册并运营的企业\n2. 至少30%本地股权\n3. 有具体的升级或创新项目计划\n4. 企业必须在相关行业有实质运营",
            "benefits": "1. 补贴率最高50%（SME可达70%）\n2. 覆盖顾问费、软件/设备采购、培训费\n3. 三大方向：\n   - 核心能力：人力资源、财务管理\n   - 创新与生产力：自动化、产品开发\n   - 市场准入：海外市场拓展",
            "currency": "SGD",
            "url": "https://www.enterprisesg.gov.sg/financial-support/enterprise-development-grant",
            "cycle": "全年受理",
            "match_countries": json!(["SG"]).to_string()
        }),
        // ═══════════════════ 🇭🇰 HK Policies ═══════════════════
        json!({
            "name": "科技券 TVP",
            "name_en": "Technology Voucher Programme",
            "country": "HK",
            "category": "government_grant",
            "provider": "香港创新科技署",
            "description": "香港科技券计划，资助企业使用科技服务和方案来提高生产力或升级业务流程。",
            "eligibility": "1. 在香港注册的非上市企业\n2. 在提交申请时实质业务运营不少于1年\n3. 在港拥有实质业务（不限行业）\n4. 不是政府资助机构或下属机构\n5. 项目须以提高生产力或业务流程升级为目的",
            "benefits": "1. 每个项目最高40万港币\n2. 企业累计最高60万港币（可申请最多6个项目）\n3. 政府资助比例：项目成本的75%\n4. 覆盖：科技方案/订制设备、现成软硬件\n5. 覆盖：项目审计、顾问服务",
            "amount_max": 600000.0,
            "currency": "HKD",
            "url": "https://www.itf.gov.hk/en/funding-programmes/facilitating-technology/technology-voucher-programme/",
            "cycle": "全年受理",
            "match_countries": json!(["HK"]).to_string()
        }),
        json!({
            "name": "Cyberport 培育计划",
            "name_en": "Cyberport Incubation Programme",
            "country": "HK",
            "category": "accelerator",
            "provider": "香港数码港 Cyberport",
            "description": "数码港的旗舰创业培育计划，为数码科技初创企业提供资金、办公空间、导师辅导和商业网络支持。",
            "eligibility": "1. 业务与数码科技相关\n2. 香港注册公司（或收到录取通知后注册）\n3. 公司成立不超过7年\n4. 每个创始人只能参与一次\n5. 全球任何国籍的创始人可申请",
            "benefits": "1. 资助金额最高50万港币\n2. 24个月培育期\n3. 免费共享办公空间\n4. 云服务和技术资源（Azure/AWS等）\n5. 导师辅导和商业配对\n6. 优先参与数码港投资者网络活动\n7. 无需出让股权",
            "amount_max": 500000.0,
            "currency": "HKD",
            "url": "https://www.cyberport.hk/en/about_cyberport/cyberport_incubation_programme",
            "cycle": "全年滚动受理",
            "match_countries": json!(["HK", "*"]).to_string()
        }),
        // ═══════════════════ 🇬🇧 UK Policies ═══════════════════
        json!({
            "name": "SEIS/EIS",
            "name_en": "Seed Enterprise Investment Scheme / Enterprise Investment Scheme",
            "country": "UK",
            "category": "tax_incentive",
            "provider": "HMRC (英国税务海关总署)",
            "description": "英国最重要的创业投资税收优惠政策。通过为投资人提供税收减免来激励天使投资进入初创企业。",
            "eligibility": "1. 英国注册的有限公司\n2. SEIS：公司成立<3年，资产<35万英镑，员工<25人\n3. EIS：公司成立<7年（知识密集型<10年），员工<250人\n4. 未在认可的证券交易所上市\n5. 从事合格的商业活动",
            "benefits": "1. SEIS投资人：50%所得税减免（最高15万英镑/年）\n2. EIS投资人：30%所得税减免（最高200万英镑/年）\n3. 资本利得税豁免（持有3年以上）\n4. 损失救济：投资损失可抵扣所得税\n5. 对创业公司：极大提高对天使投资人的吸引力",
            "currency": "GBP",
            "url": "https://www.gov.uk/guidance/venture-capital-schemes-apply-for-the-enterprise-investment-scheme",
            "cycle": "全年受理（事先保证申请）",
            "match_countries": json!(["UK"]).to_string()
        }),
        json!({
            "name": "Innovate UK Smart Grant",
            "name_en": "Innovate UK Smart Grant",
            "country": "UK",
            "category": "government_grant",
            "provider": "Innovate UK (UKRI)",
            "description": "英国创新署的旗舰创新资助项目，面向具有高创新性和市场潜力的研发项目。不限行业、技术或地区。",
            "eligibility": "1. 英国注册的企业\n2. 项目具有颠覆性创新潜力\n3. 项目有明确的市场化路径\n4. 需有详细的项目计划和成本预算\n5. 中小企业优先",
            "benefits": "1. 单个项目最高50万英镑\n2. 中小企业：最高70%成本补贴\n3. 大企业：最高50%成本补贴\n4. 项目周期6-36个月\n5. 无需出让股权\n6. 附带Innovate UK创新导师支持",
            "amount_max": 500000.0,
            "currency": "GBP",
            "url": "https://apply-for-innovation-funding.service.gov.uk/",
            "cycle": "全年滚动受理",
            "match_countries": json!(["UK"]).to_string()
        }),
        // ═══════════════════ 🇩🇪 DE / 🇰🇷 KR / 🇮🇱 IL ═══════════════════
        json!({
            "name": "EXIST",
            "name_en": "EXIST Business Startup Grant",
            "country": "DE",
            "category": "government_grant",
            "provider": "BMWi（德国联邦经济和能源部）",
            "description": "德国联邦政府面向高校科研人员和学生的创业资助计划。资助从科研到创业的转化过程。",
            "eligibility": "1. 与德国大学/科研机构关联\n2. 创始团队不超过3人\n3. 技术型或知识密集型创业项目\n4. 公司尚未注册（资助期间注册）\n5. 通过所在大学/机构申请",
            "benefits": "1. 创始人生活费补贴：每月最高3000欧元/人\n2. 儿童津贴：每月150欧元/孩\n3. 材料费：最高3万欧元\n4. 教练辅导：最高5000欧元\n5. 资助期限：1年\n6. 无需出让股权",
            "amount_max": 150000.0,
            "currency": "EUR",
            "url": "https://www.exist.de/",
            "cycle": "全年受理",
            "match_countries": json!(["DE"]).to_string()
        }),
        json!({
            "name": "K-Startup Grand Challenge",
            "name_en": "K-Startup Grand Challenge",
            "country": "KR",
            "category": "accelerator",
            "provider": "NIPA（韩国情報通信産業振興院）/ MSIT（科学技术情報通信部）",
            "description": "韩国政府面向全球创业者的旗舰加速项目。入选团队在首尔接受3个月加速培训，获得资金和市场进入支持。",
            "eligibility": "1. 非韩国籍创始人的外国创业公司\n2. 公司成立7年以内\n3. 具有创新技术或商业模式\n4. 创始团队至少1人全程参加（需去韩国）\n5. 全球任何行业可申请",
            "benefits": "1. 每支团队最高10万美元资助\n2. 3.5个月加速培训（含Demo Day）\n3. 免费办公空间和住宿\n4. 韩国市场进入支援\n5. 投资人和合作伙伴对接\n6. 无需出让股权\n7. 韩国往返机票报销",
            "amount_max": 100000.0,
            "currency": "USD",
            "url": "https://www.k-startupgc.org/",
            "cycle": "每年1批（通常4月开始申请）",
            "match_countries": json!(["KR", "*"]).to_string()
        }),
        json!({
            "name": "Israel Innovation Authority Seed",
            "name_en": "Israel Innovation Authority Early Stage Fund",
            "country": "IL",
            "category": "rd_subsidy",
            "provider": "Israel Innovation Authority",
            "description": "以色列创新局的早期研发资助计划，面向初创企业的概念验证和早期研发阶段。以色列是全球人均创业密度最高的国家。",
            "eligibility": "1. 以色列注册的创业公司\n2. 处于早期阶段（概念验证/原型开发）\n3. 项目具有技术创新性\n4. 在以色列进行主要研发活动\n5. 有清晰的商业化路径",
            "benefits": "1. 补贴研发支出的最高50%-85%\n2. 无需出让股权\n3. 只需在商业化后以特许权使用费方式偿还\n4. 创新局技术专家指导\n5. 对接全球合作伙伴和投资人",
            "currency": "ILS",
            "url": "https://innovationisrael.org.il/en/",
            "cycle": "全年受理",
            "match_countries": json!(["IL"]).to_string()
        }),
        // ═══════════════════ 🇨🇳 CN Grassroots Policies ═══════════════════
        json!({
            "name": "大学生创业无息贷款",
            "name_en": "College Student Interest-Free Startup Loan",
            "country": "CN",
            "category": "loan",
            "provider": "人社部 / 各地市人力资源局",
            "description": "应届或毕业两年内大学生可申请10-50万元无息创业贷款，3年内免息，政府贴息。部分地区（如深圳）个人最高可贷60万元。",
            "eligibility": "1. 大专及以上学历的高校毕业生\n2. 毕业2年以内（部分地区放宽至5年）\n3. 在中国境内创办小微企业或从事个体经营\n4. 无不良信用记录\n5. 创业项目在经营范围内\n6. 需提供创业计划书",
            "benefits": "1. 个人贷款最高10-50万元（各地标准不同）\n2. 合伙创业可叠加（最高300万元）\n3. 3年内政府全额贴息\n4. 还款方式灵活\n5. 创业失败可享受就业援助",
            "amount_min": 100000.0,
            "amount_max": 500000.0,
            "currency": "CNY",
            "url": "http://www.mohrss.gov.cn/",
            "cycle": "全年受理",
            "match_countries": json!(["CN"]).to_string(),
            "match_founder_education": json!(["bachelor", "masters", "phd"]).to_string(),
            "match_company_age_max": 36
        }),
        json!({
            "name": "初创期一次性创业补贴",
            "name_en": "First-Time Startup One-Time Subsidy",
            "country": "CN",
            "category": "government_grant",
            "provider": "各地市人社局",
            "description": "初次创办小微企业1年以上且正常经营即可申请5000-10000元一次性补贴。各城市标准不同，深圳为1万元。",
            "eligibility": "1. 首次创办小微企业或个体工商户\n2. 正常经营1年以上\n3. 至少带动1人就业（缴纳社保6个月以上）\n4. 在当地社保处于正常缴纳状态\n5. 未享受过同类补贴",
            "benefits": "1. 一次性创业补贴5000-10000元\n2. 创业带动就业补贴（每增1人补2000-3000元）\n3. 创业场租补贴\n4. 社会保险补贴\n5. 各地具体标准和叠加政策不同",
            "amount_min": 5000.0,
            "amount_max": 10000.0,
            "currency": "CNY",
            "url": "http://www.mohrss.gov.cn/",
            "cycle": "全年受理",
            "match_countries": json!(["CN"]).to_string(),
            "match_company_age_max": 24
        }),
        json!({
            "name": "小微企业社保全额补贴",
            "name_en": "SME Social Security Full Subsidy",
            "country": "CN",
            "category": "tax_incentive",
            "provider": "各地市人社局",
            "description": "初创小微企业聘用人员的社保企业部分由政府承担补贴，最长3年。大幅降低初创公司用工成本。",
            "eligibility": "1. 小微企业（员工<300人，工业营业收入<2000万，软件<1000万）\n2. 招用登记失业人员或应届毕业生\n3. 签订1年以上劳动合同并缴纳社保\n4. 企业正常经营\n5. 不同城市资格条件略有差异",
            "benefits": "1. 社保企业部分全额补贴（养老+医疗+失业+工伤+生育）\n2. 补贴期限最长3年\n3. 适用于每个新招录的符合条件员工\n4. 大幅降低初创企业用工成本\n5. 部分城市对大学生创业有额外加成",
            "currency": "CNY",
            "url": "http://www.mohrss.gov.cn/",
            "cycle": "全年受理",
            "match_countries": json!(["CN"]).to_string(),
            "match_max_employees": 20,
            "match_company_age_max": 36
        }),
        json!({
            "name": "高校毕业生落户安居补贴",
            "name_en": "Graduate Settlement & Housing Allowance",
            "country": "CN",
            "category": "government_grant",
            "provider": "各地市人才办",
            "description": "高校毕业生首次在城市购房/租房可享受人才补贴。各城市标准差异较大，深圳本科1.5万、硕士2.5万、博士3万。",
            "eligibility": "1. 全日制本科及以上学历毕业生\n2. 毕业后首次到该城市就业或创业\n3. 年龄一般要求35周岁以下\n4. 在该城市缴纳社保\n5. 在该城市有合法住所（购房或租房）\n6. 未享受过同类补贴",
            "benefits": "1. 本科：1-3万元\n2. 硕士：2.5-5万元\n3. 博士：3-10万元\n4. 部分城市额外提供购房补贴（10-30万不等）\n5. 部分城市提供租房补贴（每月几百至几千元）\n6. 子女入学等配套政策",
            "amount_min": 10000.0,
            "amount_max": 30000.0,
            "currency": "CNY",
            "url": "http://www.mohrss.gov.cn/",
            "cycle": "全年受理",
            "match_countries": json!(["CN"]).to_string(),
            "match_founder_education": json!(["bachelor", "masters", "phd"]).to_string()
        }),
    ];

    for p in policies {
        sqlx::query(
            r#"
            INSERT INTO policies (
                id, name, name_en, country, region, category, provider, description,
                eligibility, benefits, url, deadline, cycle,
                amount_min, amount_max, currency, equity_required,
                match_industries, match_countries, match_min_revenue, match_max_employees, match_company_age_max,
                match_founder_education, match_founder_is_expat, match_founder_is_overseas_returnee
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(Uuid::new_v4().to_string())
        .bind(p.get("name").and_then(|v| v.as_str()).unwrap_or(""))
        .bind(p.get("name_en").and_then(|v| v.as_str()))
        .bind(p.get("country").and_then(|v| v.as_str()).unwrap_or(""))
        .bind(p.get("region").and_then(|v| v.as_str()))
        .bind(p.get("category").and_then(|v| v.as_str()).unwrap_or(""))
        .bind(p.get("provider").and_then(|v| v.as_str()))
        .bind(p.get("description").and_then(|v| v.as_str()))
        .bind(p.get("eligibility").and_then(|v| v.as_str()))
        .bind(p.get("benefits").and_then(|v| v.as_str()))
        .bind(p.get("url").and_then(|v| v.as_str()))
        .bind(p.get("deadline").and_then(|v| v.as_str()))
        .bind(p.get("cycle").and_then(|v| v.as_str()))
        .bind(p.get("amount_min").and_then(|v| v.as_f64()))
        .bind(p.get("amount_max").and_then(|v| v.as_f64()))
        .bind(p.get("currency").and_then(|v| v.as_str()).unwrap_or("USD"))
        .bind(p.get("equity_required").and_then(|v| v.as_f64()))
        .bind(p.get("match_industries").and_then(|v| v.as_str()))
        .bind(p.get("match_countries").and_then(|v| v.as_str()))
        .bind(p.get("match_min_revenue").and_then(|v| v.as_f64()))
        .bind(p.get("match_max_employees").and_then(|v| v.as_i64()))
        .bind(p.get("match_company_age_max").and_then(|v| v.as_i64()))
        .bind(p.get("match_founder_education").and_then(|v| v.as_str()))
        .bind(p.get("match_founder_is_expat").and_then(|v| v.as_i64()))
        .bind(p.get("match_founder_is_overseas_returnee").and_then(|v| v.as_i64()))
        .execute(pool)
        .await?;
    }

    tracing::info!("✅ Global policies seeded (complete data with eligibility/benefits)");
    Ok(())
}
