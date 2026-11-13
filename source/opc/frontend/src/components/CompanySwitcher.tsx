import { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { companies, Company, setCurrentCompany, getCurrentCompanyId } from '@/integrations/api/client';
import { Building2, ChevronDown, Plus, Check, Settings } from 'lucide-react';
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuSeparator,
    DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import {
    Dialog,
    DialogContent,
    DialogHeader,
    DialogTitle,
    DialogTrigger,
} from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { toast } from 'sonner';

export function CompanySwitcher() {
    const queryClient = useQueryClient();
    const [isCreateOpen, setIsCreateOpen] = useState(false);
    const [newCompanyName, setNewCompanyName] = useState('');
    const [currentId, setCurrentId] = useState<string | null>(getCurrentCompanyId());

    // Fetch user's companies
    const { data: companiesData, isLoading } = useQuery({
        queryKey: ['companies'],
        queryFn: async () => {
            const result = await companies.list();
            return result.data;
        },
    });

    // Find current company
    const currentCompany = companiesData?.find(c => c.id === currentId) || companiesData?.[0];

    // Set initial company
    useEffect(() => {
        if (companiesData && companiesData.length > 0 && !currentId) {
            const currentCompanyFromBackend = companiesData.find(c => c.is_current);
            const defaultCompany = currentCompanyFromBackend || companiesData.find(c => c.is_default) || companiesData[0];
            setCurrentId(defaultCompany.id);
            setCurrentCompany(defaultCompany.id);
        }
    }, [companiesData, currentId]);

    // Switch company mutation
    const switchMutation = useMutation({
        mutationFn: async (companyId: string) => {
            await companies.switch(companyId);
            return companyId;
        },
        onSuccess: (companyId) => {
            setCurrentId(companyId);
            setCurrentCompany(companyId);
            queryClient.invalidateQueries();
            toast.success('已切换公司');
        },
        onError: () => {
            toast.error('切换失败');
        },
    });

    // Create company mutation
    const createMutation = useMutation({
        mutationFn: async (name: string) => {
            return companies.create({ name });
        },
        onSuccess: (result) => {
            queryClient.invalidateQueries({ queryKey: ['companies'] });
            setIsCreateOpen(false);
            setNewCompanyName('');
            toast.success(`公司 "${result.name}" 创建成功`);
            // Switch to new company
            switchMutation.mutate(result.id);
        },
        onError: () => {
            toast.error('创建失败');
        },
    });

    const handleSwitch = (company: Company) => {
        if (company.id !== currentId) {
            switchMutation.mutate(company.id);
        }
    };

    const handleCreate = () => {
        if (newCompanyName.trim()) {
            createMutation.mutate(newCompanyName.trim());
        }
    };

    const renderCompanyRoleBadge = (company: Company) => {
        const badge = company.role_badge;
        return (
            <Badge
                variant={badge?.variant || "outline"}
                className="ml-auto text-[10px] px-1.5"
            >
                {badge?.label || company.role}
            </Badge>
        );
    };

    if (isLoading) {
        return (
            <div className="flex items-center gap-2 px-3 py-2 rounded-lg bg-sidebar-accent/50">
                <Building2 className="h-5 w-5 animate-pulse" />
                <span className="text-sm animate-pulse">加载中...</span>
            </div>
        );
    }

    return (
        <div className="px-3 py-3 border-b border-sidebar-border">
            <DropdownMenu>
                <DropdownMenuTrigger asChild>
                    <Button
                        variant="ghost"
                        className="w-full justify-between px-3 py-3 h-auto hover:bg-sidebar-accent bg-sidebar-accent/30 text-inherit"
                    >
                        <div className="flex items-center gap-3 min-w-0">
                            <div className="flex h-9 w-9 items-center justify-center rounded-lg bg-blue-600 text-white shrink-0">
                                <Building2 className="h-5 w-5" />
                            </div>
                            <div className="text-left min-w-0 flex-1">
                                <p className="text-sm font-semibold truncate">
                                    {currentCompany?.name || '选择公司'}
                                </p>
                                <p className="text-xs opacity-60">
                                    {currentCompany?.role_badge?.label ||
                                        currentCompany?.role ||
                                        '成员'} · {companiesData?.length || 0} 个公司
                                </p>
                            </div>
                        </div>
                        <ChevronDown className="h-4 w-4 shrink-0 opacity-60" />
                    </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent align="start" className="w-64">
                    {companiesData?.map((company) => (
                        <DropdownMenuItem
                            key={company.id}
                            onClick={() => handleSwitch(company)}
                            className="flex items-center gap-2 cursor-pointer"
                        >
                            <div className="flex h-6 w-6 items-center justify-center rounded bg-muted">
                                <Building2 className="h-3 w-3" />
                            </div>
                            <span className="flex-1 truncate">{company.name}</span>
                            {company.id === currentId && (
                                <Check className="h-4 w-4 text-primary" />
                            )}
                            {renderCompanyRoleBadge(company)}
                        </DropdownMenuItem>
                    ))}

                    <DropdownMenuSeparator />

                    <Dialog open={isCreateOpen} onOpenChange={setIsCreateOpen}>
                        <DialogTrigger asChild>
                            <DropdownMenuItem
                                onSelect={(e) => e.preventDefault()}
                                className="gap-2 cursor-pointer"
                            >
                                <Plus className="h-4 w-4" />
                                <span>创建新公司</span>
                            </DropdownMenuItem>
                        </DialogTrigger>
                        <DialogContent>
                            <DialogHeader>
                                <DialogTitle>创建新公司</DialogTitle>
                            </DialogHeader>
                            <div className="space-y-4 pt-4">
                                <div className="space-y-2">
                                    <Label htmlFor="company-name">公司名称</Label>
                                    <Input
                                        id="company-name"
                                        placeholder="输入公司名称"
                                        value={newCompanyName}
                                        onChange={(e) => setNewCompanyName(e.target.value)}
                                        onKeyDown={(e) => e.key === 'Enter' && handleCreate()}
                                    />
                                </div>
                                <div className="flex justify-end gap-2">
                                    <Button variant="outline" onClick={() => setIsCreateOpen(false)}>
                                        取消
                                    </Button>
                                    <Button
                                        onClick={handleCreate}
                                        disabled={!newCompanyName.trim() || createMutation.isPending}
                                    >
                                        {createMutation.isPending ? '创建中...' : '创建'}
                                    </Button>
                                </div>
                            </div>
                        </DialogContent>
                    </Dialog>

                    <DropdownMenuItem className="gap-2 cursor-pointer" asChild>
                        <Link to="/settings">
                            <Settings className="h-4 w-4" />
                            <span>公司设置</span>
                        </Link>
                    </DropdownMenuItem>
                </DropdownMenuContent>
            </DropdownMenu>
        </div>
    );
}

export default CompanySwitcher;
