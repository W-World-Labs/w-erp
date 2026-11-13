import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api, type BackendAction } from "@/integrations/api/client";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import {
  AlertTriangle,
  Clock3,
  FileText,
  Send,
  TrendingUp,
} from "lucide-react";
import { toast } from "sonner";

type ArRiskTone = "green" | "yellow" | "orange" | "red" | "neutral";

interface ArSummaryCard {
  key: string;
  label: string;
  value_label: string;
  risk_tone: ArRiskTone;
}

interface ArInvoice {
  id: string;
  invoice_number: string;
  supplier?: string;
  due_date_label: string;
  amount_label: string;
  days_overdue_label: string;
  risk_tone: ArRiskTone;
  collection_action?: BackendAction | null;
  collection_history_count?: number;
  collection_history?: Array<{
    id: string;
    level_label: string;
    channel_name: string;
    status: string;
    status_label: string;
    sent_at?: string | null;
    last_attempt_at?: string | null;
  }>;
}

interface ArBucket {
  key: string;
  label: string;
  description: string;
  risk_tone: ArRiskTone;
  count: number;
  total_label: string;
  empty_message: string;
  invoices: ArInvoice[];
  badge?: {
    label: string;
    risk_tone: ArRiskTone;
  };
}

interface ArAgingOverview {
  total_receivable_label: string;
  total_overdue_label: string;
  overdue_rate_label: string;
  total_invoices: number;
  summary_cards: ArSummaryCard[];
  bucket_sections: ArBucket[];
}

const AR_AGING_ENDPOINT = "/api/v1/ar/aging";

const emptyArAging: ArAgingOverview = {
  total_receivable_label: "¥0",
  total_overdue_label: "¥0",
  overdue_rate_label: "0.0%",
  total_invoices: 0,
  summary_cards: [],
  bucket_sections: [],
};

const toneClasses: Record<
  ArRiskTone,
  {
    border: string;
    background: string;
    text: string;
    dot: string;
    badge: string;
  }
> = {
  green: {
    border: "border-emerald-200",
    background: "bg-emerald-50/70",
    text: "text-emerald-700",
    dot: "bg-emerald-500",
    badge: "border-emerald-200 bg-emerald-50 text-emerald-700",
  },
  yellow: {
    border: "border-yellow-200",
    background: "bg-yellow-50/70",
    text: "text-yellow-700",
    dot: "bg-yellow-500",
    badge: "border-yellow-200 bg-yellow-50 text-yellow-700",
  },
  orange: {
    border: "border-orange-200",
    background: "bg-orange-50/70",
    text: "text-orange-700",
    dot: "bg-orange-500",
    badge: "border-orange-200 bg-orange-50 text-orange-700",
  },
  red: {
    border: "border-red-200",
    background: "bg-red-50/70",
    text: "text-red-700",
    dot: "bg-red-500",
    badge: "border-red-200 bg-red-50 text-red-700",
  },
  neutral: {
    border: "border-border",
    background: "bg-muted/30",
    text: "text-foreground",
    dot: "bg-muted-foreground",
    badge: "border-border bg-background text-foreground",
  },
};

function toneFor(tone: ArRiskTone) {
  return toneClasses[tone] || toneClasses.neutral;
}

