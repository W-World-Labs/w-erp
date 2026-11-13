import { useState } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api, type BackendAction } from "@/integrations/api/client";
import { toast } from "sonner";
import { QueryStateGate } from "@/components/common/QueryStateGate";
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
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import {
  Calendar,
  Clock,
  DollarSign,
  FileText,
  Play,
  Plus,
  Square,
  Trash2,
  TrendingUp,
} from "lucide-react";

const TIME_ENTRIES_OVERVIEW_ENDPOINT = "/api/v1/time_entries/overview";

interface DisplayOption {
  value: string;
  label: string;
}

interface BadgeDisplay {
  label: string;
  variant?: "default" | "secondary" | "destructive" | "outline";
}

interface DeleteAction {
  method: "DELETE";
  endpoint: string;
  confirm_message: string;
  success_title: string;
  success_message: string;
}

interface TimeEntryRow {
  id: string;
  description: string;
  project_name: string;
  client_name: string;
  date_label: string;
  duration_label: string;
  billable_badge: BadgeDisplay;
  amount_label: string;
  can_delete: boolean;
  delete_action: DeleteAction | null;
}

interface TimeStats {
  today_minutes_label: string;
  total_records_label: string;
  billable_minutes_label: string;
  billable_amount_label: string;
  unbilled_count: number;
}

interface InvoiceAction {
  can_generate: boolean;
  label: string;
  confirm_message: string;
  action: BackendAction | null;
}

interface ActiveTimer {
  is_running: boolean;
  id?: string;
  description?: string;
  project_id?: string | null;
  client_id?: string | null;
  project_name?: string;
  client_name?: string;
  elapsed_label: string;
  start_action: BackendAction | null;
  stop_action: BackendAction | null;
}

interface TimeEntriesOverview {
  stats: TimeStats;
  entries: TimeEntryRow[];
  project_options: DisplayOption[];
  client_options: DisplayOption[];
  billable_options: DisplayOption[];
  active_timer: ActiveTimer;
  invoice_action: InvoiceAction;
  create_action: BackendAction | null;
  empty_message: string;
  today: string;
}

const emptyTimeEntriesOverview: TimeEntriesOverview = {
  stats: {
    today_minutes_label: "0m",
    total_records_label: "0",
    billable_minutes_label: "0m",
    billable_amount_label: "¥0",
    unbilled_count: 0,
  },
  entries: [],
  project_options: [],
  client_options: [],
  billable_options: [],
  active_timer: {
    is_running: false,
    elapsed_label: "0m",
    start_action: null,
    stop_action: null,
  },
  invoice_action: {
    can_generate: false,
    label: "生成发票 (0)",
    confirm_message: "确定将 0 条未开票工时转为发票？",
    action: null,
  },
  create_action: null,
  empty_message: "暂无工时记录，点击上方开始计时或手动记录",
  today: "",
};

