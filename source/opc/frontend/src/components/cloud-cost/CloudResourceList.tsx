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
import { AlertTriangle, Edit, PauseCircle, PlayCircle, PowerOff, Trash2, Server } from "lucide-react";
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
import CloudResourceDialog from "./CloudResourceDialog";

const CLOUD_COST_OVERVIEW_ENDPOINT = "/api/v1/cloud_cost/overview";

interface DeleteAction {
  method: "DELETE";
  endpoint: string;
  confirm_message: string;
  success_title?: string;
  success_message: string;
}

const CloudResourceList = ({ onEdit }: { onEdit?: () => void }) => {
  const [resources, setResources] = useState<any[]>([]);
  const [emptyMessage, setEmptyMessage] = useState("还没有添加云资源");
  const [loading, setLoading] = useState(true);
  const [editingResource, setEditingResource] = useState<any>(null);
  const [deletingAction, setDeletingAction] = useState<DeleteAction | null>(null);

  const fetchResources = async () => {
    try {
      const overview = await api.get<{
        resources: any[];
        empty_messages?: Record<string, string>;
      }>(CLOUD_COST_OVERVIEW_ENDPOINT);
      setResources(overview?.resources || []);
      setEmptyMessage(
        overview?.empty_messages?.resources || "还没有添加云资源",
      );
    } catch (error: any) {
      toast.error("加载失败：" + error.message);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchResources();
  }, []);

  const handleDelete = async () => {
    if (!deletingAction) return;

    try {
      await api.runAction(deletingAction);
      toast.success(deletingAction.success_message);
      fetchResources();
      onEdit?.();
    } catch (error: any) {
      toast.error("删除失败：" + error.message);
    } finally {
      setDeletingAction(null);
    }
  };

  const handleResourceStatusAction = async (action: BackendAction) => {
    try {
      await api.runAction(action);
      toast.success(action.success_message || "云资源状态已更新");
      fetchResources();
      onEdit?.();
    } catch (error: any) {
      toast.error("状态更新失败：" + error.message);
    }
  };

  if (loading) {
    return <div className="text-center py-8">加载中...</div>;
  }

  if (resources.length === 0) {
    return (
      <div className="text-center py-12 bg-muted/50 rounded-lg">
        <Server className="h-12 w-12 mx-auto text-muted-foreground mb-4" />
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
              <TableHead>资源名称</TableHead>
              <TableHead>云服务商</TableHead>
              <TableHead>类型</TableHead>
              <TableHead>规格</TableHead>
              <TableHead>区域</TableHead>
              <TableHead>月费用</TableHead>
              <TableHead>到期日期</TableHead>
              <TableHead>状态</TableHead>
              <TableHead className="text-right">操作</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {resources.map((resource) => (
              <TableRow key={resource.id}>
                <TableCell className="font-medium">
                  {resource.resource_name}
                </TableCell>
                <TableCell>{resource.provider_name}</TableCell>
                <TableCell>{resource.resource_type_label}</TableCell>
                <TableCell className="text-muted-foreground">
                  {resource.specification || "-"}
                </TableCell>
                <TableCell>{resource.region || "-"}</TableCell>
                <TableCell>{resource.monthly_cost_label}</TableCell>
                <TableCell>{resource.end_date_label}</TableCell>
                <TableCell>
                  <Badge
                    variant={resource.status_badge?.variant || "secondary"}
                    className={resource.status_badge?.class_name}
                  >
                    {resource.status_badge?.label}
                  </Badge>
                </TableCell>
                <TableCell className="text-right">
                  <div className="flex justify-end gap-2">
                    {resource.can_start && resource.start_action && (
                      <Button
                        variant="ghost"
                        size="icon"
                        title={resource.start_action.label || "启动资源"}
                        onClick={() =>
                          handleResourceStatusAction(resource.start_action)
                        }
                      >
                        <PlayCircle className="h-4 w-4" />
                      </Button>
                    )}
                    {resource.can_stop && resource.stop_action && (
                      <Button
                        variant="ghost"
                        size="icon"
                        title={resource.stop_action.label || "停止资源"}
                        onClick={() =>
                          handleResourceStatusAction(resource.stop_action)
                        }
                      >
                        <PauseCircle className="h-4 w-4" />
                      </Button>
                    )}
                    {resource.can_mark_expired && resource.mark_expired_action && (
                      <Button
                        variant="ghost"
                        size="icon"
                        title={
                          resource.mark_expired_action.label || "标记过期"
                        }
                        onClick={() =>
                          handleResourceStatusAction(
                            resource.mark_expired_action,
                          )
                        }
                      >
                        <AlertTriangle className="h-4 w-4" />
                      </Button>
                    )}
                    {resource.can_terminate && resource.terminate_action && (
                      <Button
                        variant="ghost"
                        size="icon"
                        title={resource.terminate_action.label || "终止资源"}
                        onClick={() =>
                          handleResourceStatusAction(resource.terminate_action)
                        }
                      >
                        <PowerOff className="h-4 w-4" />
                      </Button>
                    )}
                    {resource.can_edit && (
                      <Button
                        variant="ghost"
                        size="icon"
                        onClick={() => setEditingResource(resource)}
                      >
                        <Edit className="h-4 w-4" />
                      </Button>
                    )}
                    {resource.can_delete && resource.delete_action && (
                      <Button
                        variant="ghost"
                        size="icon"
                        onClick={() => setDeletingAction(resource.delete_action)}
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

      <CloudResourceDialog
        open={!!editingResource}
        onOpenChange={(open) => !open && setEditingResource(null)}
        initialData={editingResource}
        onSuccess={() => {
          fetchResources();
          onEdit?.();
          setEditingResource(null);
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

export default CloudResourceList;
