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
import { CheckCircle2, Edit, Trash2, DollarSign } from "lucide-react";
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
import CloudBillDialog from "./CloudBillDialog";

const CLOUD_COST_OVERVIEW_ENDPOINT = "/api/v1/cloud_cost/overview";

interface DeleteAction {
  method: "DELETE";
  endpoint: string;
  confirm_message: string;
  success_title?: string;
  success_message: string;
}

const CloudBillList = ({ onEdit }: { onEdit?: () => void }) => {
  const [bills, setBills] = useState<any[]>([]);
  const [emptyMessage, setEmptyMessage] = useState("还没有账单记录");
  const [loading, setLoading] = useState(true);
  const [editingBill, setEditingBill] = useState<any>(null);
  const [deletingAction, setDeletingAction] = useState<DeleteAction | null>(null);

  const fetchBills = async () => {
    try {
      const overview = await api.get<{
        bills: any[];
        empty_messages?: Record<string, string>;
      }>(CLOUD_COST_OVERVIEW_ENDPOINT);
      setBills(overview?.bills || []);
      setEmptyMessage(overview?.empty_messages?.bills || "还没有账单记录");
    } catch (error: any) {
      toast.error("加载失败：" + error.message);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchBills();
  }, []);

  const handleDelete = async () => {
    if (!deletingAction) return;

    try {
      await api.runAction(deletingAction);
      toast.success(deletingAction.success_message);
      fetchBills();
      onEdit?.();
    } catch (error: any) {
      toast.error("删除失败：" + error.message);
    } finally {
      setDeletingAction(null);
    }
  };

  const handleMarkPaid = async (action: BackendAction) => {
    try {
      await api.runAction(action);
      toast.success(action.success_message || "已标记为已支付");
      fetchBills();
      onEdit?.();
    } catch (error: any) {
      toast.error("标记失败：" + error.message);
    }
  };

  if (loading) {
    return <div className="text-center py-8">加载中...</div>;
  }

  if (bills.length === 0) {
    return (
      <div className="text-center py-12 bg-muted/50 rounded-lg">
        <DollarSign className="h-12 w-12 mx-auto text-muted-foreground mb-4" />
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
              <TableHead>账期</TableHead>
              <TableHead>云服务商</TableHead>
              <TableHead>账单项目</TableHead>
              <TableHead>账单日期</TableHead>
              <TableHead className="text-right">总金额</TableHead>
              <TableHead className="text-right">优惠</TableHead>
              <TableHead className="text-right">实付金额</TableHead>
              <TableHead>支付状态</TableHead>
              <TableHead className="text-right">操作</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {bills.map((bill) => (
              <TableRow key={bill.id}>
                <TableCell className="font-medium">
                  {bill.billing_period_label}
                </TableCell>
                <TableCell>{bill.provider_name || "-"}</TableCell>
                <TableCell>{bill.item_name}</TableCell>
                <TableCell>{bill.bill_date_label}</TableCell>
                <TableCell className="text-right">
                  {bill.total_amount_label}
                </TableCell>
                <TableCell className="text-right">
                  {bill.discount_amount_label}
                </TableCell>
                <TableCell className="text-right font-medium">
                  {bill.actual_amount_label}
                </TableCell>
                <TableCell>
                  <Badge
                    variant={bill.status_badge?.variant || "secondary"}
                    className={bill.status_badge?.class_name}
                  >
                    {bill.status_badge?.label}
                  </Badge>
                </TableCell>
                <TableCell className="text-right">
                  <div className="flex justify-end gap-2">
                    {bill.can_mark_paid && bill.mark_paid_action && (
                      <Button
                        variant="ghost"
                        size="icon"
                        title={bill.mark_paid_action.label || "标记已支付"}
                        onClick={() => handleMarkPaid(bill.mark_paid_action)}
                      >
                        <CheckCircle2 className="h-4 w-4" />
                      </Button>
                    )}
                    {bill.can_edit && bill.update_action && (
                      <Button
                        variant="ghost"
                        size="icon"
                        onClick={() => setEditingBill(bill)}
                      >
                        <Edit className="h-4 w-4" />
                      </Button>
                    )}
                    {bill.can_delete && bill.delete_action && (
                      <Button
                        variant="ghost"
                        size="icon"
                        onClick={() => setDeletingAction(bill.delete_action)}
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

      <CloudBillDialog
        open={!!editingBill}
        onOpenChange={(open) => !open && setEditingBill(null)}
        initialData={editingBill}
        onSuccess={() => {
          fetchBills();
          onEdit?.();
          setEditingBill(null);
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

export default CloudBillList;
