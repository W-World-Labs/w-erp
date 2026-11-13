/**
 * 错误日志面板 - 开发者/系统管理员查看前端错误上报
 */
import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { api } from '@/integrations/api/client';
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import {
    AlertTriangle, Bug, Globe, Zap, Trash2, RefreshCw, ChevronDown, ChevronRight,
    Monitor, Clock,
} from 'lucide-react';
import { toast } from 'sonner';

interface DeleteAction {
    method: 'DELETE';
    endpoint: string;
    confirm_message: string;
    success_title?: string;
    success_message: string;
}

interface ErrorLog {
    id: string;
    error_type: string;
    error_message: string;
    stack_trace?: string;
    component_stack?: string;
    page_url?: string;
    page_path?: string;
    user_agent?: string;
    metadata?: string;
    metadata_fields?: Record<string, string>;
    created_at?: string;
    created_at_label?: string;
    error_type_label?: string;
    error_type_badge?: {
        label?: string;
        icon?: string;
        variant?: 'default' | 'secondary' | 'destructive' | 'outline';
    };
    delete_action?: DeleteAction | null;
}

interface ErrorLogsResponse {
    data: ErrorLog[];
    count: number;
    type_options: Array<{ value: string; label: string }>;
}

const ERROR_TYPE_ICONS = {
    AlertTriangle,
    Bug,
    Globe,
    Zap,
} as const;

