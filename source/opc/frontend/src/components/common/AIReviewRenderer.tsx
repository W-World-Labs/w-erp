import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { ShieldAlert, AlertTriangle, ShieldCheck, Info, Copy, Check } from "lucide-react";
import { useState } from "react";
import { Button } from "@/components/ui/button";

interface AIReviewRendererProps {
    content: string;
    className?: string;
}

/**
 * 增强版 AI 审查结果渲染器
 * - Markdown 渲染 + GFM 表格支持
 * - 风险标签自动高亮 (高风险/中风险/低风险)
 * - 字段名称代码化显示
 * - 结构化标题样式
 * - 一键复制
 */
export function AIReviewRenderer({ content, className = "" }: AIReviewRendererProps) {
    const [copied, setCopied] = useState(false);

    const handleCopy = () => {
        navigator.clipboard.writeText(content);
        setCopied(true);
        setTimeout(() => setCopied(false), 2000);
    };

    return (
        <div className={`relative ${className}`}>
            {/* 复制按钮 */}
            <div className="absolute top-0 right-0 z-10">
                <Button
                    variant="ghost"
                    size="sm"
                    className="h-7 text-xs gap-1 opacity-60 hover:opacity-100"
                    onClick={handleCopy}
                >
                    {copied ? <Check className="h-3 w-3" /> : <Copy className="h-3 w-3" />}
                    {copied ? "已复制" : "复制"}
                </Button>
            </div>

            {/* Markdown 渲染 */}
            <div className="ai-review-content">
                <ReactMarkdown
                    remarkPlugins={[remarkGfm]}
                    components={{
                        // 一级标题 — 报告标题
                        h1: ({ children }) => (
                            <h1 className="text-lg font-bold border-b-2 border-primary/30 pb-2 mb-4 mt-2">
                                {children}
                            </h1>
                        ),
                        // 二级标题 — 章节标题
                        h2: ({ children }) => {
                            const text = String(children);
                            return (
                                <h2 className="text-base font-bold mt-6 mb-3 flex items-center gap-2">
                                    <span className="w-1 h-5 bg-primary rounded-full inline-block" />
                                    {text}
                                </h2>
                            );
                        },
                        // 三级标题 — 风险项标题（自动匹配风险等级标签）
                        h3: ({ children }) => {
                            const text = String(children);
                            let riskBadge = null;
                            let icon = null;

                            if (text.includes("高风险")) {
                                riskBadge = (
                                    <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-bold bg-red-500/15 text-red-600 border border-red-500/30">
                                        <ShieldAlert className="h-3 w-3" />高风险
                                    </span>
                                );
                                icon = <ShieldAlert className="h-4 w-4 text-red-500" />;
                            } else if (text.includes("中风险")) {
                                riskBadge = (
                                    <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-bold bg-yellow-500/15 text-yellow-700 border border-yellow-500/30">
                                        <AlertTriangle className="h-3 w-3" />中风险
                                    </span>
                                );
                                icon = <AlertTriangle className="h-4 w-4 text-yellow-500" />;
                            } else if (text.includes("低风险")) {
                                riskBadge = (
                                    <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-bold bg-green-500/15 text-green-600 border border-green-500/30">
                                        <ShieldCheck className="h-3 w-3" />低风险
                                    </span>
                                );
                                icon = <ShieldCheck className="h-4 w-4 text-green-500" />;
                            }

                            // 去掉文本中的风险等级文字避免重复
                            const cleanText = text
                                .replace(/（高风险）|（中风险）|（低风险）|\(高风险\)|\(中风险\)|\(低风险\)/g, "")
                                .trim();

                            return (
                                <h3 className="text-sm font-semibold mt-4 mb-2 flex items-center gap-2 flex-wrap">
                                    {icon}
                                    <span>{cleanText}</span>
                                    {riskBadge}
                                </h3>
                            );
                        },
                        // 段落 — 自动高亮风险关键词
                        p: ({ children }) => (
                            <p className="text-sm leading-relaxed mb-3 text-foreground/90">
                                {children}
                            </p>
                        ),
                        // 加粗 — 关键信息强调
                        strong: ({ children }) => {
                            const text = String(children);
                            // 风险等级关键词
                            if (text.includes("高风险") || text.includes("存在高风险")) {
                                return <strong className="text-red-600 font-bold">{children}</strong>;
                            }
                            if (text.includes("中风险")) {
                                return <strong className="text-yellow-600 font-bold">{children}</strong>;
                            }
                            if (text.includes("低风险") || text.includes("合规")) {
                                return <strong className="text-green-600 font-bold">{children}</strong>;
                            }
                            return <strong className="font-semibold text-foreground">{children}</strong>;
                        },
                        // 行内代码 — 字段名称
                        code: ({ children, className }) => {
                            // 块级代码
                            if (className) {
                                return <code className={className}>{children}</code>;
                            }
                            // 行内代码 (字段名)
                            return (
                                <code className="px-1.5 py-0.5 rounded bg-primary/10 text-primary text-xs font-mono font-medium">
                                    {children}
                                </code>
                            );
                        },
                        // 无序列表
                        ul: ({ children }) => (
                            <ul className="space-y-1.5 mb-3 ml-1">{children}</ul>
                        ),
                        // 有序列表
                        ol: ({ children }) => (
                            <ol className="space-y-2 mb-3 ml-1 list-none counter-reset-item">{children}</ol>
                        ),
                        // 列表项 — 带风险指示器
                        li: ({ children, ...props }) => {
                            const text = String(children);
                            let bulletColor = "bg-muted-foreground";

                            if (text.includes("高风险") || text.includes("严重") || text.includes("缺失")) {
                                bulletColor = "bg-red-500";
                            } else if (text.includes("中风险") || text.includes("异常") || text.includes("建议")) {
                                bulletColor = "bg-yellow-500";
                            } else if (text.includes("低风险") || text.includes("合格") || text.includes("正常")) {
                                bulletColor = "bg-green-500";
                            }

                            return (
                                <li className="flex items-start gap-2 text-sm leading-relaxed">
                                    <span className={`mt-2 w-1.5 h-1.5 rounded-full flex-shrink-0 ${bulletColor}`} />
                                    <span className="flex-1">{children}</span>
                                </li>
                            );
                        },
                        // 引用块 — 审计结论 / 建议
                        blockquote: ({ children }) => (
                            <blockquote className="border-l-3 border-primary/40 bg-primary/5 rounded-r-lg pl-4 pr-3 py-3 my-3 text-sm italic">
                                <Info className="h-4 w-4 text-primary/60 mb-1" />
                                {children}
                            </blockquote>
                        ),
                        // 表格
                        table: ({ children }) => (
                            <div className="overflow-x-auto my-3 rounded-lg border">
                                <table className="w-full text-sm">{children}</table>
                            </div>
                        ),
                        thead: ({ children }) => (
                            <thead className="bg-muted/50 text-xs uppercase tracking-wider">{children}</thead>
                        ),
                        th: ({ children }) => (
                            <th className="px-3 py-2 text-left font-semibold text-muted-foreground">{children}</th>
                        ),
                        td: ({ children }) => {
                            const text = String(children);
                            let cellClass = "px-3 py-2 border-t";

                            if (text.includes("高") || text.includes("严重")) {
                                cellClass += " text-red-600 font-medium";
                            } else if (text.includes("中") || text.includes("注意")) {
                                cellClass += " text-yellow-600 font-medium";
                            } else if (text.includes("低") || text.includes("正常") || text.includes("合格")) {
                                cellClass += " text-green-600 font-medium";
                            }

                            return <td className={cellClass}>{children}</td>;
                        },
                        // 分隔线
                        hr: () => (
                            <hr className="my-4 border-border/50" />
                        ),
                    }}
                >
                    {content}
                </ReactMarkdown>
            </div>
        </div>
    );
}
