import { useEffect, useState } from "react";
import { useSearchParams } from "react-router-dom";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api, pdf, type BackendAction, type DeleteAction } from "@/integrations/api/client";
import { API_ENDPOINTS } from "@/lib/api";
import { toast } from "sonner";
import { QueryStateGate } from "@/components/common/QueryStateGate";
import { apiErrorMessage } from "@/lib/apiError";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Plus, Trash2, Pencil, ArrowRight, FileText, Download } from "lucide-react";

interface QuotesOverview {
  stats: {
    total: number;
    draft: number;
    sent: number;
    total_amount: number;
    total_amount_label: string;
  };
  next_quote_number: string;
  quotes: any[];
  clients: any[];
  status_options: any[];
  create_action: BackendAction | null;
  permissions?: {
    can_manage: boolean;
  };
  empty_message?: string;
}

const emptyQuotesOverview: QuotesOverview = {
  stats: {
    total: 0,
    draft: 0,
    sent: 0,
    total_amount: 0,
    total_amount_label: "¥0",
  },
  next_quote_number: "",
  quotes: [],
  clients: [],
  status_options: [],
  create_action: null,
  permissions: {
    can_manage: false,
  },
  empty_message: "暂无报价单",
};

function QuotesManagement() {
  const queryClient = useQueryClient();
  const [dialogOpen, setDialogOpen] = useState(false);
  const [editingQuote, setEditingQuote] = useState<any>(null);
  const [searchParams, setSearchParams] = useSearchParams();
  const [prefilledClientId, setPrefilledClientId] = useState<
    string | undefined
  >(undefined);

  const {
    data: overview = emptyQuotesOverview,
    isLoading,
    isError,
    refetch,
  } = useQuery({
    queryKey: ["quotes-overview"],
    queryFn: () => api.get<QuotesOverview>(API_ENDPOINTS.quotesOverview),
  });

  // Open the create dialog with a preselected client when arriving from CRM.
  useEffect(() => {
    const clientId = searchParams.get("client_id");
    if (!clientId) return;
    setEditingQuote(null);
    setPrefilledClientId(clientId);
    setDialogOpen(true);
    searchParams.delete("client_id");
    setSearchParams(searchParams, { replace: true });
  }, [searchParams, setSearchParams]);

  const saveQuote = useMutation({
    mutationFn: ({
      action,
      values,
    }: {
      action: BackendAction;
      values: Record<string, any>;
    }) => api.runAction(action, values),
    onSuccess: (_data, variables) => {
      queryClient.invalidateQueries({ queryKey: ["quotes-overview"] });
      toast.success(variables.action.success_message || "报价单已保存");
      setDialogOpen(false);
      setEditingQuote(null);
    },
    onError: (e) => {
      toast.error(apiErrorMessage(e));
    },
  });

  const deleteQuote = useMutation({
    mutationFn: async (action: DeleteAction) => {
      await api.runAction(action);
      return action;
    },
    onSuccess: (action) => {
      queryClient.invalidateQueries({ queryKey: ["quotes-overview"] });
      toast.success(action.success_message);
    },
    onError: (e) => {
      toast.error(apiErrorMessage(e));
    },
  });

  const convertQuote = useMutation({
    mutationFn: (action: BackendAction) =>
      api.runAction<{ invoice_number?: string }>(action),
    onSuccess: (result) => {
      queryClient.invalidateQueries({ queryKey: ["quotes-overview"] });
      queryClient.invalidateQueries({ queryKey: ["onboarding-progress"] });
      toast.success(
        result?.invoice_number
          ? `已生成发票 ${result.invoice_number}`
          : "已根据报价单生成发票",
      );
    },
    onError: () => {
      toast.error("转发票失败，请确认报价单状态为「已接受」");
    },
  });

  const handleSubmit = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const formData = new FormData(e.currentTarget);
    const values: Record<string, any> = {};
    formData.forEach((v, k) => {
      if (v) values[k] = v;
    });

    const action = editingQuote
      ? editingQuote.update_action
      : overview.create_action;
    if (!action) {
      toast.error("当前账号没有保存报价单的权限");
      return;
    }
    saveQuote.mutate({ action, values });
  };

  const handleDeleteQuote = (action?: DeleteAction | null) => {
    if (!action || !confirm(action.confirm_message)) return;
    deleteQuote.mutate(action);
  };

  const stats = overview.stats;
  const quotes = overview.quotes;
  const clients = overview.clients;
  const statusOptions = overview.status_options;

  return (
    <QueryStateGate isError={isError} refetch={refetch}>
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">报价单管理</h1>
          <p className="text-muted-foreground">
            创建、发送和管理报价单，支持转发票
          </p>
        </div>
        <Dialog
          open={dialogOpen}
          onOpenChange={(open) => {
            setDialogOpen(open);
            if (!open) {
              setEditingQuote(null);
              setPrefilledClientId(undefined);
            }
          }}
        >
          <DialogTrigger asChild>
            <Button>
              <Plus className="h-4 w-4 mr-2" /> 新建报价单
            </Button>
          </DialogTrigger>
          <DialogContent className="max-w-lg">
            <DialogHeader>
              <DialogTitle>
                {editingQuote ? "编辑报价单" : "新建报价单"}
              </DialogTitle>
            </DialogHeader>
            <form onSubmit={handleSubmit} className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <Label>报价单号</Label>
                  <Input
                    name="quote_number"
                    placeholder="留空自动生成（如 QUO-2026-0001）"
                    defaultValue={
                      editingQuote?.quote_number || overview.next_quote_number
                    }
                  />
                </div>
                <div>
                  <Label>客户 *</Label>
                  <Select
                    name="client_id"
                    defaultValue={editingQuote?.client_id ?? prefilledClientId}
                  >
                    <SelectTrigger>
                      <SelectValue placeholder="选择客户" />
                    </SelectTrigger>
                    <SelectContent>
                      {clients.map((client: any) => (
                        <SelectItem key={client.id} value={client.id}>
                          {client.name}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
                <div className="col-span-2">
                  <Label>标题</Label>
                  <Input name="title" defaultValue={editingQuote?.title} />
                </div>
                <div>
                  <Label>小计</Label>
                  <Input
                    name="subtotal"
                    type="number"
                    step="0.01"
                    defaultValue={editingQuote?.subtotal}
                  />
                </div>
                <div>
                  <Label>税率 (%)</Label>
                  <Input
                    name="tax_rate"
                    type="number"
                    step="0.01"
                    defaultValue={editingQuote?.tax_rate}
                  />
                </div>
                <div>
                  <Label>税额</Label>
                  <Input
                    name="tax_amount"
                    type="number"
                    step="0.01"
                    defaultValue={editingQuote?.tax_amount}
                  />
                </div>
                <div>
                  <Label>折扣</Label>
                  <Input
                    name="discount"
                    type="number"
                    step="0.01"
                    defaultValue={editingQuote?.discount}
                  />
                </div>
                <div>
                  <Label>总计</Label>
                  <Input
                    name="total"
                    type="number"
                    step="0.01"
                    defaultValue={editingQuote?.total}
                  />
                </div>
                <div>
                  <Label>有效期至</Label>
                  <Input
                    name="valid_until"
                    type="date"
                    defaultValue={editingQuote?.valid_until}
                  />
                </div>
              </div>
              <div>
                <Label>状态</Label>
                <Select
                  name="status"
                  defaultValue={editingQuote?.status || "draft"}
                >
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {statusOptions.map((status: any) => (
                      <SelectItem key={status.value} value={status.value}>
                        {status.label}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              <div>
                <Label>备注</Label>
                <Textarea name="notes" defaultValue={editingQuote?.notes} />
              </div>
              <div>
                <Label>条款</Label>
                <Textarea name="terms" defaultValue={editingQuote?.terms} />
              </div>
              <Button type="submit" className="w-full">
                {editingQuote ? "保存修改" : "创建报价单"}
              </Button>
            </form>
          </DialogContent>
        </Dialog>
      </div>

      <div className="grid grid-cols-4 gap-4">
        <Card>
          <CardContent className="pt-6">
            <div className="text-2xl font-bold">{stats.total}</div>
            <p className="text-xs text-muted-foreground">总报价单</p>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="pt-6">
            <div className="text-2xl font-bold text-yellow-600">
              {stats.draft}
            </div>
            <p className="text-xs text-muted-foreground">草稿</p>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="pt-6">
            <div className="text-2xl font-bold text-blue-600">{stats.sent}</div>
            <p className="text-xs text-muted-foreground">已发送</p>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="pt-6">
            <div className="text-2xl font-bold">{stats.total_amount_label}</div>
            <p className="text-xs text-muted-foreground">总金额</p>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardContent className="p-0">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>报价单号</TableHead>
                <TableHead>客户</TableHead>
                <TableHead>标题</TableHead>
                <TableHead>状态</TableHead>
                <TableHead>总金额</TableHead>
                <TableHead>有效期</TableHead>
                <TableHead className="text-right">操作</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {isLoading ? (
                <TableRow>
                  <TableCell colSpan={7} className="text-center py-8">
                    加载中...
                  </TableCell>
                </TableRow>
              ) : quotes.length === 0 ? (
                <TableRow>
                  <TableCell colSpan={7} className="py-12">
                    <div className="flex flex-col items-center justify-center text-center">
                      <FileText className="h-16 w-16 text-muted-foreground mb-4" />
                      <h3 className="text-lg font-semibold mb-2">
                        {overview.empty_message}
                      </h3>
                      <p className="text-muted-foreground mb-6 max-w-md">
                        给客户出一份报价。客户确认后可一键转成发票，省去重复录入。
                      </p>
                      {overview.create_action && (
                        <Button
                          size="lg"
                          onClick={() => {
                            setEditingQuote(null);
                            setDialogOpen(true);
                          }}
                        >
                          <Plus className="mr-2 h-4 w-4" /> 新建报价单
                        </Button>
                      )}
                    </div>
                  </TableCell>
                </TableRow>
              ) : (
                quotes.map((quote: any) => (
                  <TableRow key={quote.id}>
                    <TableCell className="font-mono text-sm">
                      {quote.quote_number}
                    </TableCell>
                    <TableCell>{quote.client_name}</TableCell>
                    <TableCell>{quote.title || "-"}</TableCell>
                    <TableCell>
                      <Badge
                        variant={quote.status_badge?.variant || "outline"}
                        className={quote.status_badge?.class_name}
                      >
                        {quote.status_badge?.label}
                      </Badge>
                    </TableCell>
                    <TableCell className="font-medium">
                      {quote.total_label}
                    </TableCell>
                    <TableCell>{quote.valid_until_label}</TableCell>
                    <TableCell className="text-right space-x-1">
                      {quote.can_edit && quote.update_action && (
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => {
                            setEditingQuote(quote);
                            setDialogOpen(true);
                          }}
                        >
                          <Pencil className="h-4 w-4" />
                        </Button>
                      )}
                      <Button
                        variant="ghost"
                        size="sm"
                        title="下载 PDF"
                        onClick={() =>
                          pdf
                            .downloadQuote(quote.id, `${quote.quote_number}.pdf`)
                            .catch(() => toast.error("PDF 下载失败"))
                        }
                      >
                        <Download className="h-4 w-4" />
                      </Button>
                      {quote.convert_action && (
                        <Button
                          variant="ghost"
                          size="sm"
                          title="转为发票"
                          disabled={convertQuote.isPending}
                          onClick={() => convertQuote.mutate(quote.convert_action)}
                        >
                          <ArrowRight className="h-4 w-4 text-green-500" />
                        </Button>
                      )}
                      {quote.delete_action && (
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => handleDeleteQuote(quote.delete_action)}
                        >
                          <Trash2 className="h-4 w-4 text-red-500" />
                        </Button>
                      )}
                    </TableCell>
                  </TableRow>
                ))
              )}
            </TableBody>
          </Table>
        </CardContent>
      </Card>
    </div>
    </QueryStateGate>
  );
}

export default QuotesManagement;
