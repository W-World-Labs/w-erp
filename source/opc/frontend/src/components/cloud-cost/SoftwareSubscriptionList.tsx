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
import { AlertTriangle, Edit, FlaskConical, Trash2, Package, CheckCircle2, XCircle } from "lucide-react";
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
import SoftwareSubscriptionDialog from "./SoftwareSubscriptionDialog";

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

const SoftwareSubscriptionList = ({ onEdit }: { onEdit?: () => void }) => {
  const [subscriptions, setSubscriptions] = useState<any[]>([]);
  const [emptyMessage, setEmptyMessage] = useState("还没有添加软件订阅");
  const [loading, setLoading] = useState(true);
  const [editingSub, setEditingSub] = useState<any>(null);
  const [deletingAction, setDeletingAction] = useState<DeleteAction | null>(null);

  const fetchSubscriptions = async () => {
    try {
      const overview = await api.get<{
        software: any[];
        empty_messages?: Record<string, string>;
      }>(CLOUD_COST_OVERVIEW_ENDPOINT);
      setSubscriptions(overview?.software || []);
      setEmptyMessage(
        overview?.empty_messages?.software || "还没有添加软件订阅",
      );
    } catch (error: any) {
      toast.error("加载失败：" + error.message);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchSubscriptions();
  }, []);

  const handleDelete = async () => {
    if (!deletingAction) return;

    try {
      await api.runAction(deletingAction);
      toast.success(deletingAction.success_message);
      fetchSubscriptions();
      onEdit?.();
    } catch (error: any) {
      toast.error("删除失败：" + error.message);
    } finally {
      setDeletingAction(null);
    }
  };

  const handleSoftwareStatusAction = async (action: BackendAction) => {
    try {
      await api.runAction(action);
      toast.success(action.success_message || "软件订阅状态已更新");
      fetchSubscriptions();
      onEdit?.();
    } catch (error: any) {
      toast.error("状态更新失败：" + error.message);
    }
  };

  if (loading) {
    return <div className="text-center py-8">加载中...</div>;
  }

  if (subscriptions.length === 0) {
    return (
      <div className="text-center py-12 bg-muted/50 rounded-lg">
        <Package className="h-12 w-12 mx-auto text-muted-foreground mb-4" />
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
              <TableHead>软件名称</TableHead>
              <TableHead>供应商</TableHead>
              <TableHead>套餐</TableHead>
              <TableHead>许可数</TableHead>
              <TableHead>总价格</TableHead>
              <TableHead>计费周期</TableHead>
              <TableHead>下次计费</TableHead>
              <TableHead>自动续费</TableHead>
              <TableHead>状态</TableHead>
              <TableHead className="text-right">操作</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {subscriptions.map((sub) => (
              <TableRow key={sub.id}>
                <TableCell className="font-medium">
                  {sub.software_name}
                </TableCell>
                <TableCell>{sub.vendor}</TableCell>
                <TableCell className="text-muted-foreground">
                  {sub.plan_name || "-"}
                </TableCell>
                <TableCell>{sub.license_count}</TableCell>
                <TableCell>{sub.total_price_label}</TableCell>
                <TableCell>{sub.billing_cycle_label}</TableCell>
                <TableCell>{sub.next_billing_date_label}</TableCell>
                <TableCell>
                  <RenewalIcon badge={sub.auto_renewal_badge} />
                </TableCell>
                <TableCell>
                  <Badge
                    variant={sub.status_badge?.variant || "secondary"}
                    className={sub.status_badge?.class_name}
                  >
                    {sub.status_badge?.label}
                  </Badge>
                </TableCell>
                <TableCell className="text-right">
                  <div className="flex justify-end gap-2">
                    {sub.can_activate && sub.activate_action && (
                      <Button
                        variant="ghost"
                        size="icon"
                        title={sub.activate_action.label || "恢复使用"}
                        onClick={() =>
                          handleSoftwareStatusAction(sub.activate_action)
                        }
                      >
                        <CheckCircle2 className="h-4 w-4" />
                      </Button>
                    )}
                    {sub.can_start_trial && sub.start_trial_action && (
                      <Button
                        variant="ghost"
                        size="icon"
                        title={sub.start_trial_action.label || "标记试用"}
                        onClick={() =>
                          handleSoftwareStatusAction(sub.start_trial_action)
                        }
                      >
                        <FlaskConical className="h-4 w-4" />
                      </Button>
                    )}
                    {sub.can_mark_expired && sub.mark_expired_action && (
                      <Button
                        variant="ghost"
                        size="icon"
                        title={sub.mark_expired_action.label || "标记过期"}
                        onClick={() =>
                          handleSoftwareStatusAction(sub.mark_expired_action)
                        }
                      >
                        <AlertTriangle className="h-4 w-4" />
                      </Button>
                    )}
                    {sub.can_cancel && sub.cancel_action && (
                      <Button
                        variant="ghost"
                        size="icon"
                        title={sub.cancel_action.label || "取消订阅"}
                        onClick={() =>
                          handleSoftwareStatusAction(sub.cancel_action)
                        }
                      >
                        <XCircle className="h-4 w-4" />
                      </Button>
                    )}
                    {sub.can_edit && (
                      <Button
                        variant="ghost"
                        size="icon"
                        onClick={() => setEditingSub(sub)}
                      >
                        <Edit className="h-4 w-4" />
                      </Button>
                    )}
                    {sub.can_delete && sub.delete_action && (
                      <Button
                        variant="ghost"
                        size="icon"
                        onClick={() => setDeletingAction(sub.delete_action)}
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

      <SoftwareSubscriptionDialog
        open={!!editingSub}
        onOpenChange={(open) => !open && setEditingSub(null)}
        initialData={editingSub}
        onSuccess={() => {
          fetchSubscriptions();
          onEdit?.();
          setEditingSub(null);
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

export default SoftwareSubscriptionList;
