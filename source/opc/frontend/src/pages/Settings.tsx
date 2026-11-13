import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Label } from "@/components/ui/label";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Switch } from "@/components/ui/switch";
import { Building2, Users, Bell, Shield, Database, CheckCircle2, AlertCircle, FileSearch, Brain, Plus, Pencil, Trash2, Mail, type LucideIcon } from "lucide-react";
import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { api, type BackendAction, type DeleteAction } from "@/integrations/api/client";
import { API_ENDPOINTS } from "@/lib/api";
import { useNavigate } from "react-router-dom";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription } from "@/components/ui/dialog";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Progress } from "@/components/ui/progress";
import { AIConfigDialog } from "@/components/settings/AIConfigDialog";
import { EmailConfigDialog } from "@/components/settings/EmailConfigDialog";
import { SystemStatusCard } from "@/components/settings/SystemStatusCard";
import { ErrorLogsPanel } from "@/components/settings/ErrorLogsPanel";
import { DocumentParserSettingsCard } from "@/components/settings/DocumentParserSettingsCard";
import { NotificationChannelsCard } from "@/components/settings/NotificationChannelsCard";
import { CollectionAutomationCard } from "@/components/settings/CollectionAutomationCard";
import { InvoiceMailboxCard } from "@/components/settings/InvoiceMailboxCard";
import { NavigationPreferencesCard } from "@/components/settings/NavigationPreferencesCard";
import { toast } from "sonner";
import { useSettingsBackupActions } from "@/hooks/useSettingsBackupActions";

type SettingsBadge = {
  label: string;
  class_name?: string;
};

type SettingsAction = {
  label: string;
  route: string;
  variant?: "default" | "outline" | "secondary" | "ghost" | "link" | "destructive";
  method?: "GET" | "POST" | "PUT" | "DELETE";
  action_type?: "navigate" | "restore_picker" | "download";
  list_endpoint?: string;
  confirm_message?: string;
  success_message?: string;
};

type BackupListItem = {
  file_name: string;
  size_bytes?: number;
  created_at?: string;
  status?: string;
};

type SettingsConfig = {
  id: string;
  update_action?: BackendAction | null;
  delete_action?: DeleteAction | null;
  [key: string]: any;
};

type SettingsField = {
  id: string;
  label: string;
  value: string;
  read_only: boolean;
};

type SettingsItem = {
  id: string;
  label: string;
  description?: string;
  action?: SettingsAction;
};

type SettingsToggle = {
  id: string;
  label: string;
  description: string;
  checked: boolean;
  disabled: boolean;
};

type SettingsSummaryItem = {
  id: string;
  label: string;
  value: string;
};

type SettingsSection = {
  id: string;
  icon: string;
  title: string;
  description: string;
  fields: SettingsField[];
  items: SettingsItem[];
  toggles: SettingsToggle[];
  summary_items: SettingsSummaryItem[];
  empty_message?: string;
  actions: SettingsAction[];
};

type SetupCheck = {
  module: string;
  dependency: string;
  status: boolean;
  status_label: string;
  card_class_name: string;
  status_icon: string;
  status_icon_class_name: string;
  status_badge: SettingsBadge;
  action_button: {
    visible: boolean;
    label: string;
    route: string;
  };
  route: string;
  description: string;
  impact_label: string;
};

type SettingsOverview = {
  setup_completion: {
    rate: number;
    rate_label: string;
    summary_label: string;
    needs_attention: boolean;
    recommendation_title: string;
    recommendation_intro: string;
    recommendation_items: string[];
    success_message: string;
    success_alert: {
      visible: boolean;
      class_name: string;
      icon: string;
      icon_class_name: string;
      description_class_name: string;
      title: string;
      message: string;
    };
  };
  setup_checks: SetupCheck[];
  report_title: string;
  report_description: string;
  settings_sections: SettingsSection[];
  ai_configuration_create_action: BackendAction | null;
  ai_configurations: SettingsConfig[];
  email_configuration_create_action: BackendAction | null;
  email_configurations: SettingsConfig[];
  w_mail_available: boolean;
  empty_messages: Record<string, string>;
};