function TimeTracking() {
  const queryClient = useQueryClient();
  const [dialogOpen, setDialogOpen] = useState(false);
  const [timerDesc, setTimerDesc] = useState("");
  const [timerProject, setTimerProject] = useState("");
  const [timerClient, setTimerClient] = useState("");

  const {
    data: timeOverview = emptyTimeEntriesOverview,
    isLoading,
    isError,
    refetch,
  } = useQuery(
    {
      queryKey: ["time_entries_overview"],
      queryFn: () =>
        api.get<TimeEntriesOverview>(TIME_ENTRIES_OVERVIEW_ENDPOINT),
      refetchInterval: 1000,
    },
  );

  const refreshOverview = () => {
    queryClient.invalidateQueries({ queryKey: ["time_entries_overview"] });
  };

  const createEntry = useMutation({
    mutationFn: ({
      action,
      values,
    }: {
      action: BackendAction;
      values: Record<string, unknown>;
    }) => api.runAction(action, values),
    onSuccess: (_data, variables) => {
      refreshOverview();
      toast.success(variables.action.success_message || "工时已记录");
      setDialogOpen(false);
    },
  });

  const startTimerMutation = useMutation({
    mutationFn: ({
      action,
      values,
    }: {
      action: BackendAction;
      values: Record<string, unknown>;
    }) => api.runAction(action, values),
    onSuccess: (_data, variables) => {
      refreshOverview();
      toast.success(variables.action.success_message || "计时已开始");
    },
  });

  const stopTimerMutation = useMutation({
    mutationFn: (action: BackendAction) => api.runAction(action),
    onSuccess: (_data, action) => {
      refreshOverview();
      setTimerDesc("");
      setTimerProject("");
      setTimerClient("");
      toast.success(action.success_message || "工时已记录");
    },
  });

  const deleteEntry = useMutation({
    mutationFn: async (action: DeleteAction) => {
      await api.runAction(action);
      return action;
    },
    onSuccess: (action) => {
      refreshOverview();
      toast.success(action.success_message);
    },
  });

  const generateInvoice = useMutation({
    mutationFn: (action: BackendAction) =>
      api.runAction<{ toast_message?: string }>(action),
    onSuccess: (data, action) => {
      refreshOverview();
      queryClient.invalidateQueries({ queryKey: ["invoices"] });
      toast.success(data?.toast_message || action.success_message || "工时发票已生成");
    },
    onError: () => {
      toast.error("没有可计费的未开票工时记录");
    },
  });

  const startTimer = () => {
    if (!timerDesc) {
      toast.error("请输入工作描述");
      return;
    }
    const action = activeTimer.start_action;
    if (!action) {
      toast.error("当前无法开始计时");
      return;
    }
    const values: Record<string, unknown> = {
      description: timerDesc,
    };
    if (timerProject) values.project_id = timerProject;
    if (timerClient) values.client_id = timerClient;
    startTimerMutation.mutate({ action, values });
  };

  const stopTimer = () => {
    if (!activeTimer.stop_action) {
      toast.error("当前没有运行中的计时");
      return;
    }
    stopTimerMutation.mutate(activeTimer.stop_action);
  };

  const handleManualSubmit = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const formData = new FormData(e.currentTarget);
    const values: Record<string, unknown> = {};
    formData.forEach((value, key) => {
      if (value) values[key] = value;
    });
    const action = timeOverview.create_action;
    if (!action) {
      toast.error("当前无法手动记录工时");
      return;
    }
    createEntry.mutate({ action, values });
  };

  const handleDeleteEntry = (action: DeleteAction | null) => {
    if (!action || !confirm(action.confirm_message)) return;
    deleteEntry.mutate(action);
  };

  const entries = timeOverview.entries;
  const projects = timeOverview.project_options;
  const clients = timeOverview.client_options;
  const billableOptions = timeOverview.billable_options;
  const activeTimer = timeOverview.active_timer;
  const stats = timeOverview.stats;
  const invoiceAction = timeOverview.invoice_action;

  return (
    <QueryStateGate isError={isError} refetch={refetch}>
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">时间追踪</h1>
          <p className="text-muted-foreground">
            记录工时、追踪项目时间、生成计费报告
          </p>
        </div>
        <div className="flex gap-2">
          {invoiceAction.can_generate && (
            <Button
              variant="default"
              onClick={() => {
                const action = invoiceAction.action;
                if (action && confirm(invoiceAction.confirm_message))
                  generateInvoice.mutate(action);
              }}
              disabled={generateInvoice.isPending}
            >
              <FileText className="h-4 w-4 mr-2" />
              {invoiceAction.label}
            </Button>
          )}
          <Dialog open={dialogOpen} onOpenChange={setDialogOpen}>
            <DialogTrigger asChild>
              <Button variant="outline" disabled={!timeOverview.create_action}>
                <Plus className="h-4 w-4 mr-2" /> 手动记录
              </Button>
            </DialogTrigger>
            <DialogContent>
              <DialogHeader>
                <DialogTitle>手动记录工时</DialogTitle>
              </DialogHeader>
              <form onSubmit={handleManualSubmit} className="space-y-4">
                <div>
                  <Label>描述 *</Label>
                  <Input name="description" required />
                </div>
                <div>
                  <Label>日期 *</Label>
                  <Input
                    name="date"
                    type="date"
                    required
                    defaultValue={timeOverview.today}
                  />
                </div>
                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <Label>时长 (分钟)</Label>
                    <Input
                      name="duration_minutes"
                      type="number"
                      defaultValue="60"
                    />
                  </div>
                  <div>
                    <Label>时薪</Label>
                    <Input name="hourly_rate" type="number" step="0.01" />
                  </div>
                </div>
                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <Label>项目</Label>
                    <Select name="project_id">
                      <SelectTrigger>
                        <SelectValue placeholder="选择项目" />
                      </SelectTrigger>
                      <SelectContent>
                        {projects.map((project) => (
                          <SelectItem key={project.value} value={project.value}>
                            {project.label}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </div>
                  <div>
                    <Label>客户</Label>
                    <Select name="client_id">
                      <SelectTrigger>
                        <SelectValue placeholder="选择客户" />
                      </SelectTrigger>
                      <SelectContent>
                        {clients.map((client) => (
                          <SelectItem key={client.value} value={client.value}>
                            {client.label}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </div>
                </div>
                <div>
                  <Label>计费</Label>
                  <Select name="is_billable" defaultValue={billableOptions[0]?.value}>
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      {billableOptions.map((option) => (
                        <SelectItem key={option.value} value={option.value}>
                          {option.label}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
                <Button type="submit" className="w-full">
                  保存
                </Button>
              </form>
            </DialogContent>
          </Dialog>
        </div>
      </div>

      <Card className="border-2 border-primary/20">
        <CardContent className="pt-6">
          <div className="flex items-center gap-4">
            <Input
              placeholder="正在做什么？"
              value={activeTimer.is_running ? activeTimer.description || "" : timerDesc}
              onChange={(e) => setTimerDesc(e.target.value)}
              className="flex-1"
              disabled={activeTimer.is_running}
            />
            <Select
              value={
                activeTimer.is_running
                  ? activeTimer.project_id || ""
                  : timerProject
              }
              onValueChange={setTimerProject}
              disabled={activeTimer.is_running}
            >
              <SelectTrigger className="w-40">
                <SelectValue placeholder="项目" />
              </SelectTrigger>
              <SelectContent>
                {projects.map((project) => (
                  <SelectItem key={project.value} value={project.value}>
                    {project.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <Select
              value={
                activeTimer.is_running
                  ? activeTimer.client_id || ""
                  : timerClient
              }
              onValueChange={setTimerClient}
              disabled={activeTimer.is_running}
            >
              <SelectTrigger className="w-40">
                <SelectValue placeholder="客户" />
              </SelectTrigger>
              <SelectContent>
                {clients.map((client) => (
                  <SelectItem key={client.value} value={client.value}>
                    {client.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <div className="font-mono text-2xl font-bold w-28 text-center">
              {activeTimer.elapsed_label}
            </div>
            {activeTimer.is_running ? (
              <Button
                variant="destructive"
                size="lg"
                onClick={stopTimer}
                disabled={stopTimerMutation.isPending}
              >
                <Square className="h-5 w-5 mr-2" /> 停止
              </Button>
            ) : (
              <Button
                size="lg"
                onClick={startTimer}
                disabled={startTimerMutation.isPending}
              >
                <Play className="h-5 w-5 mr-2" /> 开始
              </Button>
            )}
          </div>
        </CardContent>
      </Card>

      <div className="grid grid-cols-4 gap-4">
        <Card>
          <CardContent className="pt-6">
            <div className="flex items-center gap-2">
              <Clock className="h-5 w-5 text-blue-500" />
              <div>
                <div className="text-2xl font-bold">
                  {stats.today_minutes_label}
                </div>
                <p className="text-xs text-muted-foreground">今日工时</p>
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
                  {stats.total_records_label}
                </div>
                <p className="text-xs text-muted-foreground">总记录数</p>
              </div>
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="pt-6">
            <div className="flex items-center gap-2">
              <TrendingUp className="h-5 w-5 text-green-500" />
              <div>
                <div className="text-2xl font-bold">
                  {stats.billable_minutes_label}
                </div>
                <p className="text-xs text-muted-foreground">可计费时间</p>
              </div>
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="pt-6">
            <div className="flex items-center gap-2">
              <DollarSign className="h-5 w-5 text-orange-500" />
              <div>
                <div className="text-2xl font-bold">
                  {stats.billable_amount_label}
                </div>
                <p className="text-xs text-muted-foreground">计费金额</p>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>工时记录</CardTitle>
        </CardHeader>
        <CardContent className="p-0">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>描述</TableHead>
                <TableHead>项目</TableHead>
                <TableHead>客户</TableHead>
                <TableHead>日期</TableHead>
                <TableHead>时长</TableHead>
                <TableHead>计费</TableHead>
                <TableHead>金额</TableHead>
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
              ) : entries.length === 0 ? (
                <TableRow>
                  <TableCell colSpan={8} className="py-12">
                    <div className="flex flex-col items-center justify-center text-center">
                      <Clock className="h-16 w-16 text-muted-foreground mb-4" />
                      <h3 className="text-lg font-semibold mb-2">
                        {timeOverview.empty_message}
                      </h3>
                      <p className="text-muted-foreground mb-6 max-w-md">
                        记录可计费工时，按时薪汇总后可一键生成发票。
                      </p>
                      {timeOverview.create_action && (
                        <Button
                          size="lg"
                          variant="outline"
                          onClick={() => setDialogOpen(true)}
                        >
                          <Plus className="mr-2 h-4 w-4" /> 手动记录工时
                        </Button>
                      )}
                    </div>
                  </TableCell>
                </TableRow>
              ) : (
                entries.map((entry) => (
                  <TableRow key={entry.id}>
                    <TableCell className="font-medium max-w-[200px] truncate">
                      {entry.description}
                    </TableCell>
                    <TableCell>{entry.project_name}</TableCell>
                    <TableCell>{entry.client_name}</TableCell>
                    <TableCell>{entry.date_label}</TableCell>
                    <TableCell>{entry.duration_label}</TableCell>
                    <TableCell>
                      <Badge variant={entry.billable_badge.variant}>
                        {entry.billable_badge.label}
                      </Badge>
                    </TableCell>
                    <TableCell>{entry.amount_label}</TableCell>
                    <TableCell className="text-right">
                      {entry.delete_action && (
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => handleDeleteEntry(entry.delete_action)}
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

export default TimeTracking;
