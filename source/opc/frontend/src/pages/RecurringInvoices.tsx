import { useState } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api, type BackendAction } from "@/integrations/api/client";
import { toast } from "sonner";
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
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
  Plus,
  RefreshCw,
  Bell,
  Trash2,
  Pencil,
  Pause,
  Play,
  DollarSign,
  Calendar,
} from "lucide-react";

const RECURRING_INVOICES_OVERVIEW_ENDPOINT =
  "/api/v1/recurring_invoices/overview";

interface RecurringInvoicesOverview {
  stats: {
    active_recurring: number;
    total_monthly: number;
    total_monthly_label: string;
    pending_reminders: number;
    sent_reminders: number;
  };
  recurring: any[];
  reminders: any[];
  clients: any[];
  invoices: any[];
  permissions?: {
    can_manage: boolean;
  };
  create_action: BackendAction | null;
  reminder_create_action: BackendAction | null;
  empty_messages?: {
    recurring: string;
    reminders: string;
  };
}

interface DeleteAction {
  method: "DELETE";
  endpoint: string;
  confirm_message: string;
  success_title: string;
  success_message: string;
}

interface RecurringFormOption {
  value: string;
  label: string;
}

interface RecurringFormOptions {
  frequency_options: RecurringFormOption[];
  reminder_type_options: RecurringFormOption[];
  default_values: {
    frequency: string;
    payment_terms_days: number;
    tax_rate: number;
    auto_send: number;
    reminder_type: string;
    channel: string;
  };
}

const emptyRecurringOverview: RecurringInvoicesOverview = {
  stats: {
    active_recurring: 0,
    total_monthly: 0,
    total_monthly_label: "¥0",
    pending_reminders: 0,
    sent_reminders: 0,
  },
  recurring: [],
  reminders: [],
  clients: [],
  invoices: [],
  permissions: {
    can_manage: false,
  },
  create_action: null,
  reminder_create_action: null,
  empty_messages: {
    recurring: "暂无周期发票",
    reminders: "暂无催款提醒",
  },
};

function BackendBadgeView({ badge }: { badge?: any }) {
  return (
    <Badge variant={badge?.variant || "outline"} className={badge?.class_name}>
      {badge?.label || "-"}
    </Badge>
  );
}

function ToggleActionIcon({ icon }: { icon?: string }) {
  return icon === "Pause" ? (
    <Pause className="h-4 w-4" />
  ) : (
    <Play className="h-4 w-4" />
  );
}