export function ErrorLogsPanel() {
    const queryClient = useQueryClient();
    const [filter, setFilter] = useState<string>('all');
    const [expandedId, setExpandedId] = useState<string | null>(null);

    const { data, isLoading, refetch } = useQuery({
        queryKey: ['error-logs', filter],
        queryFn: async () => {
            const params = new URLSearchParams({
                order_by: 'created_at',
                order_dir: 'desc',
                limit: '100',
            });
            if (filter !== 'all') {
                params.set('error_type', filter);
            }
            const res = await api.get<ErrorLogsResponse>(`/api/v1/error_logs?${params}`);
            return res;
        },
    });

    const deleteMutation = useMutation({
        mutationFn: (action: DeleteAction) => api.runAction(action),
        onSuccess: (_data, action) => {
            queryClient.invalidateQueries({ queryKey: ['error-logs'] });
            toast.success(action.success_message);
        },
    });

    const logs = data?.data || [];
    const typeOptions = data?.type_options || [];

    const handleDelete = (action?: DeleteAction | null) => {
        if (!action || !confirm(action.confirm_message)) return;
        deleteMutation.mutate(action);
    };

    return (
        <Card>
            <CardHeader>
                <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2">
                        <Bug className="h-5 w-5 text-destructive" />
                        <div>
                            <CardTitle>错误日志</CardTitle>
                            <CardDescription>前端自动上报的错误 · 开发者调试专用</CardDescription>
                        </div>
                    </div>
                    <div className="flex items-center gap-2">
                        <Select value={filter} onValueChange={setFilter}>
                            <SelectTrigger className="w-[140px] h-8 text-xs">
                                <SelectValue />
                            </SelectTrigger>
                            <SelectContent>
                                {typeOptions.map((option) => (
                                    <SelectItem key={option.value} value={option.value}>
                                        {option.label}
                                    </SelectItem>
                                ))}
                            </SelectContent>
                        </Select>
                        <Button variant="ghost" size="sm" onClick={() => refetch()}>
                            <RefreshCw className="h-4 w-4" />
                        </Button>
                    </div>
                </div>
            </CardHeader>
            <CardContent>
                {isLoading ? (
                    <div className="text-center text-muted-foreground py-8">加载中...</div>
                ) : logs.length === 0 ? (
                    <div className="text-center text-muted-foreground py-8">
                        🎉 暂无错误日志，系统运行正常
                    </div>
                ) : (
                    <div className="border rounded-lg overflow-hidden">
                        <Table>
                            <TableHeader>
                                <TableRow>
                                    <TableHead className="w-[32px]"></TableHead>
                                    <TableHead className="w-[100px]">类型</TableHead>
                                    <TableHead>错误信息</TableHead>
                                    <TableHead className="w-[160px]">页面</TableHead>
                                    <TableHead className="w-[120px]">时间</TableHead>
                                    <TableHead className="w-[50px]"></TableHead>
                                </TableRow>
                            </TableHeader>
                            <TableBody>
                                {logs.map((log) => {
                                    const badge = log.error_type_badge;
                                    const ErrorIcon = ERROR_TYPE_ICONS[badge?.icon as keyof typeof ERROR_TYPE_ICONS] || Bug;
                                    const isExpanded = expandedId === log.id;
                                    const meta = log.metadata_fields;

                                    return (
                                        <>
                                            <TableRow
                                                key={log.id}
                                                className="cursor-pointer hover:bg-muted/50"
                                                onClick={() => setExpandedId(isExpanded ? null : log.id)}
                                            >
                                                <TableCell className="p-2">
                                                    {isExpanded
                                                        ? <ChevronDown className="h-4 w-4 text-muted-foreground" />
                                                        : <ChevronRight className="h-4 w-4 text-muted-foreground" />}
                                                </TableCell>
                                                <TableCell>
                                                    <Badge variant={badge?.variant || 'secondary'} className="gap-1 text-xs">
                                                        <ErrorIcon className="h-3.5 w-3.5" />
                                                        {badge?.label || log.error_type_label}
                                                    </Badge>
                                                </TableCell>
                                                <TableCell className="max-w-[300px]">
                                                    <p className="text-sm truncate font-mono">{log.error_message}</p>
                                                </TableCell>
                                                <TableCell>
                                                    <span className="text-xs text-muted-foreground font-mono truncate block max-w-[150px]">
                                                        {log.page_path}
                                                    </span>
                                                </TableCell>
                                                <TableCell>
                                                    <span className="text-xs text-muted-foreground flex items-center gap-1">
                                                        <Clock className="h-3 w-3" />
                                                        {log.created_at_label}
                                                    </span>
                                                </TableCell>
                                                <TableCell>
                                                    <Button
                                                        variant="ghost"
                                                        size="sm"
                                                        className="h-6 w-6 p-0 text-muted-foreground hover:text-destructive"
                                                        onClick={(e) => { e.stopPropagation(); handleDelete(log.delete_action); }}
                                                    >
                                                        <Trash2 className="h-3.5 w-3.5" />
                                                    </Button>
                                                </TableCell>
                                            </TableRow>
                                            {isExpanded && (
                                                <TableRow key={`${log.id}-detail`}>
                                                    <TableCell colSpan={6} className="bg-muted/30 p-4">
                                                        <div className="space-y-3">
                                                            {/* 完整错误信息 */}
                                                            <div>
                                                                <div className="text-xs font-semibold text-muted-foreground uppercase tracking-wide mb-1">完整错误</div>
                                                                <p className="text-sm font-mono bg-destructive/5 text-destructive p-2 rounded break-all">{log.error_message}</p>
                                                            </div>

                                                            {/* Metadata (API 错误的 endpoint/status) */}
                                                            {meta && (
                                                                <div>
                                                                    <div className="text-xs font-semibold text-muted-foreground uppercase tracking-wide mb-1">请求详情</div>
                                                                    <div className="grid grid-cols-3 gap-2 text-xs font-mono bg-muted p-2 rounded">
                                                                        {meta.method && <div><span className="text-muted-foreground">Method:</span> {String(meta.method)}</div>}
                                                                        {meta.endpoint && <div><span className="text-muted-foreground">Endpoint:</span> {String(meta.endpoint)}</div>}
                                                                        {meta.status && <div><span className="text-muted-foreground">Status:</span> {String(meta.status)}</div>}
                                                                    </div>
                                                                </div>
                                                            )}

                                                            {/* Stack Trace */}
                                                            {log.stack_trace && (
                                                                <div>
                                                                    <div className="text-xs font-semibold text-muted-foreground uppercase tracking-wide mb-1">Stack Trace</div>
                                                                    <pre className="text-xs font-mono p-2 bg-muted rounded whitespace-pre-wrap overflow-auto max-h-40">{log.stack_trace}</pre>
                                                                </div>
                                                            )}

                                                            {/* Component Stack */}
                                                            {log.component_stack && (
                                                                <div>
                                                                    <div className="text-xs font-semibold text-muted-foreground uppercase tracking-wide mb-1">Component Stack</div>
                                                                    <pre className="text-xs font-mono p-2 bg-muted rounded whitespace-pre-wrap overflow-auto max-h-32">{log.component_stack}</pre>
                                                                </div>
                                                            )}

                                                            {/* User Agent */}
                                                            {log.user_agent && (
                                                                <div className="flex items-center gap-1 text-xs text-muted-foreground">
                                                                    <Monitor className="h-3 w-3" />
                                                                    {log.user_agent}
                                                                </div>
                                                            )}
                                                        </div>
                                                    </TableCell>
                                                </TableRow>
                                            )}
                                        </>
                                    );
                                })}
                            </TableBody>
                        </Table>
                    </div>
                )}
            </CardContent>
        </Card>
    );
}
