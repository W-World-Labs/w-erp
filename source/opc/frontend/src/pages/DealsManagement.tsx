import { useState } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api, type BackendAction } from "@/integrations/api/client";
import { API_ENDPOINTS } from "@/lib/api";
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
  Plus,
  Target,
  DollarSign,
  TrendingUp,
  AlertCircle,
} from "lucide-react";

interface DealsOverview {
  stats: {
    active_count: number;
    total_pipeline: number;
    total_pipeline_label: string;
    won_amount: number;
    won_amount_label: string;
    avg_probability: number;
    avg_probability_label: string;
  };
  stage_options: any[];
  pipeline_stages: any[];
  won_deals: any[];
  lost_deals: any[];
  clients: any[];
  create_action: BackendAction | null;
}

const emptyDealsOverview: DealsOverview = {
  stats: {
    active_count: 0,
    total_pipeline: 0,
    total_pipeline_label: "¥0",
    won_amount: 0,
    won_amount_label: "¥0",
    avg_probability: 0,
    avg_probability_label: "0%",
  },
  stage_options: [],
  pipeline_stages: [],
  won_deals: [],
  lost_deals: [],
  clients: [],
  create_action: null,
};

function DealsManagement() {
  const queryClient = useQueryClient();
  const [dialogOpen, setDialogOpen] = useState(false);

  const { data: overview = emptyDealsOverview, isLoading } = useQuery({
    queryKey: ["deals-overview"],
    queryFn: () => api.get<DealsOverview>(API_ENDPOINTS.dealsOverview),
  });

  const createDeal = useMutation({
    mutationFn: ({
      action,
      values,
    }: {
      action: BackendAction;
      values: Record<string, any>;
    }) => api.runAction(action, values),
    onSuccess: (_data, variables) => {
      queryClient.invalidateQueries({ queryKey: ["deals-overview"] });
      toast.success(variables.action.success_message || "销售机会已创建");
      setDialogOpen(false);
    },
  });

  const updateDeal = useMutation({
    mutationFn: (action: BackendAction) =>
      api.runAction(action, action.default_values),
    onSuccess: (_data, action) => {
      queryClient.invalidateQueries({ queryKey: ["deals-overview"] });
      toast.success(action.success_message || "阶段已更新");
    },
  });

  const handleSubmit = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const formData = new FormData(e.currentTarget);
    const values: Record<string, any> = {};
    formData.forEach((v, k) => {
      if (v) values[k] = v;
    });
    const action = overview.create_action;
    if (!action) {
      toast.error("当前账号没有创建销售机会的权限");
      return;
    }
    createDeal.mutate({ action, values });
  };

  const stats = overview.stats;
  const clients = overview.clients;
  const pipelineStages = overview.pipeline_stages;
  const wonDeals = overview.won_deals;
  const lostDeals = overview.lost_deals;

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">销售机会</h1>
          <p className="text-muted-foreground">跟踪销售线索到成交的完整流程</p>
        </div>
        <Dialog open={dialogOpen} onOpenChange={setDialogOpen}>
          <DialogTrigger asChild>
            <Button>
              <Plus className="h-4 w-4 mr-2" /> 新增机会
            </Button>
          </DialogTrigger>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>新增销售机会</DialogTitle>
            </DialogHeader>
            <form onSubmit={handleSubmit} className="space-y-4">
              <div>
                <Label>标题 *</Label>
                <Input name="title" required />
              </div>
              <div>
                <Label>关联客户</Label>
                <Select name="client_id">
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
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <Label>预计金额</Label>
                  <Input name="amount" type="number" />
                </div>
                <div>
                  <Label>成交概率 (%)</Label>
                  <Input
                    name="probability"
                    type="number"
                    min="0"
                    max="100"
                    defaultValue="10"
                  />
                </div>
              </div>
              <div>
                <Label>预计成交日期</Label>
                <Input name="expected_close_date" type="date" />
              </div>
              <div>
                <Label>描述</Label>
                <Textarea name="description" />
              </div>
              <Button type="submit" className="w-full">
                创建
              </Button>
            </form>
          </DialogContent>
        </Dialog>
      </div>

      <div className="grid grid-cols-4 gap-4">
        <Card>
          <CardContent className="pt-6">
            <div className="flex items-center gap-2">
              <Target className="h-5 w-5 text-blue-500" />
              <div>
                <div className="text-2xl font-bold">{stats.active_count}</div>
                <p className="text-xs text-muted-foreground">进行中</p>
              </div>
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="pt-6">
            <div className="flex items-center gap-2">
              <DollarSign className="h-5 w-5 text-purple-500" />
              <div>
                <div className="text-2xl font-bold">
                  {stats.total_pipeline_label}
                </div>
                <p className="text-xs text-muted-foreground">管道总额</p>
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
                  {stats.won_amount_label}
                </div>
                <p className="text-xs text-muted-foreground">已赢单</p>
              </div>
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="pt-6">
            <div className="flex items-center gap-2">
              <AlertCircle className="h-5 w-5 text-orange-500" />
              <div>
                <div className="text-2xl font-bold">
                  {stats.avg_probability_label}
                </div>
                <p className="text-xs text-muted-foreground">平均概率</p>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>

      {isLoading ? (
        <div className="text-center py-8">加载中...</div>
      ) : (
        <div className="grid grid-cols-4 gap-4">
          {pipelineStages.map((stage: any) => (
            <Card key={stage.value}>
              <CardHeader className="pb-2">
                <div className="flex items-center justify-between">
                  <Badge className={stage.color}>{stage.label}</Badge>
                  <span className="text-xs text-muted-foreground">
                    {stage.count} 个 · {stage.amount_label}
                  </span>
                </div>
              </CardHeader>
              <CardContent className="space-y-2">
                {stage.count === 0 && (
                  <p className="text-xs text-muted-foreground text-center py-4">
                    暂无
                  </p>
                )}
                {stage.deals.map((deal: any) => (
                  <div
                    key={deal.id}
                    className="border rounded-lg p-3 space-y-1 hover:shadow-sm transition-shadow"
                  >
                    <p className="text-sm font-medium">{deal.title}</p>
                    <p className="text-xs text-muted-foreground">
                      {deal.client_name}
                    </p>
                    <div className="flex justify-between text-xs">
                      <span>{deal.amount_label}</span>
                      <span>{deal.probability_label}</span>
                    </div>
                    {deal.can_update_stage && (
                      <div className="flex gap-1 pt-1">
                        {deal.next_stages.map((nextStage: any) => (
                          <Button
                            key={nextStage.value}
                            variant="ghost"
                            size="sm"
                            className="h-6 text-xs px-2"
                            disabled={!nextStage.action}
                            onClick={() => {
                              if (nextStage.action) {
                                updateDeal.mutate(nextStage.action);
                              }
                            }}
                          >
                            → {nextStage.label}
                          </Button>
                        ))}
                      </div>
                    )}
                  </div>
                ))}
              </CardContent>
            </Card>
          ))}
        </div>
      )}

      {(wonDeals.length > 0 || lostDeals.length > 0) && (
        <div className="grid grid-cols-2 gap-4">
          <Card>
            <CardHeader>
              <CardTitle className="text-sm text-green-600">
                赢单 ({wonDeals.length})
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-1">
              {wonDeals.map((deal: any) => (
                <div key={deal.id} className="flex justify-between text-sm">
                  <span>{deal.title}</span>
                  <span className="font-medium">{deal.amount_label}</span>
                </div>
              ))}
            </CardContent>
          </Card>
          <Card>
            <CardHeader>
              <CardTitle className="text-sm text-red-600">
                丢单 ({lostDeals.length})
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-1">
              {lostDeals.map((deal: any) => (
                <div key={deal.id} className="flex justify-between text-sm">
                  <span>{deal.title}</span>
                  <span className="text-muted-foreground">
                    {deal.lost_reason || "-"}
                  </span>
                </div>
              ))}
            </CardContent>
          </Card>
        </div>
      )}
    </div>
  );
}

export default DealsManagement;
