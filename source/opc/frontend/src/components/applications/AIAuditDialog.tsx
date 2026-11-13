import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription } from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Loader2, Brain, CheckCircle2, AlertCircle, TrendingUp, Target, Lightbulb } from "lucide-react";
import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { api, type BackendAction } from "@/integrations/api/client";
import { toast } from "sonner";
import { Card, CardContent } from "@/components/ui/card";
import { AIReviewRenderer } from '@/components/common/AIReviewRenderer';

type AuditType = 'application' | 'material' | 'contract' | 'employee' | 'budget' | 'project';

type AIAuditRequestPayload = {
  audit_type: AuditType;
  target_id?: string;
  [key: string]: unknown;
};

type AIAuditContext = {
  audit_type: AuditType | string;
  heading: string;
  description: string;
  feature_items: Array<{
    icon: string;
    label: string;
  }>;
  start_button_label: string;
  loading_label: string;
  success_message: string;
  error_message: string;
  audit_action: BackendAction | null;
};

interface AIAuditDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  auditType: AuditType;
  targetId?: string;
  auditRequest?: AIAuditRequestPayload;
  title: string;
}

const defaultAuditContext: AIAuditContext = {
  audit_type: "application",
  heading: "AI 审计",
  description: "",
  feature_items: [],
  start_button_label: "开始 AI 审核",
  loading_label: "AI 分析中...",
  success_message: "AI 审核完成",
  error_message: "AI 审核失败，请检查 AI 配置",
  audit_action: null,
};

const auditFeatureIcons = {
  CheckCircle2,
  AlertCircle,
  TrendingUp,
  Target,
  Lightbulb,
} as const;

export function AIAuditDialog({
  open,
  onOpenChange,
  auditType,
  targetId,
  auditRequest,
  title
}: AIAuditDialogProps) {
  const [loading, setLoading] = useState(false);
  const [analysis, setAnalysis] = useState<string>('');
  const [hasAnalysis, setHasAnalysis] = useState(false);

  const { data: auditContext = defaultAuditContext } = useQuery({
    queryKey: ["ai-audit-context", auditType],
    queryFn: () =>
      api.get<AIAuditContext>(
        `/api/ai/audit-context?audit_type=${encodeURIComponent(auditType)}`,
      ),
    enabled: open,
  });

  const handleAudit = async () => {
    setLoading(true);
    setAnalysis('');
    setHasAnalysis(false);

    try {
      if (!auditContext.audit_action) {
        toast.error('后端未返回 AI 审核动作');
        return;
      }

      const result = await api.runAction<{ analysis: string }>(auditContext.audit_action, {
        ...(auditRequest || {
          audit_type: auditType,
          target_id: targetId,
        }),
        audit_type: auditType,
      });

      setAnalysis(result?.analysis || '');
      setHasAnalysis(true);
      toast.success(auditContext.success_message);
    } catch (error: any) {
      console.error('AI 审核失败:', error);
      toast.error(error.message || auditContext.error_message);
    } finally {
      setLoading(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-4xl max-h-[90vh]">
        <DialogHeader>
          <div className="flex items-center gap-2">
            <Brain className="h-5 w-5 text-primary" />
            <DialogTitle>AI 智能审核</DialogTitle>
          </div>
          <DialogDescription>
            {title}
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4">
          {!hasAnalysis ? (
            <Card>
              <CardContent className="pt-6">
                <div className="text-center space-y-4">
                  <Brain className="h-16 w-16 mx-auto text-primary opacity-50" />
                  <div>
                    <h3 className="font-semibold mb-2">
                      {auditContext.heading}
                    </h3>
                    <p className="text-sm text-muted-foreground mb-4">
                      {auditContext.description}
                    </p>
                    <div className="flex flex-wrap gap-2 justify-center text-xs text-muted-foreground">
                      {auditContext.feature_items.map((item) => {
                        const FeatureIcon = auditFeatureIcons[item.icon as keyof typeof auditFeatureIcons] || CheckCircle2;
                        return (
                          <div key={`${item.icon}:${item.label}`} className="flex items-center gap-1">
                            <FeatureIcon className="h-3 w-3" />
                            <span>{item.label}</span>
                          </div>
                        );
                      })}
                    </div>
                  </div>
                  <Button
                    onClick={handleAudit}
                    disabled={loading}
                    size="lg"
                    className="w-full max-w-xs"
                  >
                    {loading ? (
                      <>
                        <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                        {auditContext.loading_label}
                      </>
                    ) : (
                      <>
                        <Brain className="mr-2 h-4 w-4" />
                        {auditContext.start_button_label}
                      </>
                    )}
                  </Button>
                </div>
              </CardContent>
            </Card>
          ) : (
            <>
              <ScrollArea className="h-[60vh] pr-4">
                <div className="pr-4">
                  <AIReviewRenderer content={analysis} />
                </div>
              </ScrollArea>

              <div className="flex justify-end gap-2 pt-4 border-t">
                <Button
                  variant="outline"
                  onClick={() => {
                    setAnalysis('');
                    setHasAnalysis(false);
                  }}
                >
                  重新审核
                </Button>
                <Button onClick={() => onOpenChange(false)}>
                  关闭
                </Button>
              </div>
            </>
          )}
        </div>
      </DialogContent>
    </Dialog>
  );
}