export function AccountsReceivableAgingPanel() {
  const queryClient = useQueryClient();
  const {
    data: arAging = emptyArAging,
    isError,
    isLoading,
  } = useQuery({
    queryKey: ["accounts-receivable-aging"],
    queryFn: () => api.get<ArAgingOverview>(AR_AGING_ENDPOINT),
  });

  const collectionMutation = useMutation({
    mutationFn: async (action: BackendAction) => {
      await api.runAction(action);
      return action;
    },
    onSuccess: (action) => {
      queryClient.invalidateQueries({
        queryKey: ["accounts-receivable-aging"],
      });
      queryClient.invalidateQueries({
        queryKey: ["payments-invoice-reminders"],
      });
      toast.success(action.success_message || "催收提醒已生成");
    },
    onError: () => {
      toast.error("生成催收提醒失败");
    },
  });

  const handleCollectionReminder = (action: BackendAction) => {
    collectionMutation.mutate(action);
  };

  if (isLoading) {
    return (
      <section className="rounded-lg border bg-card p-6">
        <div className="flex items-center gap-2 text-sm text-muted-foreground">
          <Clock3 className="h-4 w-4" />
          加载应收账龄...
        </div>
      </section>
    );
  }

  if (isError) {
    return (
      <section className="rounded-lg border border-destructive/30 bg-card p-6">
        <div className="flex items-center gap-2 text-sm text-destructive">
          <AlertTriangle className="h-4 w-4" />
          应收账龄暂时不可用
        </div>
      </section>
    );
  }

  return (
    <section className="rounded-lg border bg-card p-6">
      <div className="mb-5 flex flex-col gap-3 md:flex-row md:items-start md:justify-between">
        <div>
          <div className="flex items-center gap-2">
            <TrendingUp className="h-5 w-5 text-primary" />
            <h2 className="text-xl font-semibold">应收账龄看板</h2>
          </div>
          <p className="mt-1 text-sm text-muted-foreground">
            应收总额 {arAging.total_receivable_label}，逾期金额{" "}
            {arAging.total_overdue_label}，共 {arAging.total_invoices} 张票据
          </p>
        </div>
        <Badge variant="outline" className="w-fit">
          逾期率 {arAging.overdue_rate_label}
        </Badge>
      </div>

      <div className="mb-5 grid gap-3 md:grid-cols-4">
        {arAging.summary_cards.map((card) => {
          const tone = toneFor(card.risk_tone);
          return (
            <div
              key={card.key}
              className={cn(
                "rounded-md border p-4",
                tone.border,
                tone.background,
              )}
            >
              <div className="text-sm text-muted-foreground">{card.label}</div>
              <div className={cn("mt-1 text-2xl font-semibold", tone.text)}>
                {card.value_label}
              </div>
            </div>
          );
        })}
      </div>

      <div className="grid gap-3 lg:grid-cols-3">
        {arAging.bucket_sections.map((bucket) => {
          const tone = toneFor(bucket.risk_tone);
          return (
            <div
              key={bucket.key}
              className={cn("rounded-md border p-4", tone.border)}
            >
              <div className="mb-3 flex items-start justify-between gap-3">
                <div>
                  <div className="flex items-center gap-2">
                    <span className={cn("h-2.5 w-2.5 rounded-full", tone.dot)} />
                    <h3 className="font-semibold">{bucket.label}</h3>
                  </div>
                  <p className="mt-1 text-xs text-muted-foreground">
                    {bucket.description}
                  </p>
                </div>
                <Badge variant="outline" className={tone.badge}>
                  {bucket.badge?.label}
                </Badge>
              </div>

              <div className="mb-3 flex items-baseline justify-between border-b pb-3">
                <span className="text-sm text-muted-foreground">
                  {bucket.count} 张
                </span>
                <span className={cn("text-lg font-semibold", tone.text)}>
                  {bucket.total_label}
                </span>
              </div>

              {bucket.invoices.length === 0 ? (
                <div className="py-5 text-center text-sm text-muted-foreground">
                  {bucket.empty_message}
                </div>
              ) : (
                <div className="space-y-2">
                  {bucket.invoices.slice(0, 3).map((invoice) => {
                    const invoiceTone = toneFor(invoice.risk_tone);
                    return (
                      <div
                        key={invoice.id}
                        className="rounded-md border bg-background p-3"
                      >
                        <div className="flex items-start justify-between gap-3">
                          <div className="min-w-0">
                            <div className="flex items-center gap-2">
                              <FileText className="h-4 w-4 shrink-0 text-muted-foreground" />
                              <span className="truncate font-medium">
                                {invoice.invoice_number}
                              </span>
                            </div>
                            <div className="mt-1 text-xs text-muted-foreground">
                              {invoice.supplier || "-"} · 到期{" "}
                              {invoice.due_date_label}
                            </div>
                          </div>
                          <div className="text-right">
                            <div className="font-medium">
                              {invoice.amount_label}
                            </div>
                            <div className={cn("text-xs", invoiceTone.text)}>
                              {invoice.days_overdue_label}
                            </div>
                          </div>
                        </div>
                        {invoice.collection_action && (
                          <Button
                            type="button"
                            variant="outline"
                            size="sm"
                            className="mt-3 h-8 w-full justify-start"
                            disabled={collectionMutation.isPending}
                            onClick={() =>
                              handleCollectionReminder(
                                invoice.collection_action!,
                              )
                            }
                          >
                            <Send className="mr-2 h-3.5 w-3.5" />
                            {invoice.collection_action.label}
                          </Button>
                        )}
                        {(invoice.collection_history_count || 0) > 0 && (
                          <div className="mt-3 border-t pt-2">
                            <div className="mb-1 text-xs font-medium text-muted-foreground">
                              自动催收历史（{invoice.collection_history_count}）
                            </div>
                            <div className="space-y-1">
                              {invoice.collection_history
                                ?.slice(0, 2)
                                .map((delivery) => (
                                  <div
                                    key={delivery.id}
                                    className="flex items-center justify-between gap-2 text-xs"
                                  >
                                    <span className="truncate">
                                      {delivery.level_label} · {delivery.channel_name}
                                    </span>
                                    <Badge
                                      variant={
                                        delivery.status === "sent"
                                          ? "secondary"
                                          : delivery.status === "failed"
                                            ? "destructive"
                                            : "outline"
                                      }
                                      className="shrink-0"
                                    >
                                      {delivery.status_label}
                                    </Badge>
                                  </div>
                                ))}
                            </div>
                          </div>
                        )}
                      </div>
                    );
                  })}
                </div>
              )}
            </div>
          );
        })}
      </div>
    </section>
  );
}
