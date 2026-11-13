import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
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
import {
  Building2,
  Globe,
  Mail,
  MessageSquare,
  Pencil,
  Phone,
  Plus,
  Search,
  Trash2,
  Users,
  type LucideIcon,
} from "lucide-react";
import { QueryStateGate } from "@/components/common/QueryStateGate";
import { apiErrorMessage } from "@/lib/apiError";

const CLIENTS_OVERVIEW_ENDPOINT = "/api/v1/clients/overview";

interface ClientBadge {
  value: string;
  label: string;
  variant?: "default" | "secondary" | "destructive" | "outline";
  icon?: string;
}

interface DeleteAction {
  method: "DELETE";
  endpoint: string;
  confirm_message: string;
  success_title: string;
  success_message: string;
}

interface ClientOverviewItem {
  id: string;
  name?: string;
  short_name?: string;
  client_type?: string;
  type_badge: ClientBadge;
  industry?: string;
  website?: string;
  email?: string;
  phone?: string;
  address?: string;
  city?: string;
  source?: string;
  notes?: string;
  annual_revenue?: number;
  annual_revenue_label: string;
  credit_profile?: {
    score: number;
    score_label: string;
    grade: string;
    label: string;
    average_payment_delay_label: string;
    collection_event_count: number;
    basis: string;
  };
  update_action?: BackendAction | null;
  can_delete?: boolean;
  delete_action: DeleteAction | null;
  contact_create_action?: BackendAction | null;
  interaction_create_action?: BackendAction | null;
}

interface ContactOverviewItem {
  id: string;
  client_id?: string;
  name?: string;
  title?: string;
  email?: string;
}

interface InteractionOverviewItem {
  id: string;
  interaction_type?: string;
  interaction_badge: ClientBadge;
  subject?: string;
  description?: string;
  interaction_date?: string;
}

interface ClientsOverview {
  stats: {
    total: number;
    active: number;
    prospect: number;
    total_revenue: number;
    total_revenue_label: string;
  };
  clients: ClientOverviewItem[];
  contacts: ContactOverviewItem[];
  interactions: InteractionOverviewItem[];
  detail_client: ClientOverviewItem | null;
  detail_contacts: ContactOverviewItem[];
  detail_interactions: InteractionOverviewItem[];
  type_options: ClientBadge[];
  source_options: ClientBadge[];
  interaction_type_options: ClientBadge[];
  create_action: BackendAction | null;
  empty_message: string;
}

const emptyClientsOverview: ClientsOverview = {
  stats: {
    total: 0,
    active: 0,
    prospect: 0,
    total_revenue: 0,
    total_revenue_label: "¥0.0万",
  },
  clients: [],
  contacts: [],
  interactions: [],
  detail_client: null,
  detail_contacts: [],
  detail_interactions: [],
  type_options: [],
  source_options: [],
  interaction_type_options: [],
  create_action: null,
  empty_message: "暂无客户数据",
};

const interactionIcons: Record<string, LucideIcon> = {
  Phone,
  Mail,
  Users,
  MessageSquare,
};

function ClientTypeBadge({ badge }: { badge: ClientBadge }) {
  return <Badge variant={badge.variant || "outline"}>{badge.label}</Badge>;
}

function InteractionIcon({ badge }: { badge: ClientBadge }) {
  const Icon = badge.icon ? interactionIcons[badge.icon] : null;
  const SafeIcon = Icon || MessageSquare;

  return <SafeIcon className="h-4 w-4 text-muted-foreground" />;
}