const emptySettingsOverview: SettingsOverview = {
  setup_completion: {
    rate: 0,
    rate_label: "0%",
    summary_label: "已完成 0 / 0 项基础数据配置",
    needs_attention: false,
    recommendation_title: "",
    recommendation_intro: "",
    recommendation_items: [],
    success_message: "",
    success_alert: {
      visible: false,
      class_name: "",
      icon: "CheckCircle2",
      icon_class_name: "",
      description_class_name: "",
      title: "",
      message: "",
    },
  },
  setup_checks: [],
  report_title: "",
  report_description: "",
  settings_sections: [],
  ai_configuration_create_action: null,
  ai_configurations: [],
  email_configuration_create_action: null,
  email_configurations: [],
  w_mail_available: false,
  empty_messages: {},
};

function SettingsBadge({ badge }: { badge?: SettingsBadge | null }) {
  if (!badge) return null;
  return (
    <span className={`text-xs px-2 py-1 rounded border ${badge.class_name || ""}`}>
      {badge.label}
    </span>
  );
}

const setupCheckIcons: Record<string, LucideIcon> = {
  AlertCircle,
  CheckCircle2,
};

const settingsSectionIcons: Record<string, LucideIcon> = {
  Bell,
  Building2,
  Database,
  Shield,
  Users,
};

function SetupCheckStatusIcon({ result }: { result: SetupCheck }) {
  const Icon = setupCheckIcons[result.status_icon] || AlertCircle;
  return <Icon className={result.status_icon_class_name} />;
}

function SettingsAlertIcon({
  icon,
  className,
}: {
  icon: string;
  className: string;
}) {
  const Icon = setupCheckIcons[icon] || AlertCircle;
  return <Icon className={className} />;
}

