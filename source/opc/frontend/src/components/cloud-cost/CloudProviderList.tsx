import { useState, useEffect } from "react";
import { api } from "@/integrations/api/client";
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
import { Edit, Trash2, Cloud } from "lucide-react";
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
import CloudProviderDialog from "./CloudProviderDialog";

const CLOUD_COST_OVERVIEW_ENDPOINT = "/api/v1/cloud_cost/overview";

interface DeleteAction {
  method: "DELETE";
  endpoint: string;
  confirm_message: string;
  success_title?: string;
  success_message: string;
}

const CloudProviderList = ({ onEdit }: { onEdit?: () => void }) => {
  const [providers, setProviders] = useState<any[]>([]);
  const [emptyMessage, setEmptyMessage] = useState("还没有添加云服务商账户");
  const [loading, setLoading] = useState(true);
  const [editingProvider, setEditingProvider] = useState<any>(null);
  const [deletingAction, setDeletingAction] = useState<DeleteAction | null>(null);

  const fetchProviders = async () => {
    try {
      const overview = await api.get<{
        providers: any[];
        empty_messages?: Record<string, string>;
      }>(CLOUD_COST_OVERVIEW_ENDPOINT);
      setProviders(overview?.providers || []);
      setEmptyMessage(
        overview?.empty_messages?.providers || "还没有添加云服务商账户",
      );
    } catch (error: any) {
      toast.error("加载失败：" + error.message);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchProviders();
  }, []);

  const handleDelete = async () => {
    if (!deletingAction) return;

    try {
      await api.runAction(deletingAction);
      toast.success(deletingAction.success_message);
      fetchProviders();
      onEdit?.();
    } catch (error: any) {
      toast.error("删除失败：" + error.message);
    } finally {
      setDeletingAction(null);
    }
  };

  if (loading) {
    return <div className="text-center py-8">加载中...</div>;
  }

  if (providers.length === 0) {
    return (
      <div className="text-center py-12 bg-muted/50 rounded-lg">
        <Cloud className="h-12 w-12 mx-auto text-muted-foreground mb-4" />
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
              <TableHead>服务商</TableHead>
              <TableHead>账户名称</TableHead>
              <TableHead>账户ID</TableHead>
              <TableHead>区域</TableHead>
              <TableHead>联系人</TableHead>
              <TableHead>状态</TableHead>
              <TableHead className="text-right">操作</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {providers.map((provider) => (
              <TableRow key={provider.id}>
                <TableCell className="font-medium">
                  {provider.provider_type_label}
                </TableCell>
                <TableCell>{provider.account_name}</TableCell>
                <TableCell className="text-muted-foreground">
                  {provider.account_id || "-"}
                </TableCell>
                <TableCell>{provider.region || "-"}</TableCell>
                <TableCell>{provider.contact_person || "-"}</TableCell>
                <TableCell>
                  <Badge
                    variant={provider.status_badge?.variant || "secondary"}
                    className={provider.status_badge?.class_name}
                  >
                    {provider.status_badge?.label}
                  </Badge>
                </TableCell>
                <TableCell className="text-right">
                  <div className="flex justify-end gap-2">
                    {provider.can_edit && (
                      <Button
                        variant="ghost"
                        size="icon"
                        onClick={() => setEditingProvider(provider)}
                      >
                        <Edit className="h-4 w-4" />
                      </Button>
                    )}
                    {provider.can_delete && provider.delete_action && (
                      <Button
                        variant="ghost"
                        size="icon"
                        onClick={() => setDeletingAction(provider.delete_action)}
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

      <CloudProviderDialog
        open={!!editingProvider}
        onOpenChange={(open) => !open && setEditingProvider(null)}
        initialData={editingProvider}
        onSuccess={() => {
          fetchProviders();
          onEdit?.();
          setEditingProvider(null);
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

export default CloudProviderList;
