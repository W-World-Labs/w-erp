import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  companies,
  CompanyMemberRoleBadge,
  CompanyMembersOverview,
  getCurrentCompanyId,
} from "@/integrations/api/client";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "@/components/ui/alert-dialog";
import { toast } from "sonner";
import {
  Calculator,
  Crown,
  Eye,
  Landmark,
  Loader2,
  Search,
  Shield,
  Trash2,
  UserPlus,
  Users,
  type LucideIcon,
} from "lucide-react";

const emptyOverview: CompanyMembersOverview = {
  company: {
    id: "",
    name: "当前公司",
    current_user_role: "",
    is_admin: false,
  },
  stats: {
    total: 0,
    owners: 0,
    admins: 0,
  },
  members: [],
  assignable_roles: [],
  empty_message: "暂无成员数据",
};

const roleIcons: Record<string, LucideIcon> = {
  Crown,
  Shield,
  Users,
  Calculator,
  Eye,
  Landmark,
};

function RoleBadgeView({ badge }: { badge: CompanyMemberRoleBadge }) {
  const Icon = roleIcons[badge.icon] || Users;

  return (
    <Badge variant={badge.variant}>
      <Icon className="mr-1 h-3 w-3" />
      {badge.label}
    </Badge>
  );
}

function RoleOptionLabel({ option }: { option: CompanyMemberRoleBadge }) {
  const Icon = roleIcons[option.icon] || Users;

  return (
    <span className="flex items-center gap-2">
      <Icon className="h-3.5 w-3.5" />
      {option.label}
    </span>
  );
}

