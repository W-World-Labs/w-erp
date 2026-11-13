import { useState, useEffect } from "react";
import { api, type BackendAction } from "@/integrations/api/client";
import { toast } from "sonner";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { AlertTriangle, ArrowRightLeft, Edit, Trash2, Globe, CheckCircle2, XCircle } from "lucide-react";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import DomainSubscriptionDialog from "./DomainSubscriptionDialog";

const CLOUD_COST_OVERVIEW_ENDPOINT = "/api/v1/cloud_cost/overview";

interface DeleteAction {
  method: "DELETE";
  endpoint: string;
  confirm_message: string;
  success_title?: string;
  success_message: string;
}

const RenewalIcon = ({ badge }: { badge?: any }) =>
  badge?.icon === "CheckCircle2" ? (
    <CheckCircle2 className={`h-4 w-4 ${badge.class_name || ""}`} />
  ) : (
    <XCircle className={`h-4 w-4 ${badge?.class_name || ""}`} />
  );

const DomainSubscriptionList = ({ onEdit }: { onEdit?: () => void }) => {
  const [domains, setDomains] = useState<any[]>([]);
  const [emptyMessage, setEmptyMessage] = useState("还没有添加域名");
  const [loading, setLoading] = useState(true);
  const [editingDomain, setEditingDomain] = useState<any>(null);
  const [deletingAction, setDeletingAction] = useState<DeleteAction | null>(null);

  const fetchDomains = async () => {
    try {
      const overview = await api.get<{
        domains: any[];
        empty_messages?: Record<string, string>;
      }>(CLOUD_COST_OVERVIEW_ENDPOINT);
      setDomains(overview?.domains || []);
      setEmptyMessage(overview?.empty_messages?.domains || "还没有添加域名");
    } catch (error: any) {
      toast.error("加载失败：" + error.message);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchDomains();
  }, []);

  const handleDelete = async () => {
    if (!deletingAction) return;

    try {
      await api.runAction(deletingAction);
      toast.success(deletingAction.success_message);
      fetchDomains();
      onEdit?.();
    } catch (error: any) {
      toast.error("删除失败：" + error.message);
    } finally {
      setDeletingAction(null);
    }
  };

  const handleDomainStatusAction = async (action: BackendAction) => {
    try {
      await api.runAction(action);
      toast.success(action.success_message || "域名状态已更新");
      fetchDomains();
      onEdit?.();
    } catch (error: any) {
      toast.error("状态更新失败：" + error.message);
    }
  };

  if (loading) {
    return <div className="text-center py-8">加载中...</div>;
  }

  if (domains.length === 0) {
    return (
      <div className="text-center py-12 bg-muted/50 rounded-lg">
        <Globe className="h-12 w-12 mx-auto text-muted-foreground mb-4" />
        <p className="text-muted-foreground">{emptyMessage}</p>
      </div>
    );
  }

  return (
    <>
      <div className="border rounded-lg">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>域名</TableHead>
              <TableHead>注册商</TableHead>
              <TableHead>注册日期</TableHead>
              <TableHead>到期日期</TableHead>
              <TableHead>剩余天数</TableHead>
              <TableHead>首次购买</TableHead>
              <TableHead>年度续费</TableHead>
              <TableHead>自动续费</TableHead>
              <TableHead>状态</TableHead>
              <TableHead className="text-right">操作</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {domains.map((domain) => (
              <TableRow key={domain.id}>
                <TableCell className="font-medium">
                  {domain.domain_name}
                </TableCell>
                <TableCell>{domain.registrar}</TableCell>
                <TableCell>{domain.registration_date_label}</TableCell>
                <TableCell>{domain.expiry_date_label}</TableCell>
                <TableCell>
                  <span className={domain.expiry_state?.class_name || ""}>
                    {domain.expiry_state?.label}
                  </span>
                </TableCell>
                <TableCell>{domain.initial_purchase_cost_label}</TableCell>
                <TableCell>{domain.annual_fee_label}</TableCell>
                <TableCell>
                  <RenewalIcon badge={domain.auto_renewal_badge} />
                </TableCell>
                <TableCell>
                  <Badge
                    variant={domain.status_badge?.variant || "secondary"}
                    className={domain.status_badge?.class_name}
                  >
                    {domain.status_badge?.label}
                  </Badge>
                </TableCell>
                <TableCell className="text-right">
                  <div className="flex justify-end gap-2">
                    {domain.can_activate && domain.activate_action && (
                      <Button
                        variant="ghost"
                        size="icon"
                        title={domain.activate_action.label || "恢复正常"}
                        onClick={() =>
                          handleDomainStatusAction(domain.activate_action)
                        }
                      >
                        <CheckCircle2 className="h-4 w-4" />
                      </Button>
                    )}
                    {domain.can_mark_pending_transfer &&
                      domain.mark_pending_transfer_action && (
                      <Button
                        variant="ghost"
                        size="icon"
                        title={
                          domain.mark_pending_transfer_action.label ||
                          "标记转移中"
                        }
                        onClick={() =>
                          handleDomainStatusAction(
                            domain.mark_pending_transfer_action,
                          )
                        }
                      >
                        <ArrowRightLeft className="h-4 w-4" />
                      </Button>
                    )}
                    {domain.can_mark_expired && domain.mark_expired_action && (
                      <Button
                        variant="ghost"
                        size="icon"
                        title={
                          domain.mark_expired_action.label || "标记过期"
                        }
                        onClick={() =>
                          handleDomainStatusAction(domain.mark_expired_action)
                        }
                      >
                        <AlertTriangle className="h-4 w-4" />
                      </Button>
                    )}
                    {domain.can_edit && (
                      <Button
                        variant="ghost"
                        size="icon"
                        onClick={() => setEditingDomain(domain)}
                      >
                        <Edit className="h-4 w-4" />
                      </Button>
                    )}
                    {domain.can_delete && domain.delete_action && (
                      <Button
                        variant="ghost"
                        size="icon"
                        onClick={() => setDeletingAction(domain.delete_action)}
                      >
                        <Trash2 className="h-4 w-4" />
                      </Button>
                    )}
                  </div>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </div>

      <DomainSubscriptionDialog
        open={!!editingDomain}
        onOpenChange={(open) => !open && setEditingDomain(null)}
        initialData={editingDomain}
        onSuccess={() => {
          fetchDomains();
          onEdit?.();
          setEditingDomain(null);
        }}
      />

      <AlertDialog
        open={!!deletingAction}
        onOpenChange={(open) => !open && setDeletingAction(null)}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>确认删除</AlertDialogTitle>
            <AlertDialogDescription>
              {deletingAction?.confirm_message}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>取消</AlertDialogCancel>
            <AlertDialogAction onClick={handleDelete}>确定</AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  );
};

export default DomainSubscriptionList;
