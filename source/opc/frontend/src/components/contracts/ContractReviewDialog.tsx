import { Dialog, DialogContent, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import { ShieldAlert, ShieldCheck, AlertTriangle, Clock, RefreshCw, User, FileText } from "lucide-react";
import { AIReviewRenderer } from "@/components/common/AIReviewRenderer";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { api, type BackendAction } from "@/integrations/api/client";
import { useToast } from "@/hooks/use-toast";

interface ContractReviewDialogProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    contract: any;
}

const reviewIconMap = {
    ShieldAlert,
    ShieldCheck,
    AlertTriangle,
    Clock,
    RefreshCw,
};

export function ContractReviewDialog({ open, onOpenChange, contract }: ContractReviewDialogProps) {
    const { toast } = useToast();
    const queryClient = useQueryClient();

    const retriggerMutation = useMutation({
        mutationFn: async (action: BackendAction) => api.runAction(action),
        onSuccess: (_data, action) => {
            toast({ title: action.success_title || "已重新提交审查", description: action.success_message || "AI 正在重新分析合同" });
            queryClient.invalidateQueries({ queryKey: ["contracts-overview"] });
            onOpenChange(false);
        },
        onError: () => {
            toast({ title: "审查失败", description: "请稍后重试", variant: "destructive" });
        },
    });

    if (!contract) return null;

    const reviewDisplay = contract.ai_review_display || {};
    const RiskIcon = reviewIconMap[reviewDisplay.icon as keyof typeof reviewIconMap] || Clock;
    const reviewResult = contract.ai_review_result || "";
    const hasReview = Boolean(reviewDisplay.has_review) && reviewResult;
    const counterpartyBadge = contract.counterparty_type_badge;
    const emptyState = reviewDisplay.empty_state || {};
    const EmptyIcon = reviewIconMap[emptyState.icon as keyof typeof reviewIconMap] || Clock;
    const retriggerAction = contract.retrigger_review_action as BackendAction | null | undefined;

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent className="max-w-2xl max-h-[85vh] flex flex-col">
                <DialogHeader>
                    <DialogTitle className="flex items-center gap-2 text-lg">
                        <ShieldAlert className="h-5 w-5" />
                        AI 防坑卫士 — 合同审查报告
                    </DialogTitle>
                </DialogHeader>

                {/* 合同基本信息 */}
                <div className="grid grid-cols-2 gap-3 text-sm">
                    <div className="flex items-center gap-2">
                        <FileText className="h-4 w-4 text-muted-foreground" />
                        <span className="text-muted-foreground">合同:</span>
                        <span className="font-medium truncate">{contract.contract_name || "—"}</span>
                    </div>
                    {counterpartyBadge?.label && (
                        <div className="flex items-center gap-2">
                            <User className="h-4 w-4 text-muted-foreground" />
                            <span className="text-muted-foreground">对方身份:</span>
                            <Badge variant={counterpartyBadge.variant || "outline"} className="text-xs">
                                {counterpartyBadge.label}
                            </Badge>
                        </div>
                    )}
                </div>

                {/* 风险等级卡片 */}
                <div className={`rounded-lg border p-4 ${reviewDisplay.border_class_name || "border-muted"} ${reviewDisplay.class_name || "bg-muted text-muted-foreground"}`}>
                    <div className="flex items-center justify-between">
                        <div className="flex items-center gap-3">
                            <RiskIcon className="h-6 w-6" />
                            <div>
                                <p className="font-semibold text-base">{reviewDisplay.label || "暂无审查结果"}</p>
                                {contract.ai_reviewed_at_label && contract.ai_reviewed_at_label !== "-" && (
                                    <p className="text-xs opacity-70 mt-0.5">
                                        审查时间: {contract.ai_reviewed_at_label}
                                    </p>
                                )}
                            </div>
                        </div>
                        <Button
                            variant="outline"
                            size="sm"
                            className="gap-1"
                            onClick={() => {
                                const action = contract.retrigger_review_action as BackendAction | null | undefined;
                                if (action) retriggerMutation.mutate(action);
                            }}
                            disabled={retriggerMutation.isPending || !retriggerAction}
                        >
                            <RefreshCw className={`h-3.5 w-3.5 ${retriggerMutation.isPending ? "animate-spin" : ""}`} />
                            重新审查
                        </Button>
                    </div>
                </div>

                <Separator />

                {/* 审查报告内容 */}
                <ScrollArea className="flex-1 min-h-0">
                    {hasReview ? (
                        <AIReviewRenderer content={reviewResult} />
                    ) : (
                        <div className="flex flex-col items-center justify-center py-12 text-muted-foreground">
                            <EmptyIcon className={`h-8 w-8 mb-3 ${emptyState.icon_class_name || ""}`} />
                            <p className="text-sm">{emptyState.title || "暂无审查结果"}</p>
                            {emptyState.description && (
                                <p className="text-xs mt-1">{emptyState.description}</p>
                            )}
                        </div>
                    )}
                </ScrollArea>
            </DialogContent>
        </Dialog>
    );
}
