# OCR 回归测试集

本目录维护输入、出处、哈希和人工核对的标准答案；通过本机 Ollama 对比文字识别与业务字段抽取。它不执行 ERP 写入，也不评价合同法律结论。

首版 11 个页面用例：4 个中文合成回归样本、2 个 Microsoft 发票示例（JPG/PDF）、2 张 ExpressExpense 真实收据照片、英文服务合同第 25 页与中文空白合同第 6/8 页（这两份合同仅在操作者显式下载的外部缓存中使用）。相同来源文档的变体通过 `family` 关联，不能当作独立样本；公开数据也可能已进入模型训练集，不属于盲测。

## 运行

需要 Python 3.10+。PDF 推理需要系统 `pdftoppm`（Poppler）；验证和评分不需要 PDF 工具或模型。测试器不自动安装依赖、不自动拉取模型、不修改生产配置。模型 API 固定使用 `127.0.0.1:11434`。

```bash
python3 scripts/test_ocr_benchmark.py
python3 scripts/ocr_benchmark.py validate
python3 scripts/ocr_benchmark.py fetch

python3 scripts/ocr_benchmark.py run \
  --model glm-ocr:q8_0 \
  --model hf.co/PaddlePaddle/PaddleOCR-VL-1.6-GGUF:latest \
  --model ovisocr2-local:q8_0 \
  --task text --output-dir tests/ocr/results/run-001

python3 scripts/ocr_benchmark.py run \
  --model glm-ocr:q8_0 \
  --model hf.co/PaddlePaddle/PaddleOCR-VL-1.6-GGUF:latest \
  --model ovisocr2-local:q8_0 \
  --task fields --case synthetic-01_invoice --case azure-invoice-image \
  --case expressexpense-srd-1078 --case cn-contract-blank-p8 \
  --output-dir tests/ocr/results/run-001

python3 scripts/ocr_benchmark.py score --output-dir tests/ocr/results/run-001
```

使用新的输出目录保留每次结果。原始响应、失败、文件/模型摘要、参数及所选用例清单均保存；文字和字段任务分别评分。模型名称必须与 `ollama list` 一致，OvisOCR2 是本机从已校验社区 GGUF 导入的名称，并非 Ollama 官方库标签。仓库不含模型权重。

现有离线脚本支持清单及仓库文件哈希校验；公开候选不包含 CI 工作流；不下载外部资料、不调用模型。缓存缺失显示 `cache_missing_unverified`，不会声称外部文件已验证；真正推理遇到缺文件/哈希不符会记录失败。PDF 每次只渲染指定页面，最长边 1600px；照片保持原始分辨率。CPU 16 线程，串行运行，每页结束释放模型，耗时包含冷加载、PDF 渲染与请求准备，不能直接与热加载结果比较。

## 标注与指标

- `reference_text` 仅放人工核对的完整可见文字。真实收据局部不可辨认、中文合同文本层含异常控制字符，因此这些用例保留 null，不计算全文 CER。已有 5 页有全文标注。
- `expected_fields` 按字段名做精确匹配，普通文本保留大小写，编号保留前导零，日期要求 ISO，金额按 Decimal 比较。空白金额、未选择的付款方式等必须返回 null，缺少键也不算正确。字段提示词仅获得字段名和通用规则，不获得标准答案。
- `expected_text_values` 是文字中实际可见的值，例如 `$` 而不是推断的 USD、原始日期格式而不是要求模型格式化后的日期。文字覆盖不能代表字段关联正确率。
- 可见文字 CER 会去除 HTML/Markdown 排版，并统一 Unicode/空白；不评价表格结构。多栏阅读顺序或页眉位置差异仍可能影响 CER。
- 发票重点区分本期总额 110、历史余额 500、总应付 610，以及供应商/购买方。原示例行单价与行总额有内部不一致，不能擅自“修正”标准答案。
- `observed_cer` / `observed_text_value_coverage` 评价实际返回文字（仍保留空/缺失分母），独立于协议接受指标；例如有文字但 `done:false` 仍是未确认完成，不能凭 observed 分数称请求成功。
- 失败、超时、截断和已选择却缺失的用例保留在分母。部分字段标注只能衡量这些字段；未标注字段、整表行列和法律推理不在通过声明范围内。

字段任务使用测试器统一提示词，不等同于 ERP 生产接口的端到端验收；未运行后续业务模型或人工确认流程。

## 增补样本

在 `manifest.json` 新增稳定 ID、类别/语言、family、出处/下载地址/许可、获取日期、文件 SHA-256 与人工核对记录。先看原图/PDF，再标注，最后跑模型；禁止用待测模型的输出反填标准答案。真实用户文件需在对应产品/组织权限范围内另行授权和脱敏，不应直接进入本共享测试集。

可以重用已知失败样本做回归，但应保留原始失败输出；改模型、提示词或转换参数后用新目录复测，按类别分别分析。不要为提高分数静默改标准答案、删除失败页或只报告成功页。`regression`/`challenge` 是用途标签，不是已经隔离的训练/测试划分。

## 来源和许可

- 合成图是本项目人工生成的虚构数据，无实际财务或法律效力。
- Microsoft [Document Intelligence samples](https://github.com/Azure-Samples/document-intelligence-code-samples) 的发票示例固定至 `8ac994c2712f3104d00ca4224b0ded6fd1ecbf70`；保留 [MIT 声明](licenses/Microsoft-MIT.txt)。
- 收据来自 [ExpressExpense.com 官方 SRD](https://expressexpense.com/blog/free-receipt-images-ocr-machine-learning-dataset/)，依据发布方明确的数据集 MIT 许可，保留 [MIT 声明](licenses/ExpressExpense-MIT.txt)。原始 ZIP 条目及来源见清单，不以第三方标注许可覆盖原图。所选照片未发现个人姓名/卡号信息，商户公开信息保留。
- 英文 [Model Services Contract](https://www.gov.uk/government/publications/model-services-contract-terms)：Contains public sector information licensed under the [Open Government Licence v3.0](https://www.nationalarchives.gov.uk/doc/open-government-licence/version/3/). 许可针对信息，不授予政府标志的使用权，也不暗示认可。因此本公开候选不携带整份 PDF，只保留来源及可选缓存配置。
- 中文合同来自[临夏州市场监管局官方文本库](https://scjg.linxia.gov.cn/scjg/zfxxgk/fdzdgknr/zdlyxxgk/XFWQ/htwbk/art/2022/art_628cbdd912a9450daa699af20d407ec9.html)。未发现明确开放转载许可，原 PDF/渲染页仅下载到 `~/.cache/w-erp-ocr-corpus`，不入库。历史示范文本用于 OCR，不表示它是当前适用的法律模板。