const Settings = () => {
  const navigate = useNavigate();
  const [reportOpen, setReportOpen] = useState(false);
  const [isChecking, setIsChecking] = useState(false);
  const [aiConfigOpen, setAiConfigOpen] = useState(false);
  const [editingConfig, setEditingConfig] = useState<any>(null);
  const [emailConfigOpen, setEmailConfigOpen] = useState(false);
  const [editingEmailConfig, setEditingEmailConfig] = useState<any>(null);

  const {
    data: settingsOverview = emptySettingsOverview,
    refetch: refetchSettingsOverview,
  } = useQuery({
    queryKey: ["settings-overview"],
    queryFn: () => api.get<SettingsOverview>(API_ENDPOINTS.settingsOverview),
  });
  const checkResults = settingsOverview.setup_checks;
  const setupCompletion = settingsOverview.setup_completion;
  const settingsSections = settingsOverview.settings_sections;
  const aiConfigs = settingsOverview.ai_configurations;
  const emailConfigs = settingsOverview.email_configurations;
  const refreshSettingsOverview = () => {
    void refetchSettingsOverview();
  };

  const {
    backupBusy,
    backupList,
    selectedBackup,
    setSelectedBackup,
    restoreOpen,
    setRestoreOpen,
    handleSettingsAction: runBackupAction,
    handleConfirmRestore,
  } = useSettingsBackupActions(refreshSettingsOverview);

  const handleRunCheck = () => {
    setIsChecking(true);
    setTimeout(() => {
      setIsChecking(false);
      setReportOpen(true);
    }, 1500);
  };

  const handleEditAIConfig = (config: SettingsConfig) => {
    setEditingConfig(config);
    setAiConfigOpen(true);
  };

  const handleAddAIConfig = () => {
    setEditingConfig(null);
    setAiConfigOpen(true);
  };

  const handleEditEmailConfig = (config: SettingsConfig) => {
    setEditingEmailConfig(config);
    setEmailConfigOpen(true);
  };

  const handleAddEmailConfig = () => {
    setEditingEmailConfig(null);
    setEmailConfigOpen(true);
  };

  const handleSettingsAction = async (action?: SettingsAction) => {
    const result = await runBackupAction(action);
    if (result?.navigated && result.route) navigate(result.route);
  };

  const handleDeleteConfig = async (action: DeleteAction) => {
    if (!confirm(action.confirm_message)) return;

    try {
      await api.runAction(action);
      toast.success(action.success_message);
      refreshSettingsOverview();
    } catch (error: any) {
      console.error('Error deleting settings config:', error);
      toast.error(error.message || '删除失败');
    }
  };

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold text-foreground">系统设置</h1>
        <p className="text-muted-foreground mt-2">配置系统参数和功能选项</p>
      </div>

      <div className="grid gap-6">
        {/* 系统状态监控 - 最重要的卡片放在第一位 */}
        <SystemStatusCard />

        {/* 错误日志 - 开发者/管理员调试 */}
        <ErrorLogsPanel />

        <NavigationPreferencesCard />
        <DocumentParserSettingsCard />
        <NotificationChannelsCard />
        <CollectionAutomationCard />
        <InvoiceMailboxCard />

        <Card>
          <CardHeader>
            <div className="flex items-center gap-2">
              <FileSearch className="h-5 w-5 text-primary" />
              <CardTitle>数据完整性检查</CardTitle>
            </div>
            <CardDescription>检测所有模块的依赖关系并生成完善建议</CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <p className="text-sm text-muted-foreground">
                系统会检查所有基础模块的必要数据是否完整，并提供针对性的完善建议。
              </p>
            </div>
            <Button onClick={handleRunCheck} disabled={isChecking}>
              {isChecking ? "检查中..." : "开始检查"}
            </Button>
          </CardContent>
        </Card>
        {settingsSections.map((section) => {
          const SectionIcon = settingsSectionIcons[section.icon] || Database;
          return (
            <Card key={section.id}>
              <CardHeader>
                <div className="flex items-center gap-2">
                  <SectionIcon className="h-5 w-5 text-primary" />
                  <CardTitle>{section.title}</CardTitle>
                </div>
                <CardDescription>{section.description}</CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                {section.fields.length > 0 && (
                  <div className="grid gap-4 md:grid-cols-2">
                    {section.fields.map((field) => (
                      <div key={field.id} className="space-y-2">
                        <Label htmlFor={field.id}>{field.label}</Label>
                        <Input
                          id={field.id}
                          value={field.value}
                          readOnly={field.read_only}
                        />
                      </div>
                    ))}
                  </div>
                )}

                {section.items.length > 0 ? (
                  <div className="space-y-3">
                    {section.items.map((item) => (
                      <div
                        key={item.id}
                        className="flex items-center justify-between p-3 bg-muted rounded-lg"
                      >
                        <div>
                          <span className="font-medium">{item.label}</span>
                          {item.description && (
                            <p className="text-sm text-muted-foreground">
                              {item.description}
                            </p>
                          )}
                        </div>
                        {item.action && (
                          <Button
                            variant={item.action.variant || "outline"}
                            size="sm"
                            onClick={() => handleSettingsAction(item.action)}
                          >
                            {item.action.label}
                          </Button>
                        )}
                      </div>
                    ))}
                  </div>
                ) : (
                  section.empty_message && (
                    <p className="text-sm text-muted-foreground">
                      {section.empty_message}
                    </p>
                  )
                )}

                {section.toggles.map((toggle) => (
                  <div
                    key={toggle.id}
                    className="flex items-center justify-between"
                  >
                    <div className="space-y-0.5">
                      <Label>{toggle.label}</Label>
                      <p className="text-sm text-muted-foreground">
                        {toggle.description}
                      </p>
                    </div>
                    <Switch checked={toggle.checked} disabled={toggle.disabled} />
                  </div>
                ))}

                {section.summary_items.map((item) => (
                  <div key={item.id} className="space-y-2">
                    <Label>{item.label}</Label>
                    <p className="text-sm text-muted-foreground">{item.value}</p>
                  </div>
                ))}

                {section.actions.length > 0 && (
                  <div className="flex gap-2">
                    {section.actions.map((action) => (
                      <Button
                        key={action.label}
                        variant={action.variant || "default"}
                        disabled={backupBusy}
                        onClick={() => void handleSettingsAction(action)}
                      >
                        {action.label}
                      </Button>
                    ))}
                  </div>
                )}
              </CardContent>
            </Card>
          );
        })}

        <Card>
          <CardHeader>
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <Brain className="h-5 w-5 text-primary" />
                <div>
                  <CardTitle>AI 配置</CardTitle>
                  <CardDescription>管理 OpenAI 兼容的 API 配置</CardDescription>
                </div>
              </div>
              <Button
                onClick={handleAddAIConfig}
                size="sm"
                disabled={!settingsOverview.ai_configuration_create_action}
              >
                <Plus className="h-4 w-4 mr-2" />
                添加配置
              </Button>
            </div>
          </CardHeader>
          <CardContent>
            {aiConfigs && aiConfigs.length > 0 ? (
              <div className="space-y-3">
                {aiConfigs.map((config) => (
                  <div
                    key={config.id}
                    className="flex items-center justify-between p-4 bg-muted rounded-lg"
                  >
                    <div className="flex-1">
                      <div className="flex items-center gap-2">
                        <span className="font-medium">{config.display_name}</span>
                        <SettingsBadge badge={config.enabled_badge} />
                      </div>
                      <div className="text-sm text-muted-foreground mt-1 space-y-0.5">
                        <p>端点: {config.endpoint_label}</p>
                        <p className="text-xs">
                          模型: {config.model_label} | API Key: {config.api_key_label}
                        </p>
                      </div>
                    </div>
                    <div className="flex gap-2">
                      {config.update_action && (
                        <Button
                          variant="outline"
                          size="sm"
                          onClick={() => handleEditAIConfig(config)}
                        >
                          <Pencil className="h-4 w-4" />
                        </Button>
                      )}
                      {config.delete_action && (
                        <Button
                          variant="outline"
                          size="sm"
                          onClick={() => handleDeleteConfig(config.delete_action)}
                        >
                          <Trash2 className="h-4 w-4" />
                        </Button>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            ) : (
              <div className="text-center py-8 text-muted-foreground">
                <Brain className="h-12 w-12 mx-auto mb-3 opacity-50" />
                <p>{settingsOverview.empty_messages.ai_configurations}</p>
                <p className="text-sm mt-1">
                  {settingsOverview.empty_messages.ai_configurations_hint}
                </p>
              </div>
            )}
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <Mail className="h-5 w-5 text-primary" />
                <div>
                  <CardTitle>邮件配置</CardTitle>
                  <CardDescription>选择 W-Mail 或保留 SMTP 配置</CardDescription>
                </div>
              </div>
              <Button
                onClick={handleAddEmailConfig}
                size="sm"
                disabled={!settingsOverview.email_configuration_create_action}
              >
                <Plus className="h-4 w-4 mr-2" />
                添加配置
              </Button>
            </div>
          </CardHeader>
          <CardContent>
            {emailConfigs && emailConfigs.length > 0 ? (
              <div className="space-y-3">
                {emailConfigs.map((config) => (
                  <div
                    key={config.id}
                    className="flex items-center justify-between p-4 bg-muted rounded-lg"
                  >
                    <div className="flex-1">
                      <div className="flex items-center gap-2">
                        <span className="font-medium">{config.display_name}</span>
                        <SettingsBadge badge={config.enabled_badge} />
                      </div>
                      <div className="text-sm text-muted-foreground mt-1 space-y-0.5">
                        <p>发件人: {config.sender_label}</p>
                        <p className="text-xs">{config.provider === "w-mail" ? "W-Mail · 服务端受管凭据" : "SMTP · 当前未实现发送"}</p>
                      </div>
                    </div>
                    <div className="flex gap-2">
                      {config.update_action && (
                        <Button
                          variant="outline"
                          size="sm"
                          onClick={() => handleEditEmailConfig(config)}
                        >
                          <Pencil className="h-4 w-4" />
                        </Button>
                      )}
                      {config.delete_action && (
                        <Button
                          variant="outline"
                          size="sm"
                          onClick={() => handleDeleteConfig(config.delete_action)}
                        >
                          <Trash2 className="h-4 w-4" />
                        </Button>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            ) : (
              <div className="text-center py-8 text-muted-foreground">
                <Mail className="h-12 w-12 mx-auto mb-3 opacity-50" />
                <p>{settingsOverview.empty_messages.email_configurations}</p>
                <p className="text-sm mt-1">
                  {settingsOverview.empty_messages.email_configurations_hint}
                </p>
              </div>
            )}
          </CardContent>
        </Card>
      </div>

      {/* 数据完整性报告对话框 */}
      <Dialog open={reportOpen} onOpenChange={setReportOpen}>
        <DialogContent className="max-w-3xl max-h-[80vh] overflow-y-auto">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <FileSearch className="h-5 w-5" />
              {settingsOverview.report_title}
            </DialogTitle>
            <DialogDescription>
              {settingsOverview.report_description}
            </DialogDescription>
          </DialogHeader>

          <div className="space-y-6">
            {/* 总体完成度 */}
            <div className="space-y-3">
              <div className="flex items-center justify-between">
                <span className="text-sm font-medium">总体完成度</span>
                <span className="text-2xl font-bold text-primary">
                  {setupCompletion.rate_label}
                </span>
              </div>
              <Progress value={setupCompletion.rate} className="h-2" />
              <p className="text-sm text-muted-foreground">
                {setupCompletion.summary_label}
              </p>
            </div>

            {/* 检查结果详情 */}
            <div className="space-y-3">
              <h3 className="font-semibold">检查结果详情</h3>
              {checkResults.map((result, index) => (
                <Card key={index} className={result.card_class_name}>
                  <CardContent className="pt-4">
                    <div className="flex items-start gap-3">
                      <SetupCheckStatusIcon result={result} />
                      <div className="flex-1 space-y-2">
                        <div className="flex items-start justify-between">
                          <div>
                            <h4 className="font-medium">{result.module}</h4>
                            <p className="text-sm text-muted-foreground mt-1">
                              依赖项：{result.dependency}
                            </p>
                          </div>
                          <SettingsBadge badge={result.status_badge} />
                        </div>
                        <p className="text-sm text-muted-foreground">
                          {result.description}
                        </p>
                        {result.impact_label && (
                          <div className="text-sm">
                            <span className="text-muted-foreground">影响模块：</span>
                            <span className="ml-2 text-foreground">
                              {result.impact_label}
                            </span>
                          </div>
                        )}
                        {result.action_button.visible && (
                          <Button
                            size="sm"
                            variant="outline"
                            onClick={() => {
                              setReportOpen(false);
                              navigate(result.action_button.route);
                            }}
                          >
                            {result.action_button.label}
                          </Button>
                        )}
                      </div>
                    </div>
                  </CardContent>
                </Card>
              ))}
            </div>

            {/* 完善建议 */}
            {setupCompletion.needs_attention && (
              <Alert>
                <AlertCircle className="h-4 w-4" />
                <AlertDescription>
                  <strong>{setupCompletion.recommendation_title}</strong>
                  {setupCompletion.recommendation_intro}
                  <ol className="list-decimal list-inside mt-2 space-y-1">
                    {setupCompletion.recommendation_items.map((item) => (
                      <li key={item}>{item}</li>
                    ))}
                  </ol>
                </AlertDescription>
              </Alert>
            )}

            {setupCompletion.success_alert.visible && (
              <Alert className={setupCompletion.success_alert.class_name}>
                <SettingsAlertIcon
                  icon={setupCompletion.success_alert.icon}
                  className={setupCompletion.success_alert.icon_class_name}
                />
                <AlertDescription
                  className={setupCompletion.success_alert.description_class_name}
                >
                  <strong>{setupCompletion.success_alert.title}</strong>
                  {setupCompletion.success_alert.message}
                </AlertDescription>
              </Alert>
            )}
          </div>
        </DialogContent>
      </Dialog>

      {/* AI 配置对话框 */}
      <AIConfigDialog
        open={aiConfigOpen}
        onOpenChange={setAiConfigOpen}
        config={editingConfig}
        createAction={settingsOverview.ai_configuration_create_action}
        onSuccess={refreshSettingsOverview}
      />

      <EmailConfigDialog
        open={emailConfigOpen}
        onOpenChange={setEmailConfigOpen}
        config={editingEmailConfig}
        createAction={settingsOverview.email_configuration_create_action}
        wMailAvailable={settingsOverview.w_mail_available}
        onSuccess={refreshSettingsOverview}
      />

      <Dialog open={restoreOpen} onOpenChange={setRestoreOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>恢复数据</DialogTitle>
            <DialogDescription>
              恢复将覆盖当前所有数据，重启后生效。请选择备份文件并确认。
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4">
            {backupList.length === 0 ? (
              <p className="text-sm text-muted-foreground">暂无可用备份</p>
            ) : (
              <div className="space-y-2 max-h-64 overflow-y-auto">
                {backupList.map((item) => (
                  <label
                    key={item.file_name}
                    className="flex items-start gap-3 p-3 rounded-lg border cursor-pointer hover:bg-muted"
                  >
                    <input
                      type="radio"
                      name="restore-backup"
                      className="mt-1"
                      checked={selectedBackup === item.file_name}
                      onChange={() => setSelectedBackup(item.file_name)}
                    />
                    <div className="text-sm">
                      <div className="font-medium">{item.file_name}</div>
                      <div className="text-muted-foreground">
                        {item.created_at || "-"}
                        {typeof item.size_bytes === "number"
                          ? ` · ${(item.size_bytes / 1024).toFixed(1)} KB`
                          : ""}
                      </div>
                    </div>
                  </label>
                ))}
              </div>
            )}
            <div className="flex justify-end gap-2">
              <Button variant="outline" onClick={() => setRestoreOpen(false)} disabled={backupBusy}>
                取消
              </Button>
              <Button
                variant="destructive"
                onClick={() => void handleConfirmRestore()}
                disabled={backupBusy || !selectedBackup}
              >
                {backupBusy ? "处理中…" : "确认恢复"}
              </Button>
            </div>
          </div>
        </DialogContent>
      </Dialog>
    </div>
  );
};

export default Settings;