function ClientManagement() {
  const queryClient = useQueryClient();
  const navigate = useNavigate();
  const [search, setSearch] = useState("");
  const [dialogOpen, setDialogOpen] = useState(false);
  const [editingClient, setEditingClient] = useState<any>(null);
  const [detailClient, setDetailClient] = useState<ClientOverviewItem | null>(
    null,
  );
  const [interactionDialogOpen, setInteractionDialogOpen] = useState(false);
  const [contactDialogOpen, setContactDialogOpen] = useState(false);

  const {
    data: overview = emptyClientsOverview,
    isLoading,
    isError,
    refetch,
  } = useQuery({
    queryKey: ["clients-overview", search, detailClient?.id],
    queryFn: () => {
      const params = new URLSearchParams();
      const searchValue = search.trim();
      if (searchValue) {
        params.set("search", searchValue);
      }
      if (detailClient?.id) {
        params.set("selected_client_id", detailClient.id);
      }
      const query = params.toString();
      return api.get<ClientsOverview>(
        `${CLIENTS_OVERVIEW_ENDPOINT}${query ? `?${query}` : ""}`,
      );
    },
  });

  const clients = overview.clients;
  const clientStats = overview.stats;
  const detailContacts = overview.detail_contacts;
  const detailInteractions = overview.detail_interactions;
  const currentDetailClient = overview.detail_client || detailClient;

  const invalidateClientsOverview = () => {
    queryClient.invalidateQueries({ queryKey: ["clients-overview"] });
  };

  const saveClient = useMutation({
    mutationFn: ({
      action,
      values,
    }: {
      action: BackendAction;
      values: Record<string, any>;
    }) => api.runAction(action, values),
    onSuccess: (_data, variables) => {
      invalidateClientsOverview();
      toast.success(variables.action.success_message || "客户信息已保存");
      setDialogOpen(false);
      setEditingClient(null);
    },
    onError: (e) => {
      toast.error(apiErrorMessage(e));
    },
  });

  const deleteClient = useMutation({
    mutationFn: async (action: DeleteAction) => {
      await api.runAction(action);
      return action;
    },
    onSuccess: (action) => {
      invalidateClientsOverview();
      toast.success(action.success_message);
    },
    onError: (e) => {
      toast.error(apiErrorMessage(e));
    },
  });

  const createInteraction = useMutation({
    mutationFn: ({
      action,
      values,
    }: {
      action: BackendAction;
      values: Record<string, any>;
    }) => api.runAction(action, values),
    onSuccess: (_data, variables) => {
      invalidateClientsOverview();
      toast.success(variables.action.success_message || "互动记录已添加");
      setInteractionDialogOpen(false);
    },
    onError: (e) => {
      toast.error(apiErrorMessage(e));
    },
  });

  const createContact = useMutation({
    mutationFn: ({
      action,
      values,
    }: {
      action: BackendAction;
      values: Record<string, any>;
    }) => api.runAction(action, values),
    onSuccess: (_data, variables) => {
      invalidateClientsOverview();
      toast.success(variables.action.success_message || "联系人已添加");
      setContactDialogOpen(false);
    },
    onError: (e) => {
      toast.error(apiErrorMessage(e));
    },
  });

  const handleSubmit = (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const formData = new FormData(event.currentTarget);
    const values: Record<string, any> = {};
    formData.forEach((value, key) => {
      if (value) values[key] = value;
    });

    const action = editingClient
      ? editingClient.update_action
      : overview.create_action;
    if (!action) {
      toast.error("当前账号没有保存客户资料的权限");
      return;
    }
    saveClient.mutate({ action, values });
  };

  const handleInteractionSubmit = (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!currentDetailClient) return;
    const action = currentDetailClient.interaction_create_action;
    if (!action) {
      toast.error("当前无法添加互动记录");
      return;
    }
    const formData = new FormData(event.currentTarget);
    const defaultValues =
      (action as BackendAction & { default_values?: Record<string, any> })
        .default_values || {};
    const values: Record<string, any> = { ...defaultValues };
    formData.forEach((value, key) => {
      if (value) values[key] = value;
    });
    createInteraction.mutate({ action, values });
  };

  const handleContactSubmit = (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!currentDetailClient) return;
    const action = currentDetailClient.contact_create_action;
    if (!action) {
      toast.error("当前无法添加联系人");
      return;
    }
    const formData = new FormData(event.currentTarget);
    const defaultValues =
      (action as BackendAction & { default_values?: Record<string, any> })
        .default_values || {};
    const values: Record<string, any> = { ...defaultValues };
    formData.forEach((value, key) => {
      if (value) values[key] = value;
    });
    createContact.mutate({ action, values });
  };

  const handleDeleteClient = (action: DeleteAction | null) => {
    if (!action || !confirm(action.confirm_message)) return;
    deleteClient.mutate(action);
  };

  if (currentDetailClient) {
    return (
      <div className="space-y-6">
        <div className="flex items-center justify-between">
          <div>
            <Button
              variant="ghost"
              onClick={() => setDetailClient(null)}
              className="mb-2"
            >
              ← 返回客户列表
            </Button>
            <h1 className="text-2xl font-bold">{currentDetailClient.name}</h1>
            <p className="text-muted-foreground">
              {currentDetailClient.industry} · {currentDetailClient.email}
            </p>
          </div>
          <div className="flex items-center gap-3">
            <Button
              onClick={() =>
                navigate(`/quotes?client_id=${currentDetailClient.id}`)
              }
            >
              <Plus className="mr-2 h-4 w-4" /> 为此客户创建报价
            </Button>
            <ClientTypeBadge badge={currentDetailClient.type_badge} />
          </div>
        </div>

        <div className="grid grid-cols-1 gap-6 lg:grid-cols-3">
          <Card>
            <CardHeader>
              <CardTitle className="text-sm">客户信息</CardTitle>
            </CardHeader>
            <CardContent className="space-y-3 text-sm">
              {currentDetailClient.phone && (
                <div className="flex items-center gap-2">
                  <Phone className="h-4 w-4" /> {currentDetailClient.phone}
                </div>
              )}
              {currentDetailClient.email && (
                <div className="flex items-center gap-2">
                  <Mail className="h-4 w-4" /> {currentDetailClient.email}
                </div>
              )}
              {currentDetailClient.website && (
                <div className="flex items-center gap-2">
                  <Globe className="h-4 w-4" /> {currentDetailClient.website}
                </div>
              )}
              {currentDetailClient.address && (
                <div className="flex items-center gap-2">
                  <Building2 className="h-4 w-4" />{" "}
                  {currentDetailClient.address}
                  {currentDetailClient.city
                    ? `, ${currentDetailClient.city}`
                    : ""}
                </div>
              )}
              {currentDetailClient.source && (
                <div>
                  <span className="text-muted-foreground">来源：</span>
                  {currentDetailClient.source}
                </div>
              )}
              {currentDetailClient.annual_revenue && (
                <div>
                  <span className="text-muted-foreground">年营收：</span>
                  {currentDetailClient.annual_revenue_label}
                </div>
              )}
              {currentDetailClient.credit_profile && (
                <div className="border-t pt-2">
                  <div className="flex items-center justify-between">
                    <span className="text-muted-foreground">回款信用评分：</span>
                    <Badge variant="outline">
                      {currentDetailClient.credit_profile.grade} · {currentDetailClient.credit_profile.score_label}
                    </Badge>
                  </div>
                  <p className="mt-1 text-xs text-muted-foreground">
                    {currentDetailClient.credit_profile.average_payment_delay_label} · 催收记录 {currentDetailClient.credit_profile.collection_event_count} 条
                  </p>
                </div>
              )}
              {currentDetailClient.notes && (
                <div className="border-t pt-2">
                  <p className="text-muted-foreground">
                    {currentDetailClient.notes}
                  </p>
                </div>
              )}
            </CardContent>
          </Card>

          <Card>
            <CardHeader className="flex flex-row items-center justify-between">
              <CardTitle className="text-sm">联系人</CardTitle>
              <Button
                size="sm"
                variant="outline"
                onClick={() => setContactDialogOpen(true)}
                disabled={!currentDetailClient.contact_create_action}
              >
                <Plus className="mr-1 h-4 w-4" /> 添加
              </Button>
            </CardHeader>
            <CardContent className="space-y-2">
              {detailContacts.length === 0 && (
                <p className="text-sm text-muted-foreground">暂无联系人</p>
              )}
              {detailContacts.map((contact) => (
                <div
                  key={contact.id}
                  className="flex items-center gap-3 rounded bg-muted/50 p-2"
                >
                  <Users className="h-4 w-4 text-muted-foreground" />
                  <div>
                    <p className="text-sm font-medium">
                      {contact.name}{" "}
                      {contact.title && (
                        <span className="text-muted-foreground">
                          · {contact.title}
                        </span>
                      )}
                    </p>
                    {contact.email && (
                      <p className="text-xs text-muted-foreground">
                        {contact.email}
                      </p>
                    )}
                  </div>
                </div>
              ))}
            </CardContent>
            <Dialog open={contactDialogOpen} onOpenChange={setContactDialogOpen}>
              <DialogContent>
                <DialogHeader>
                  <DialogTitle>
                    {currentDetailClient.contact_create_action?.label ||
                      "添加联系人"}
                  </DialogTitle>
                </DialogHeader>
                <form onSubmit={handleContactSubmit} className="space-y-4">
                  <div>
                    <Label>姓名 *</Label>
                    <Input name="name" required />
                  </div>
                  <div>
                    <Label>职务</Label>
                    <Input name="title" />
                  </div>
                  <div>
                    <Label>邮箱</Label>
                    <Input name="email" type="email" />
                  </div>
                  <div>
                    <Label>电话</Label>
                    <Input name="phone" />
                  </div>
                  <Button type="submit">保存</Button>
                </form>
              </DialogContent>
            </Dialog>
          </Card>

          <Card>
            <CardHeader className="flex flex-row items-center justify-between">
              <CardTitle className="text-sm">互动时间线</CardTitle>
              <Dialog
                open={interactionDialogOpen}
                onOpenChange={setInteractionDialogOpen}
              >
                <DialogTrigger asChild>
                  <Button
                    size="sm"
                    variant="outline"
                    disabled={!currentDetailClient.interaction_create_action}
                  >
                    <Plus className="mr-1 h-4 w-4" /> 记录
                  </Button>
                </DialogTrigger>
                <DialogContent>
                  <DialogHeader>
                    <DialogTitle>添加互动记录</DialogTitle>
                  </DialogHeader>
                  <form
                    onSubmit={handleInteractionSubmit}
                    className="space-y-4"
                  >
                    <div>
                      <Label>类型</Label>
                      <Select name="interaction_type" defaultValue="note">
                        <SelectTrigger>
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          {overview.interaction_type_options.map((option) => (
                            <SelectItem key={option.value} value={option.value}>
                              {option.label}
                            </SelectItem>
                          ))}
                        </SelectContent>
                      </Select>
                    </div>
                    <div>
                      <Label>主题</Label>
                      <Input name="subject" />
                    </div>
                    <div>
                      <Label>内容</Label>
                      <Textarea name="description" />
                    </div>
                    <div>
                      <Label>日期</Label>
                      <Input
                        name="interaction_date"
                        type="date"
                      />
                    </div>
                    <Button type="submit">保存</Button>
                  </form>
                </DialogContent>
              </Dialog>
            </CardHeader>
            <CardContent>
              {detailInteractions.length === 0 && (
                <p className="text-sm text-muted-foreground">暂无互动记录</p>
              )}
              <div className="space-y-3">
                {detailInteractions.map((interaction) => (
                  <div key={interaction.id} className="flex gap-3 text-sm">
                    <div className="mt-0.5 flex-shrink-0">
                      <InteractionIcon badge={interaction.interaction_badge} />
                    </div>
                    <div>
                      <p className="font-medium">
                        {interaction.subject ||
                          interaction.interaction_badge.label}
                      </p>
                      {interaction.description && (
                        <p className="mt-0.5 text-xs text-muted-foreground">
                          {interaction.description}
                        </p>
                      )}
                      <p className="mt-1 text-xs text-muted-foreground">
                        {interaction.interaction_date}
                      </p>
                    </div>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>
        </div>
      </div>
    );
  }

  return (
    <QueryStateGate isError={isError} refetch={refetch}>
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">客户管理 (CRM)</h1>
          <p className="text-muted-foreground">
            管理客户资料、联系人和互动记录
          </p>
        </div>
        <Dialog
          open={dialogOpen}
          onOpenChange={(open) => {
            setDialogOpen(open);
            if (!open) setEditingClient(null);
          }}
        >
          <DialogTrigger asChild>
            <Button disabled={!overview.create_action}>
              <Plus className="mr-2 h-4 w-4" /> 新增客户
            </Button>
          </DialogTrigger>
          <DialogContent className="max-w-lg">
            <DialogHeader>
              <DialogTitle>
                {editingClient ? "编辑客户" : "新增客户"}
              </DialogTitle>
            </DialogHeader>
            <form onSubmit={handleSubmit} className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <Label>客户名称 *</Label>
                  <Input
                    name="name"
                    required
                    defaultValue={editingClient?.name}
                  />
                </div>
                <div>
                  <Label>简称</Label>
                  <Input
                    name="short_name"
                    defaultValue={editingClient?.short_name}
                  />
                </div>
                <div>
                  <Label>客户类型</Label>
                  <Select
                    name="client_type"
                    defaultValue={editingClient?.client_type || "prospect"}
                  >
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      {overview.type_options.map((option) => (
                        <SelectItem key={option.value} value={option.value}>
                          {option.label}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
                <div>
                  <Label>行业</Label>
                  <Input
                    name="industry"
                    defaultValue={editingClient?.industry}
                  />
                </div>
                <div>
                  <Label>邮箱</Label>
                  <Input
                    name="email"
                    type="email"
                    defaultValue={editingClient?.email}
                  />
                </div>
                <div>
                  <Label>电话</Label>
                  <Input name="phone" defaultValue={editingClient?.phone} />
                </div>
                <div>
                  <Label>网站</Label>
                  <Input name="website" defaultValue={editingClient?.website} />
                </div>
                <div>
                  <Label>来源</Label>
                  <Select
                    name="source"
                    defaultValue={editingClient?.source || ""}
                  >
                    <SelectTrigger>
                      <SelectValue placeholder="选择来源" />
                    </SelectTrigger>
                    <SelectContent>
                      {overview.source_options.map((option) => (
                        <SelectItem key={option.value} value={option.value}>
                          {option.label}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
                <div className="col-span-2">
                  <Label>地址</Label>
                  <Input name="address" defaultValue={editingClient?.address} />
                </div>
                <div>
                  <Label>城市</Label>
                  <Input name="city" defaultValue={editingClient?.city} />
                </div>
                <div>
                  <Label>年营收</Label>
                  <Input
                    name="annual_revenue"
                    type="number"
                    defaultValue={editingClient?.annual_revenue}
                  />
                </div>
              </div>
              <div>
                <Label>备注</Label>
                <Textarea name="notes" defaultValue={editingClient?.notes} />
              </div>
              <Button type="submit" className="w-full">
                {editingClient ? "保存修改" : "创建客户"}
              </Button>
            </form>
          </DialogContent>
        </Dialog>
      </div>

      <div className="grid grid-cols-4 gap-4">
        <Card>
          <CardContent className="pt-6">
            <div className="text-2xl font-bold">{clientStats.total}</div>
            <p className="text-xs text-muted-foreground">总客户数</p>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="pt-6">
            <div className="text-2xl font-bold text-green-600">
              {clientStats.active}
            </div>
            <p className="text-xs text-muted-foreground">活跃客户</p>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="pt-6">
            <div className="text-2xl font-bold text-blue-600">
              {clientStats.prospect}
            </div>
            <p className="text-xs text-muted-foreground">潜在客户</p>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="pt-6">
            <div className="text-2xl font-bold">
              {clientStats.total_revenue_label}
            </div>
            <p className="text-xs text-muted-foreground">客户总营收</p>
          </CardContent>
        </Card>
      </div>

      <div className="flex gap-4">
        <div className="relative flex-1">
          <Search className="absolute left-3 top-3 h-4 w-4 text-muted-foreground" />
          <Input
            placeholder="搜索客户名称、邮箱..."
            className="pl-10"
            value={search}
            onChange={(event) => setSearch(event.target.value)}
          />
        </div>
      </div>

      <Card>
        <CardContent className="p-0">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>客户名称</TableHead>
                <TableHead>类型</TableHead>
                <TableHead>行业</TableHead>
                <TableHead>联系方式</TableHead>
                <TableHead>来源</TableHead>
                <TableHead>年营收</TableHead>
                <TableHead>回款信用</TableHead>
                <TableHead className="text-right">操作</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {isLoading ? (
                <TableRow>
                    <TableCell colSpan={8} className="py-8 text-center">
                    加载中...
                  </TableCell>
                </TableRow>
              ) : clients.length === 0 ? (
                <TableRow>
                  <TableCell colSpan={8} className="py-12">
                    <div className="flex flex-col items-center justify-center text-center">
                      <Users className="h-16 w-16 text-muted-foreground mb-4" />
                      <h3 className="text-lg font-semibold mb-2">
                        {overview.empty_message}
                      </h3>
                      {!search.trim() && (
                        <>
                          <p className="text-muted-foreground mb-6 max-w-md">
                            添加你的第一个客户，之后就能给他出报价、开发票、跟踪回款。
                          </p>
                          {overview.create_action && (
                            <Button
                              size="lg"
                              onClick={() => {
                                setEditingClient(null);
                                setDialogOpen(true);
                              }}
                            >
                              <Plus className="mr-2 h-4 w-4" /> 添加第一个客户
                            </Button>
                          )}
                        </>
                      )}
                    </div>
                  </TableCell>
                </TableRow>
              ) : (
                clients.map((client) => (
                  <TableRow
                    key={client.id}
                    className="cursor-pointer hover:bg-muted/50"
                    onClick={() => setDetailClient(client)}
                  >
                    <TableCell className="font-medium">
                      {client.name}
                      {client.short_name && (
                        <span className="ml-1 text-muted-foreground">
                          ({client.short_name})
                        </span>
                      )}
                    </TableCell>
                    <TableCell>
                      <ClientTypeBadge badge={client.type_badge} />
                    </TableCell>
                    <TableCell>{client.industry || "-"}</TableCell>
                    <TableCell>
                      <div className="text-sm">{client.email || "-"}</div>
                      {client.phone && (
                        <div className="text-xs text-muted-foreground">
                          {client.phone}
                        </div>
                      )}
                    </TableCell>
                    <TableCell>{client.source || "-"}</TableCell>
                    <TableCell>{client.annual_revenue_label}</TableCell>
                    <TableCell>
                      {client.credit_profile ? (
                        <Badge variant="outline">
                          {client.credit_profile.grade} · {client.credit_profile.score_label}
                        </Badge>
                      ) : (
                        "-"
                      )}
                    </TableCell>
                    <TableCell className="text-right">
                      {client.update_action && (
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={(event) => {
                            event.stopPropagation();
                            setEditingClient(client);
                            setDialogOpen(true);
                          }}
                        >
                          <Pencil className="h-4 w-4" />
                        </Button>
                      )}
                      {client.delete_action && (
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={(event) => {
                            event.stopPropagation();
                            handleDeleteClient(client.delete_action);
                          }}
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

export default ClientManagement;