function RecurringInvoices() {
  const queryClient = useQueryClient();
  const [recurringDialogOpen, setRecurringDialogOpen] = useState(false);
  const [editingRecurring, setEditingRecurring] = useState<any>(null);
  const [reminderDialogOpen, setReminderDialogOpen] = useState(false);

  const { data: overview = emptyRecurringOverview, isLoading } = useQuery({
    queryKey: ["recurring-overview"],
    queryFn: () =>
      api.get<RecurringInvoicesOverview>(RECURRING_INVOICES_OVERVIEW_ENDPOINT),
  });
  const { data: formOptions } = useQuery({
    queryKey: ["recurring-form-options"],
    queryFn: () =>
      api.get<RecurringFormOptions>("/api/v1/recurring_invoices/form_options"),
  });

  const recurring = overview.recurring;
  const reminders = overview.reminders;
  const clients = overview.clients;
  const invoices = overview.invoices;
  const recurringStats = overview.stats;
  const frequencyOptions = formOptions?.frequency_options ?? [];
  const reminderTypeOptions = formOptions?.reminder_type_options ?? [];
  const recurringDefaultValues = formOptions?.default_values;

  const saveRecurring = useMutation({
    mutationFn: async ({
      action,
      values,
    }: {
      action: BackendAction;
      values: Record<string, unknown>;
    }) => {
      await api.runAction(action, values);
      return action;
    },
    onSuccess: (action) => {
      queryClient.invalidateQueries({ queryKey: ["recurring-overview"] });
      toast.success(action.success_message || "周期发票已保存");
      setRecurringDialogOpen(false);
      setEditingRecurring(null);
    },
  });
  const deleteRecurring = useMutation({
    mutationFn: async (action: DeleteAction) => {
      await api.runAction(action);
      return action;
    },
    onSuccess: (action) => {
      queryClient.invalidateQueries({ queryKey: ["recurring-overview"] });
      toast.success(action.success_message);
    },
  });
  const createReminder = useMutation({
    mutationFn: async ({
      action,
      values,
    }: {
      action: BackendAction;
      values: Record<string, unknown>;
    }) => {
      await api.runAction(action, values);
      return action;
    },
    onSuccess: (action) => {
      queryClient.invalidateQueries({ queryKey: ["recurring-overview"] });
      toast.success(action.success_message || "催款提醒已创建");
      setReminderDialogOpen(false);
    },
  });
  const deleteReminder = useMutation({
    mutationFn: async (action: DeleteAction) => {
      await api.runAction(action);
      return action;
    },
    onSuccess: (action) => {
      queryClient.invalidateQueries({ queryKey: ["recurring-overview"] });
      toast.success(action.success_message);
    },
  });

  const handleRecurringSubmit = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const fd = new FormData(e.currentTarget);
    const values: Record<string, any> = {};
    fd.forEach((v, k) => {
      if (v) values[k] = v;
    });
    if (values.auto_send) values.auto_send = values.auto_send === "1" ? 1 : 0;
    const action = editingRecurring
      ? editingRecurring.update_action
      : overview.create_action;
    if (!action) {
      toast.error("当前无法保存周期发票");
      return;
    }
    saveRecurring.mutate({ action, values });
  };

  const handleReminderSubmit = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const fd = new FormData(e.currentTarget);
    const values: Record<string, any> = {};
    fd.forEach((v, k) => {
      if (v) values[k] = v;
    });
    const action = overview.reminder_create_action;
    if (!action) {
      toast.error("当前无法创建催款提醒");
      return;
    }
    createReminder.mutate({ action, values });
  };

  const handleDeleteRecurring = (action?: DeleteAction | null) => {
    if (!action || !confirm(action.confirm_message)) return;
    deleteRecurring.mutate(action);
  };

  const handleDeleteReminder = (action?: DeleteAction | null) => {
    if (!action || !confirm(action.confirm_message)) return;
    deleteReminder.mutate(action);
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">周期发票 & 催款</h1>
          <p className="text-muted-foreground">
            自动生成周期性发票，智能催款提醒
          </p>
        </div>
      </div>

      {/* Stats */}
      <div className="grid grid-cols-4 gap-4">
        <Card>
          <CardContent className="pt-6">
            <div className="flex items-center gap-2">
              <RefreshCw className="h-5 w-5 text-blue-500" />
              <div>
                <div className="text-2xl font-bold">
                  {recurringStats.active_recurring}
                </div>
                <p className="text-xs text-muted-foreground">活跃周期发票</p>
              </div>
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="pt-6">
            <div className="flex items-center gap-2">
              <DollarSign className="h-5 w-5 text-green-500" />
              <div>
                <div className="text-2xl font-bold">
                  {recurringStats.total_monthly_label}
                </div>
                <p className="text-xs text-muted-foreground">月均收入</p>
              </div>
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="pt-6">
            <div className="flex items-center gap-2">
              <Bell className="h-5 w-5 text-yellow-500" />
              <div>
                <div className="text-2xl font-bold">
                  {recurringStats.pending_reminders}
                </div>
                <p className="text-xs text-muted-foreground">待发催款</p>
              </div>
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="pt-6">
            <div className="flex items-center gap-2">
              <Calendar className="h-5 w-5 text-purple-500" />
              <div>
                <div className="text-2xl font-bold">
                  {recurringStats.sent_reminders}
                </div>
                <p className="text-xs text-muted-foreground">已发催款</p>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>

      <Tabs defaultValue="recurring">
        <TabsList>
          <TabsTrigger value="recurring">周期发票</TabsTrigger>
          <TabsTrigger value="reminders">催款提醒</TabsTrigger>
        </TabsList>

        {/* Recurring Tab */}
        <TabsContent value="recurring" className="space-y-4">
          <div className="flex justify-end">
            <Dialog
              open={recurringDialogOpen}
              onOpenChange={(o) => {
                setRecurringDialogOpen(o);
                if (!o) setEditingRecurring(null);
              }}
            >
              <DialogTrigger asChild>
                <Button disabled={!overview.create_action}>
                  <Plus className="h-4 w-4 mr-2" /> 新建周期发票
                </Button>
              </DialogTrigger>
              <DialogContent className="max-w-lg">
                <DialogHeader>
                  <DialogTitle>
                    {editingRecurring ? "编辑周期发票" : "新建周期发票"}
                  </DialogTitle>
                </DialogHeader>
                <form onSubmit={handleRecurringSubmit} className="space-y-4">
                  <div>
                    <Label>标题 *</Label>
                    <Input
                      name="title"
                      required
                      defaultValue={editingRecurring?.title}
                    />
                  </div>
                  <div className="grid grid-cols-2 gap-4">
                    <div>
                      <Label>金额 *</Label>
                      <Input
                        name="amount"
                        type="number"
                        step="0.01"
                        required
                        defaultValue={editingRecurring?.amount}
                      />
                    </div>
                    <div>
                      <Label>频率 *</Label>
                      <Select
                        name="frequency"
                        defaultValue={
                          editingRecurring?.frequency ||
                          recurringDefaultValues?.frequency
                        }
                      >
                        <SelectTrigger>
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          {frequencyOptions.map((option) => (
                            <SelectItem key={option.value} value={option.value}>
                              {option.label}
                            </SelectItem>
                          ))}
                        </SelectContent>
                      </Select>
                    </div>
                  </div>
                  <div className="grid grid-cols-2 gap-4">
                    <div>
                      <Label>开始日期 *</Label>
                      <Input
                        name="start_date"
                        type="date"
                        required
                        defaultValue={editingRecurring?.start_date}
                      />
                    </div>
                    <div>
                      <Label>结束日期</Label>
                      <Input
                        name="end_date"
                        type="date"
                        defaultValue={editingRecurring?.end_date}
                      />
                    </div>
                  </div>
                  <div className="grid grid-cols-2 gap-4">
                    <div>
                      <Label>付款期限 (天)</Label>
                      <Input
                        name="payment_terms_days"
                        type="number"
                        defaultValue={
                          editingRecurring?.payment_terms_days ??
                          recurringDefaultValues?.payment_terms_days
                        }
                      />
                    </div>
                    <div>
                      <Label>税率 (%)</Label>
                      <Input
                        name="tax_rate"
                        type="number"
                        step="0.01"
                        defaultValue={editingRecurring?.tax_rate}
                      />
                    </div>
                  </div>
                  <div>
                    <Label>客户</Label>
                    <Select
                      name="client_id"
                      defaultValue={editingRecurring?.client_id}
                    >
                      <SelectTrigger>
                        <SelectValue placeholder="选择客户" />
                      </SelectTrigger>
                      <SelectContent>
                        {clients.map((c: any) => (
                          <SelectItem key={c.id} value={c.id}>
                            {c.name}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </div>
                  <div>
                    <Label>备注</Label>
                    <Textarea
                      name="notes"
                      defaultValue={editingRecurring?.notes}
                    />
                  </div>
                  <Button type="submit" className="w-full">
                    {editingRecurring ? "保存" : "创建"}
                  </Button>
                </form>
              </DialogContent>
            </Dialog>
          </div>
          <Card>
            <CardContent className="p-0">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>标题</TableHead>
                    <TableHead>客户</TableHead>
                    <TableHead>金额</TableHead>
                    <TableHead>频率</TableHead>
                    <TableHead>下次生成</TableHead>
                    <TableHead>状态</TableHead>
                    <TableHead>已生成</TableHead>
                    <TableHead className="text-right">操作</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {isLoading ? (
                    <TableRow>
                      <TableCell colSpan={8} className="text-center py-8">
                        加载中...
                      </TableCell>
                    </TableRow>
                  ) : recurring.length === 0 ? (
                    <TableRow>
                      <TableCell
                        colSpan={8}
                        className="text-center py-8 text-muted-foreground"
                      >
                        {overview.empty_messages?.recurring}
                      </TableCell>
                    </TableRow>
                  ) : (
                    recurring.map((r: any) => (
                      <TableRow key={r.id}>
                        <TableCell className="font-medium">{r.title}</TableCell>
                        <TableCell>{r.client_name || "-"}</TableCell>
                        <TableCell>{r.amount_label}</TableCell>
                        <TableCell>{r.frequency_label}</TableCell>
                        <TableCell>{r.next_invoice_date_label}</TableCell>
                        <TableCell>
                          <BackendBadgeView badge={r.status_badge} />
                        </TableCell>
                        <TableCell>{r.total_generated_label}</TableCell>
                        <TableCell className="text-right space-x-1">
                          {r.can_edit && r.update_action && (
                            <>
                              <Button
                                variant="ghost"
                                size="sm"
                                onClick={() => {
                                  const action = r.toggle_action;
                                  if (action) {
                                    saveRecurring.mutate({
                                      action,
                                      values: action.default_values ?? {},
                                    });
                                  }
                                }}
                              >
                                <ToggleActionIcon
                                  icon={r.toggle_action?.icon}
                                />
                              </Button>
                              <Button
                                variant="ghost"
                                size="sm"
                                onClick={() => {
                                  setEditingRecurring(r);
                                  setRecurringDialogOpen(true);
                                }}
                              >
                                <Pencil className="h-4 w-4" />
                              </Button>
                            </>
                          )}
                          {r.delete_action && (
                            <Button
                              variant="ghost"
                              size="sm"
                              onClick={() => handleDeleteRecurring(r.delete_action)}
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
        </TabsContent>

        {/* Reminders Tab */}
        <TabsContent value="reminders" className="space-y-4">
          <div className="flex justify-end">
            <Dialog
              open={reminderDialogOpen}
              onOpenChange={setReminderDialogOpen}
            >
              <DialogTrigger asChild>
                <Button disabled={!overview.reminder_create_action}>
                  <Plus className="h-4 w-4 mr-2" /> 新建催款提醒
                </Button>
              </DialogTrigger>
              <DialogContent>
                <DialogHeader>
                  <DialogTitle>新建催款提醒</DialogTitle>
                </DialogHeader>
                <form onSubmit={handleReminderSubmit} className="space-y-4">
                  <div>
                    <Label>关联发票 *</Label>
                    <Select name="invoice_id">
                      <SelectTrigger>
                        <SelectValue placeholder="选择发票" />
                      </SelectTrigger>
                      <SelectContent>
                        {invoices.map((i: any) => (
                          <SelectItem key={i.id} value={i.id}>
                            {i.invoice_number || i.id}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </div>
                  <div>
                    <Label>提醒类型 *</Label>
                    <Select
                      name="reminder_type"
                      defaultValue={recurringDefaultValues?.reminder_type}
                    >
                      <SelectTrigger>
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        {reminderTypeOptions.map((option) => (
                          <SelectItem key={option.value} value={option.value}>
                            {option.label}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </div>
                  <div>
                    <Label>发送日期 *</Label>
                    <Input name="scheduled_date" type="date" required />
                  </div>
                  <div>
                    <Label>主题</Label>
                    <Input name="subject" />
                  </div>
                  <div>
                    <Label>内容</Label>
                    <Textarea name="message" />
                  </div>
                  <Button type="submit" className="w-full">
                    创建
                  </Button>
                </form>
              </DialogContent>
            </Dialog>
          </div>
          <Card>
            <CardContent className="p-0">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>发票</TableHead>
                    <TableHead>类型</TableHead>
                    <TableHead>计划日期</TableHead>
                    <TableHead>状态</TableHead>
                    <TableHead>已发送</TableHead>
                    <TableHead>渠道</TableHead>
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
                  ) : reminders.length === 0 ? (
                    <TableRow>
                      <TableCell
                        colSpan={7}
                        className="text-center py-8 text-muted-foreground"
                      >
                        {overview.empty_messages?.reminders}
                      </TableCell>
                    </TableRow>
                  ) : (
                    reminders.map((r: any) => (
                      <TableRow key={r.id}>
                        <TableCell>{r.invoice_display}</TableCell>
                        <TableCell>{r.reminder_type_label}</TableCell>
                        <TableCell>{r.scheduled_date_label}</TableCell>
                        <TableCell>
                          <BackendBadgeView badge={r.status_badge} />
                        </TableCell>
                        <TableCell>{r.sent_at_label}</TableCell>
                        <TableCell>{r.channel_label}</TableCell>
                        <TableCell className="text-right">
                          {r.delete_action && (
                            <Button
                              variant="ghost"
                              size="sm"
                              onClick={() => handleDeleteReminder(r.delete_action)}
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
        </TabsContent>
      </Tabs>
    </div>
  );
}

export default RecurringInvoices;
