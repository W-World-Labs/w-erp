import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { api, type BackendAction } from "@/integrations/api/client";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Plus, FileText, AlertCircle, Calendar, Download } from "lucide-react";
import { LicenseDialog } from "@/components/licenses/LicenseDialog";
import { toast } from "sonner";
import { DependencyAlert } from "@/components/common/DependencyAlert";
import {
  AlertDialog, AlertDialogAction, AlertDialogCancel, AlertDialogContent,
  AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogTitle
} from "@/components/ui/alert-dialog";

interface LicenseBadge {
  label: string;
  variant: "default" | "secondary" | "destructive" | "outline";
  class_name?: string;
}

interface DeleteAction {
  method: "DELETE";
  endpoint: string;
  confirm_message: string;
  success_title: string;
  success_message: string;
}

interface LicenseRow {
  id: string;
  license_name: string;
  license_type: string;
  license_type_label: string;
  license_number?: string | null;
  issuing_authority?: string | null;
  issue_date?: string | null;
  issue_date_label: string;
  expiry_date?: string | null;
  expiry_date_label: string;
  expiry_state: {
    label: string;
    class_name: string;
  };
  file_path?: string | null;
  has_file: boolean;
  status_badge: LicenseBadge;
  can_edit: boolean;
  update_action: BackendAction | null;
  can_delete: boolean;
  delete_action: DeleteAction | null;
}

interface LicensesOverview {
  licenses: LicenseRow[];
  permissions: {
    can_manage: boolean;
  };
  create_action: BackendAction | null;
  empty_message: string;
}

const emptyOverview: LicensesOverview = {
  licenses: [],
  permissions: {
    can_manage: false,
  },
  create_action: null,
  empty_message: "还没有证照记录",
};

function LicenseBadgeView({ badge }: { badge: LicenseBadge }) {
  return (
    <Badge variant={badge.variant} className={badge.class_name}>
      {badge.label}
    </Badge>
  );
}

export default function Licenses() {
  const [dialogOpen, setDialogOpen] = useState(false);
  const [editingLicense, setEditingLicense] = useState<any>(null);
  const [licenseToDelete, setLicenseToDelete] = useState<DeleteAction | null>(
    null,
  );

  const { data: overview = emptyOverview, refetch } = useQuery({
    queryKey: ["licenses-overview"],
    queryFn: async () => {
      return api.get<LicensesOverview>("/api/v1/licenses/overview");
    },
  });
  const licenses = overview.licenses;

  const handleEdit = (license: any) => {
    setEditingLicense(license);
    setDialogOpen(true);
  };

  const handleDelete = async (action: DeleteAction) => {
    try {
      await api.runAction(action);
      toast.success(action.success_message);
      refetch();
    } catch (err: any) {
      toast.error(err?.message || "删除失败");
    } finally {
      setLicenseToDelete(null);
    }
  };

  return (
    <div className="space-y-6">
      <DependencyAlert dependency="company-info" />

      <div className="flex justify-between items-center">
        <div>
          <h1 className="text-3xl font-bold">证照管理</h1>
          <p className="text-muted-foreground mt-2">
            管理公司各类证照，及时提醒到期续办
          </p>
        </div>
        <Button disabled={!overview.create_action} onClick={() => { setEditingLicense(null); setDialogOpen(true); }}>
          <Plus className="mr-2 h-4 w-4" />
          添加证照
        </Button>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {licenses.map((license) => (
          <Card key={license.id} className="p-6">
            <div className="flex justify-between items-start mb-4">
              <div className="flex items-center gap-2">
                <FileText className="h-5 w-5 text-primary" />
                <h3 className="font-semibold">{license.license_name}</h3>
              </div>
              <LicenseBadgeView badge={license.status_badge} />
            </div>

            <div className="space-y-2 text-sm">
              <div className="flex items-center gap-2 text-muted-foreground">
                <span>类型：</span>
                <span>{license.license_type_label}</span>
              </div>
              <div className="flex items-center gap-2 text-muted-foreground">
                <span>编号：</span>
                <span className="font-mono">{license.license_number}</span>
              </div>
              {license.issuing_authority && (
                <div className="flex items-center gap-2 text-muted-foreground">
                  <span>发证机关：</span>
                  <span>{license.issuing_authority}</span>
                </div>
              )}
              <div className="flex items-center gap-2 text-muted-foreground">
                <Calendar className="h-4 w-4" />
                <span>发证日期：</span>
                <span>{license.issue_date_label}</span>
              </div>
              {license.expiry_date && (
                <div className="flex items-center gap-2 text-muted-foreground">
                  <AlertCircle className="h-4 w-4" />
                  <span>到期日期：</span>
                  <span>{license.expiry_date_label}</span>
                  <span className={license.expiry_state.class_name}>
                    {license.expiry_state.label}
                  </span>
                </div>
              )}
            </div>

            <div className="flex gap-2 mt-4">
              {license.update_action && (
                <Button variant="outline" size="sm" onClick={() => handleEdit(license)} className="flex-1">
                  编辑
                </Button>
              )}
              {license.has_file && license.file_path && (
                <Button variant="outline" size="sm" asChild>
                  <a href={license.file_path} target="_blank" rel="noopener noreferrer">
                    <Download className="h-4 w-4" />
                  </a>
                </Button>
              )}
              {license.can_delete && overview.permissions.can_manage && license.delete_action && (
                <Button variant="destructive" size="sm" onClick={() => setLicenseToDelete(license.delete_action)}>
                  删除
                </Button>
              )}
            </div>
          </Card>
        ))}
      </div>

      {licenses.length === 0 && (
        <Card className="p-12 text-center">
          <FileText className="h-12 w-12 mx-auto mb-4 text-muted-foreground" />
          <h3 className="text-lg font-semibold mb-2">{overview.empty_message}</h3>
          <p className="text-muted-foreground mb-4">
            开始添加公司的营业执照、税务登记等证照信息
          </p>
          <Button disabled={!overview.create_action} onClick={() => { setEditingLicense(null); setDialogOpen(true); }}>
            <Plus className="mr-2 h-4 w-4" />
            添加第一个证照
          </Button>
        </Card>
      )}

      <LicenseDialog
        open={dialogOpen}
        onOpenChange={setDialogOpen}
        license={editingLicense}
        action={
          editingLicense ? editingLicense.update_action : overview.create_action
        }
        onSuccess={() => {
          refetch();
          setDialogOpen(false);
          setEditingLicense(null);
        }}
      />

      <AlertDialog open={!!licenseToDelete} onOpenChange={() => setLicenseToDelete(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>确认删除</AlertDialogTitle>
            <AlertDialogDescription>
              {licenseToDelete?.confirm_message}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>取消</AlertDialogCancel>
            <AlertDialogAction onClick={() => {
              const action = licenseToDelete;
              if (action) handleDelete(action);
            }}>
              确认删除
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}
