import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { useToast } from "@/hooks/use-toast";
import { History, Download, RotateCcw, FileText, Clock, CheckCircle, Info } from "lucide-react";
import type { LucideIcon } from "lucide-react";
import {
  getMaterialVersionHistory,
  rollbackMaterialVersionAction,
  type MaterialVersion,
  type MaterialVersionAction,
} from "@/lib/materialVersions";
import { files } from "@/integrations/api/client";

const badgeIcons: Record<string, LucideIcon> = {
  CheckCircle,
};

interface MaterialVersionHistoryProps {
  materialId: string;
  materialName: string;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export const MaterialVersionHistory = ({
  materialId,
  materialName,
  open,
  onOpenChange
}: MaterialVersionHistoryProps) => {
  const { toast } = useToast();
  const queryClient = useQueryClient();

  // 查询版本历史
  const { data: history, isLoading } = useQuery({
    queryKey: ['material-versions', materialId],
    queryFn: () => getMaterialVersionHistory(materialId),
    enabled: open
  });
  const versions = history?.versions ?? [];
  const currentVersion = history?.current_version ?? null;

  // 回滚到指定版本
  const rollbackMutation = useMutation({
    mutationFn: (action: MaterialVersionAction) =>
      rollbackMaterialVersionAction(action),
    onSuccess: ({ version }, action) => {
      queryClient.invalidateQueries({ queryKey: ['material-versions', materialId] });
      queryClient.invalidateQueries({ queryKey: ['application_materials'] });
      toast({
        title: action.success_title || "回滚成功",
        description: action.success_message || `已回滚到版本 ${version.version_number}`
      });
    },
    onError: (error: Error) => {
      toast({
        title: "回滚失败",
        description: error.message,
        variant: "destructive"
      });
    }
  });

  // 下载指定版本
  const downloadVersion = async (version: MaterialVersion) => {
    try {
      const data = await files.download(version.file_path);

      const url = URL.createObjectURL(data);
      const a = document.createElement('a');
      a.href = url;
      a.download = version.file_name;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);

      toast({
        title: "下载成功",
        description: `已下载版本 ${version.version_number}`
      });
    } catch (error: any) {
      toast({
        title: "下载失败",
        description: error.message,
        variant: "destructive"
      });
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-4xl max-h-[85vh]">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <History className="h-5 w-5" />
            版本历史：{materialName}
          </DialogTitle>
          <DialogDescription>
            查看材料的所有版本，支持版本对比和回滚
          </DialogDescription>
        </DialogHeader>

        {isLoading ? (
          <div className="text-center py-8 text-muted-foreground">加载中...</div>
        ) : versions.length === 0 ? (
          <Alert>
            <Info className="h-4 w-4" />
            <AlertDescription>
              该材料暂无版本历史记录
            </AlertDescription>
          </Alert>
        ) : (
          <>
            {/* 当前版本提示 */}
            {currentVersion && (
              <Alert className="border-primary bg-primary/5">
                <CheckCircle className="h-4 w-4 text-primary" />
                <AlertDescription>
                  当前使用版本 {currentVersion.version_number} - {currentVersion.file_name}
                  （{currentVersion.created_at_label}）
                </AlertDescription>
              </Alert>
            )}

            <ScrollArea className="h-[500px] pr-4">
              <div className="space-y-3">
                {versions.map((version) => (
                  <Card 
                    key={version.id}
                    className={version.card_class_name}
                  >
                    <CardContent className="p-4">
                      <div className="flex items-start justify-between">
                        <div className="flex-1 space-y-2">
                          {/* 版本信息头部 */}
                          <div className="flex items-center gap-2">
                            <Badge
                              variant={version.version_badge.variant || "secondary"}
                              className={version.version_badge.class_name || ""}
                            >
                              {version.version_badge.label}
                            </Badge>
                            {version.current_badge && (() => {
                              const CurrentBadgeIcon = badgeIcons[version.current_badge.icon || ""] || CheckCircle;
                              return (
                                <Badge
                                  variant={version.current_badge.variant || "outline"}
                                  className={`gap-1 ${version.current_badge.class_name || ""}`}
                                >
                                  <CurrentBadgeIcon className="h-3 w-3" />
                                  {version.current_badge.label}
                                </Badge>
                              );
                            })()}
                            {version.latest_badge && (
                              <Badge
                                variant={version.latest_badge.variant || "outline"}
                                className={version.latest_badge.class_name || ""}
                              >
                                {version.latest_badge.label}
                              </Badge>
                            )}
                          </div>

                          {/* 文件信息 */}
                          <div className="flex items-center gap-2 text-sm">
                            <FileText className="h-4 w-4 text-muted-foreground" />
                            <span className="font-medium">{version.file_name}</span>
                            <span className="text-muted-foreground">({version.file_size})</span>
                          </div>

                          {/* 时间和创建者 */}
                          <div className="flex items-center gap-4 text-sm text-muted-foreground">
                            <div className="flex items-center gap-1">
                              <Clock className="h-3 w-3" />
                              {version.created_at_label}
                            </div>
                          </div>

                          {/* 变更说明 */}
                          {version.change_notes && (
                            <div className="bg-muted/50 rounded p-2 text-sm">
                              <div className="font-medium text-muted-foreground mb-1">变更说明：</div>
                              <div>{version.change_notes}</div>
                            </div>
                          )}

                          {/* 描述 */}
                          {version.description && (
                            <div className="text-sm text-muted-foreground">
                              {version.description}
                            </div>
                          )}
                        </div>

                        {/* 操作按钮 */}
                        <div className="flex flex-col gap-2 ml-4">
                          <Button
                            size="sm"
                            variant="outline"
                            onClick={() => downloadVersion(version)}
                            className="gap-1"
                          >
                            <Download className="h-3 w-3" />
                            下载
                          </Button>
                          {version.rollback_enabled && version.rollback_action && (
                            <Button
                              size="sm"
                              variant="default"
                              onClick={() => {
                                const action = version.rollback_action;
                                if (action && confirm(action.confirm_message)) {
                                  rollbackMutation.mutate(action);
                                }
                              }}
                              disabled={rollbackMutation.isPending}
                              className="gap-1"
                            >
                              <RotateCcw className="h-3 w-3" />
                              回滚
                            </Button>
                          )}
                        </div>
                      </div>
                    </CardContent>

                    {/* 版本分隔线 */}
                    <Separator className="my-0" />
                  </Card>
                ))}
              </div>
            </ScrollArea>

            {/* 统计信息 */}
            <div className="flex items-center justify-between pt-2 border-t text-sm text-muted-foreground">
              <div>{history?.version_count_label}</div>
              <div>{history?.earliest_version_label}</div>
            </div>
          </>
        )}
      </DialogContent>
    </Dialog>
  );
};
