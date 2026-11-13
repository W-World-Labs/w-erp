import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import { api, type BackendAction } from "@/integrations/api/client";
import { Card, CardContent, CardDescription, CardHeader, CardTitle, CardFooter } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
    Globe2, Building2, TrendingUp, Handshake, CheckCircle2, ChevronRight, Scale, Search, Clock, Users, Zap, ExternalLink, Lightbulb, DollarSign
} from "lucide-react";
import { Input } from "@/components/ui/input";
import {
    Select, SelectContent, SelectItem, SelectTrigger, SelectValue,
} from "@/components/ui/select";
import { toast } from "sonner";

interface PolicyDashboardOverview {
    policies: any[];
    policy_count: number;
    category_options: any[];
    country_options: any[];
    matched_policies: any[];
    match_count: number;
    applications: any[];
    applications_empty_message: string;
}

const PolicyDashboard = () => {
    const [activeTab, setActiveTab] = useState("match");
    const [searchQuery, setSearchQuery] = useState("");
    const [categoryFilter, setCategoryFilter] = useState("all");
    const [countryFilter, setCountryFilter] = useState("all");
    const navigate = useNavigate();

    const { data: overview, isLoading: isLoadingOverview, refetch: refetchOverview } = useQuery({
        queryKey: ["policy-dashboard-overview", categoryFilter, countryFilter, searchQuery],
        queryFn: async () => {
            const params = new URLSearchParams();
            if (categoryFilter !== "all") params.set("category", categoryFilter);
            if (countryFilter !== "all") params.set("country", countryFilter);
            if (searchQuery) params.set("search", searchQuery);
            const query = params.toString();
            return api.get<PolicyDashboardOverview>(
                `/api/v1/policies/dashboard/overview${query ? `?${query}` : ""}`
            );
        },
    });

    const displayedPolicies = overview?.policies || [];
    const matchedPolicies = overview?.matched_policies || [];
    const categoryOptions = overview?.category_options || [];
    const countryOptions = overview?.country_options || [];
    const applications = overview?.applications || [];
    const isLoadingPolicies = isLoadingOverview;
    const isLoadingMatches = isLoadingOverview;
    const isLoadingApps = isLoadingOverview;

    const handleApply = async (action?: BackendAction | null) => {
        if (!action) {
            toast.error("后端未返回政策申报动作");
            return;
        }
        try {
            await api.runAction(action, action.default_values);
            toast.success(action.success_message || "已添加到申报追踪列表");
            refetchOverview();
        } catch (error) {
            console.error(error);
            toast.error("添加申报失败");
        }
    };

    return (
        <div className="space-y-6">
            <div className="flex items-center justify-between">
                <div>
                    <h1 className="text-3xl font-bold flex items-center gap-2">
                        <Globe2 className="h-8 w-8 text-primary" />
                        全球政策智能大厅
                    </h1>
                    <p className="text-muted-foreground mt-2">
                        OPC Intelligence — 智能分析您的企业画像，一键匹配全球顶级创业津贴、税收优惠与顶级加速器
                    </p>
                </div>
            </div>

            <Tabs value={activeTab} onValueChange={setActiveTab} className="space-y-6">
                <TabsList className="grid w-full grid-cols-3 max-w-2xl bg-muted/60">
                    <TabsTrigger value="match" className="data-[state=active]:bg-primary data-[state=active]:text-primary-foreground">
                        <Zap className="mr-2 h-4 w-4" /> 智能雷达匹配
                    </TabsTrigger>
                    <TabsTrigger value="explore">
                        <Search className="mr-2 h-4 w-4" /> 全球前沿政策库
                    </TabsTrigger>
                    <TabsTrigger value="tracking">
                        <Scale className="mr-2 h-4 w-4" /> 申报进度追踪
                    </TabsTrigger>
                </TabsList>

                {/* 智能雷达匹配 */}
                <TabsContent value="match" className="space-y-6">
                    <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
                        <div className="md:col-span-1 space-y-6">
                            <Card className="bg-primary/5 border-primary/20">
                                <CardHeader>
                                    <CardTitle className="text-xl flex items-center gap-2">
                                        <Lightbulb className="h-5 w-5 text-yellow-500" />
                                        诊断与画像
                                    </CardTitle>
                                    <CardDescription>我们是如何为您推荐这些政策的？</CardDescription>
                                </CardHeader>
                                <CardContent className="space-y-4 text-sm">
                                    <div className="space-y-2">
                                        <p className="font-medium">引擎调用的数据源：</p>
                                        <ul className="list-disc pl-5 text-muted-foreground space-y-1">
                                            <li>企业注册地与经营地倾向</li>
                                            <li>员工规模、财务收入数据 (来自发票/账单)</li>
                                            <li>创始人画像 (高学历、外籍、海归)</li>
                                            <li>公司成立年限 (初创保护期)</li>
                                        </ul>
                                    </div>
                                    <Button variant="outline" className="w-full mt-4" onClick={() => window.location.href = '/company'}>
                                        去完善更多画像信息 <ChevronRight className="h-4 w-4 ml-2" />
                                    </Button>
                                </CardContent>
                            </Card>
                        </div>

                        <div className="md:col-span-2 space-y-4">
                            {isLoadingMatches ? (
                                <div className="py-12 flex flex-col items-center justify-center">
                                    <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-primary mb-4"></div>
                                    <p className="text-muted-foreground">正在调用 OPC AI 引擎为您进行深度画像匹配...</p>
                                </div>
                            ) : matchedPolicies && matchedPolicies.length > 0 ? (
                                matchedPolicies.map((match: any, index: number) => {
                                    const p = match.policy;
                                    return (
                                        <Card key={p.id || index} className={match.card_class_name || ""}>
                                            {match.rank_badge?.visible && (
                                                <div className={`absolute top-0 right-0 px-3 py-1 text-xs font-bold rounded-bl-lg z-10 ${match.rank_badge.class_name}`}>
                                                    {match.rank_badge.label}
                                                </div>
                                            )}
                                            <CardHeader className="pb-2">
                                                <div className="flex justify-between items-start">
                                                    <div>
                                                        <div className="flex items-center gap-2 mb-1">
                                                            <Badge variant="outline" className="bg-blue-50 text-blue-700 border-blue-200">
                                                                {p.country_badge?.label || "—"}
                                                            </Badge>
                                                            <Badge variant="outline" className={p.category_badge?.class_name}>
                                                                {p.category_badge?.label || p.category}
                                                            </Badge>
                                                        </div>
                                                        <CardTitle className="text-xl">{p.name}</CardTitle>
                                                        {p.name_en && <CardDescription>{p.name_en}</CardDescription>}
                                                    </div>
                                                    <div className="text-right">
                                                        <div className="text-2xl font-bold text-green-600">
                                                            {match.match_score_label}
                                                        </div>
                                                        <div className="text-xs text-muted-foreground mt-1">匹配指数</div>
                                                    </div>
                                                </div>
                                            </CardHeader>
                                            <CardContent>
                                                <div className="mt-2 mb-4">
                                                    <p className="text-sm text-muted-foreground">{p.description}</p>
                                                </div>

                                                <div className="grid grid-cols-2 gap-4 mt-4 bg-muted/50 p-3 rounded-lg text-sm">
                                                    {p.amount_max && (
                                                        <div>
                                                            <span className="text-muted-foreground block mb-1">资助金额</span>
                                                            <span className="font-semibold">{p.amount_max_label}</span>
                                                        </div>
                                                    )}
                                                    {p.provider && (
                                                        <div>
                                                            <span className="text-muted-foreground block mb-1">实施机构</span>
                                                            <span className="font-medium">{p.provider}</span>
                                                        </div>
                                                    )}
                                                </div>

                                                <div className="mt-4">
                                                    <p className="text-sm font-medium mb-2 flex items-center gap-1">
                                                        <CheckCircle2 className="h-4 w-4 text-green-500" />
                                                        满足的特征项
                                                    </p>
                                                    <div className="flex flex-wrap gap-2">
                                                        {match.match_reasons?.map((reason: string, i: number) => (
                                                            <Badge key={i} variant="secondary" className="bg-green-50 text-green-700 hover:bg-green-100">
                                                                {reason}
                                                            </Badge>
                                                        ))}
                                                    </div>
                                                </div>
                                            </CardContent>
                                            <CardFooter className="pt-2 border-t mt-2 flex justify-end gap-2">
                                                {p.url && (
                                                    <Button variant="outline" size="sm" onClick={() => window.open(p.url, "_blank")}>
                                                        查阅原文 <ExternalLink className="ml-2 h-3 w-3" />
                                                    </Button>
                                                )}
                                                <Button
                                                    size="sm"
                                                    onClick={() => handleApply(p.apply_action)}
                                                    disabled={!p.apply_action}
                                                >
                                                    一键加入追踪看板 <ChevronRight className="ml-2 h-4 w-4" />
                                                </Button>
                                                <Button size="sm" variant="outline" onClick={() => navigate(`/policies/${p.id}`)}>
                                                    查看详情
                                                </Button>
                                            </CardFooter>
                                        </Card>
                                    );
                                })
                            ) : (
                                <div className="text-center py-12 border rounded-lg bg-card text-muted-foreground">
                                    暂未匹配到满足核心条件的政策。随着规模的增长，系统会持续为您推荐。
                                </div>
                            )}
                        </div>
                    </div>
                </TabsContent>

                {/* Explore All */}
                <TabsContent value="explore" className="space-y-4">
                    <div className="flex gap-4 mb-6">
                        <div className="flex-1 relative">
                            <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
                            <Input
                                placeholder="搜索 YC、国高新、科技券..."
                                className="pl-9"
                                value={searchQuery}
                                onChange={(e) => setSearchQuery(e.target.value)}
                            />
                        </div>
                        <Select value={countryFilter} onValueChange={setCountryFilter}>
                            <SelectTrigger className="w-[180px]">
                                <SelectValue placeholder="筛选地区" />
                            </SelectTrigger>
                            <SelectContent>
                                {countryOptions.map((option: any) => (
                                    <SelectItem key={option.value} value={option.value}>
                                        {option.label}
                                    </SelectItem>
                                ))}
                            </SelectContent>
                        </Select>
                        <Select value={categoryFilter} onValueChange={setCategoryFilter}>
                            <SelectTrigger className="w-[180px]">
                                <SelectValue placeholder="政策类型" />
                            </SelectTrigger>
                            <SelectContent>
                                {categoryOptions.map((option: any) => (
                                    <SelectItem key={option.value} value={option.value}>
                                        {option.label}
                                    </SelectItem>
                                ))}
                            </SelectContent>
                        </Select>
                    </div>

                    <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4">
                        {isLoadingPolicies ? (
                            <div className="col-span-12 text-center py-8">加载中...</div>
                        ) : displayedPolicies.length > 0 ? (
                            displayedPolicies.map((p: any) => (
                                <Card key={p.id} className="flex flex-col h-full hover:shadow-md transition-shadow">
                                    <CardHeader className="pb-2 flex-grow">
                                        <div className="flex justify-between items-start mb-2">
                                            <Badge variant="secondary">{p.country_badge?.label || "—"}</Badge>
                                            <Badge variant="outline" className={`text-xs ${p.category_badge?.class_name || ""}`}>
                                                {p.category_badge?.label || p.category}
                                            </Badge>
                                        </div>
                                        <CardTitle className="text-lg leading-tight">{p.name}</CardTitle>
                                        {p.name_en && <CardDescription>{p.name_en}</CardDescription>}
                                        <p className="text-sm text-muted-foreground mt-2 line-clamp-3">{p.description}</p>
                                    </CardHeader>
                                    <CardContent className="pb-4">
                                        {p.amount_max && (
                                            <div className="text-sm font-semibold text-emerald-700 bg-emerald-50 px-2 py-1 flex items-center gap-1 rounded inline-flex">
                                                <DollarSign className="h-3 w-3" /> 最高可获 {p.amount_max_label}
                                            </div>
                                        )}
                                    </CardContent>
                                    <CardFooter className="pt-0 border-t flex justify-between items-center bg-muted/20 p-4">
                                        <span className="text-xs text-muted-foreground">{p.provider || "未知机构"}</span>
                                        <Button size="sm" variant="ghost" onClick={() => navigate(`/policies/${p.id}`)}>
                                            查看详情
                                        </Button>
                                        <Button
                                            size="sm"
                                            variant="ghost"
                                            onClick={() => handleApply(p.apply_action)}
                                            disabled={!p.apply_action}
                                        >
                                            加入追踪
                                        </Button>
                                    </CardFooter>
                                </Card>
                            ))
                        ) : (
                            <div className="col-span-12 text-center py-12 text-muted-foreground border rounded-lg bg-card">
                                没有找到符合筛选条件的政策，请调整检索词。
                            </div>
                        )}
                    </div>
                </TabsContent>

                {/* Tracking Tab */}
                <TabsContent value="tracking" className="space-y-6">
                    <Card>
                        <CardHeader>
                            <CardTitle>我的申报看板</CardTitle>
                            <CardDescription>追踪企业政策申报全生命周期</CardDescription>
                        </CardHeader>
                        <CardContent>
                            {isLoadingApps ? (
                                <p className="py-8 text-center text-muted-foreground">加载中...</p>
                            ) : applications && applications.length > 0 ? (
                                <div className="space-y-4">
                                    {applications.map((app: any) => (
                                        <div key={app.id} className="flex justify-between items-center p-4 border rounded-lg hover:border-primary/30 bg-card transition-colors">
                                            <div>
                                                <div className="flex items-center gap-2 mb-1">
                                                    <div className={`text-xs px-2 py-0.5 rounded-full font-medium ${app.status_badge?.class_name || 'bg-gray-100'}`}>
                                                        {app.status_badge?.label || app.status}
                                                    </div>
                                                    <span className="text-xs text-muted-foreground flex items-center gap-1"><Clock className="w-3 h-3" /> 更新于 {app.updated_at_label}</span>
                                                </div>
                                                <h4 className="font-semibold text-lg">{app.policy_name || "未知政策"}</h4>
                                                <p className="text-sm text-muted-foreground mt-1">{app.notes || "暂无备注"}</p>
                                            </div>
                                            <div>
                                                <Button variant="outline" size="sm" onClick={() => window.location.href = '/projects'}>
                                                    去项目工作台推进 <ChevronRight className="ml-1 w-4 h-4" />
                                                </Button>
                                            </div>
                                        </div>
                                    ))}
                                </div>
                            ) : (
                                <div className="text-center py-12">
                                    <Handshake className="h-12 w-12 text-muted-foreground mx-auto mb-4" />
                                    <h3 className="text-lg font-semibold mb-2">追踪看板还是空的</h3>
                                    <p className="text-muted-foreground mb-6">{overview?.applications_empty_message || "切换到“智能雷达匹配”看看哪些高额补贴适合您吧！"}</p>
                                    <Button onClick={() => setActiveTab("match")}>
                                        <Zap className="mr-2 h-4 w-4" /> 去做雷达匹配
                                    </Button>
                                </div>
                            )}
                        </CardContent>
                    </Card>
                </TabsContent>
            </Tabs>
        </div>
    );
};

export default PolicyDashboard;