export default function UserManagement() {
  const queryClient = useQueryClient();
  const companyId = getCurrentCompanyId();
  const [searchTerm, setSearchTerm] = useState("");
  const [inviteOpen, setInviteOpen] = useState(false);
  const [inviteEmail, setInviteEmail] = useState("");
  const [inviteRole, setInviteRole] = useState("member");

  const { data: overview = emptyOverview, isLoading } = useQuery({
    queryKey: ["company-members-overview", companyId, searchTerm],
    queryFn: async () => {
      if (!companyId) return emptyOverview;
      return companies.members.overview(companyId, searchTerm);
    },
    enabled: !!companyId,
  });

  const isAdmin = overview.company.is_admin;
  const members = overview.members;
  const stats = overview.stats;
  const assignableRoles = overview.assignable_roles;

  const refreshMembersOverview = () => {
    queryClient.invalidateQueries({ queryKey: ["company-members-overview"] });
  };

  const inviteMutation = useMutation({
    mutationFn: async ({ email, role }: { email: string; role: string }) => {
      if (!companyId) throw new Error("No company selected");
      return companies.members.add(companyId, email, role);
    },
    onSuccess: () => {
      refreshMembersOverview();
      setInviteOpen(false);
      setInviteEmail("");
      setInviteRole("member");
      toast.success("成员邀请成功");
    },
    onError: (error: any) => {
      const msg = error?.message || "";
      if (msg.includes("not found")) {
        toast.error("该邮箱用户未注册，请先邀请其注册系统");
      } else if (msg.includes("UNIQUE")) {
        toast.error("该用户已是公司成员");
      } else {
        toast.error("邀请失败：" + msg);
      }
    },
  });

  const updateRoleMutation = useMutation({
    mutationFn: async ({
      memberId,
      role,
    }: {
      memberId: string;
      role: string;
    }) => {
      if (!companyId) throw new Error("No company selected");
      return companies.members.update(companyId, memberId, role);
    },
    onSuccess: () => {
      refreshMembersOverview();
      toast.success("角色已更新");
    },
    onError: () => toast.error("更新角色失败"),
  });

  const removeMutation = useMutation({
    mutationFn: async (memberId: string) => {
      if (!companyId) throw new Error("No company selected");
      return companies.members.remove(companyId, memberId);
    },
    onSuccess: () => {
      refreshMembersOverview();
      toast.success("成员已移除");
    },
    onError: () => toast.error("移除失败"),
  });

  const handleInvite = () => {
    const email = inviteEmail.trim();
    if (!email) return;
    inviteMutation.mutate({ email, role: inviteRole });
  };

  if (!companyId) {
    return (
      <div className="flex items-center justify-center py-20 text-muted-foreground">
        请先在左侧选择一个公司
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold text-foreground">成员管理</h1>
          <p className="mt-2 text-muted-foreground">
            管理{" "}
            <span className="font-medium text-foreground">
              {overview.company.name}
            </span>{" "}
            的团队成员与角色
          </p>
        </div>
        {isAdmin && (
          <Dialog open={inviteOpen} onOpenChange={setInviteOpen}>
            <DialogTrigger asChild>
              <Button>
                <UserPlus className="mr-2 h-4 w-4" />
                邀请成员
              </Button>
            </DialogTrigger>
            <DialogContent>
              <DialogHeader>
                <DialogTitle>邀请新成员</DialogTitle>
                <DialogDescription>
                  输入对方已注册的邮箱地址，将其加入当前公司
                </DialogDescription>
              </DialogHeader>
              <div className="space-y-4 pt-2">
                <div className="space-y-2">
                  <Label htmlFor="invite-email">邮箱地址</Label>
                  <Input
                    id="invite-email"
                    type="email"
                    placeholder="colleague@example.com"
                    value={inviteEmail}
                    onChange={(event) => setInviteEmail(event.target.value)}
                    onKeyDown={(event) =>
                      event.key === "Enter" && handleInvite()
                    }
                  />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="invite-role">角色</Label>
                  <Select value={inviteRole} onValueChange={setInviteRole}>
                    <SelectTrigger id="invite-role">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      {assignableRoles.map((role) => (
                        <SelectItem key={role.value} value={role.value}>
                          <RoleOptionLabel option={role} />
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
                <div className="flex justify-end gap-2 pt-2">
                  <Button
                    variant="outline"
                    onClick={() => setInviteOpen(false)}
                  >
                    取消
                  </Button>
                  <Button
                    onClick={handleInvite}
                    disabled={!inviteEmail.trim() || inviteMutation.isPending}
                  >
                    {inviteMutation.isPending ? "邀请中..." : "发送邀请"}
                  </Button>
                </div>
              </div>
            </DialogContent>
          </Dialog>
        )}
      </div>

      <div className="grid gap-4 md:grid-cols-3">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">总成员数</CardTitle>
            <Users className="h-4 w-4 text-primary" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.total}</div>
            <p className="text-xs text-muted-foreground">当前公司团队规模</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">所有者</CardTitle>
            <Crown className="h-4 w-4 text-yellow-500" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.owners}</div>
            <p className="text-xs text-muted-foreground">拥有完整管理权限</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">管理员</CardTitle>
            <Shield className="h-4 w-4 text-blue-500" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.admins}</div>
            <p className="text-xs text-muted-foreground">可管理成员和数据</p>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <div className="flex flex-col gap-4 md:flex-row md:items-center md:justify-between">
            <div>
              <CardTitle>成员列表</CardTitle>
              <CardDescription>
                {isAdmin
                  ? "管理公司成员的角色和权限"
                  : "查看当前公司的所有成员"}
              </CardDescription>
            </div>
            <div className="relative w-full md:w-64">
              <Search className="absolute left-3 top-3 h-4 w-4 text-muted-foreground" />
              <Input
                placeholder="搜索成员..."
                value={searchTerm}
                onChange={(event) => setSearchTerm(event.target.value)}
                className="pl-10"
              />
            </div>
          </div>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <div className="flex items-center justify-center py-8">
              <Loader2 className="h-8 w-8 animate-spin text-primary" />
            </div>
          ) : members.length === 0 ? (
            <div className="py-8 text-center text-muted-foreground">
              {overview.empty_message}
            </div>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>成员</TableHead>
                  <TableHead>邮箱</TableHead>
                  <TableHead>角色</TableHead>
                  {isAdmin && (
                    <TableHead className="text-right">操作</TableHead>
                  )}
                </TableRow>
              </TableHeader>
              <TableBody>
                {members.map((member) => (
                  <TableRow key={member.id}>
                    <TableCell>
                      <div className="flex items-center gap-3">
                        <Avatar className="h-8 w-8">
                          <AvatarFallback className="text-xs">
                            {member.avatar_fallback}
                          </AvatarFallback>
                        </Avatar>
                        <span className="font-medium">
                          {member.display_name}
                        </span>
                      </div>
                    </TableCell>
                    <TableCell className="text-muted-foreground">
                      {member.email}
                    </TableCell>
                    <TableCell>
                      {member.can_edit_role ? (
                        <Select
                          value={member.role}
                          onValueChange={(newRole) =>
                            updateRoleMutation.mutate({
                              memberId: member.id,
                              role: newRole,
                            })
                          }
                        >
                          <SelectTrigger className="h-8 w-32">
                            <SelectValue />
                          </SelectTrigger>
                          <SelectContent>
                            {assignableRoles.map((role) => (
                              <SelectItem key={role.value} value={role.value}>
                                <RoleOptionLabel option={role} />
                              </SelectItem>
                            ))}
                          </SelectContent>
                        </Select>
                      ) : (
                        <RoleBadgeView badge={member.role_badge} />
                      )}
                    </TableCell>
                    {isAdmin && (
                      <TableCell className="text-right">
                        {member.can_remove && (
                          <AlertDialog>
                            <AlertDialogTrigger asChild>
                              <Button
                                variant="ghost"
                                size="sm"
                                className="text-destructive hover:text-destructive"
                              >
                                <Trash2 className="h-4 w-4" />
                              </Button>
                            </AlertDialogTrigger>
                            <AlertDialogContent>
                              <AlertDialogHeader>
                                <AlertDialogTitle>
                                  确认移除成员
                                </AlertDialogTitle>
                                <AlertDialogDescription>
                                  确定要将{" "}
                                  <span className="font-medium">
                                    {member.display_name}
                                  </span>{" "}
                                  从公司中移除吗？此操作不可撤销。
                                </AlertDialogDescription>
                              </AlertDialogHeader>
                              <AlertDialogFooter>
                                <AlertDialogCancel>取消</AlertDialogCancel>
                                <AlertDialogAction
                                  className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
                                  onClick={() =>
                                    removeMutation.mutate(member.id)
                                  }
                                >
                                  确认移除
                                </AlertDialogAction>
                              </AlertDialogFooter>
                            </AlertDialogContent>
                          </AlertDialog>
                        )}
                      </TableCell>
                    )}
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
