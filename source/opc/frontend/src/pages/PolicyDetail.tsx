import { useParams, useNavigate } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import { api, type BackendAction } from "@/integrations/api/client";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
    ArrowLeft, ExternalLink, Globe2, Building2, Calendar, DollarSign,
    CheckCircle2, AlertCircle, Users, GraduationCap, Clock, Percent,
    Target, MapPin, Briefcase, FileText, Sparkles, type LucideIcon
} from "lucide-react";
import { toast } from "sonner";

type PolicyDetailRecord = {
    id: string;
    apply_action?: BackendAction | null;
    [key: string]: any;
};

const profileIcons: Record<string, LucideIcon> = {
    Globe2,
    Briefcase,
    Users,
    Clock,
    GraduationCap,
    DollarSign,
};

function ProfileIcon({ name }: { name?: string }) {
    const Icon = profileIcons[name || ""] || Target;
    return <Icon className="h-4 w-4 text-primary mt-0.5 flex-shrink-0" />;
}

const PolicyDetail = () => {
    const { id } = useParams<{ id: string }>();
    const navigate = useNavigate();

    const { data: policy, isLoading, error } = useQuery({
        queryKey: ["policy", id],
        queryFn: async () => {
            return api.get<PolicyDetailRecord>(`/api/v1/policies/${id}`);
        },
        enabled: !!id,
    });

    const handleApply = async () => {
        if (!policy) return;
        try {
            const action = policy.apply_action;
            if (!action) {
                throw new Error("后端未返回政策申报动作");
            }
            await api.runAction(action, action.default_values);
            toast.success(action.success_message || "已添加到申报追踪看板");
        } catch {
            toast.error("添加失败");
        }
    };

    if (isLoading) {
        return (
            <div className="flex items-center justify-center py-24">
                <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-primary" />
            </div>
        );
    }

    if (error || !policy) {
        return (
            <div className="flex flex-col items-center justify-center py-24 text-muted-foreground">
                <AlertCircle className="h-12 w-12 mb-4" />
                <h2 className="text-lg font-semibold mb-2">政策未找到</h2>
                <Button variant="outline" onClick={() => navigate("/policies")}>
                    <ArrowLeft className="h-4 w-4 mr-2" /> 返回政策大厅
                </Button>
            </div>
        );
    }

    return (
        <div className="space-y-6 max-w-4xl mx-auto">
            <Button
                variant="ghost"
                size="sm"
                className="gap-1 -ml-2 text-muted-foreground hover:text-foreground"
                onClick={() => navigate("/policies")}
            >
                <ArrowLeft className="h-4 w-4" />
                返回全球政策大厅
            </Button>

            <Card className="overflow-hidden border-0 shadow-lg">
                <div className="bg-gradient-to-r from-primary/10 via-primary/5 to-transparent p-6 pb-4">
                    <div className="flex flex-wrap gap-2 mb-3">
                        <Badge variant="outline" className={`text-xs font-medium ${policy.category_badge?.class_name || ""}`}>
                            {policy.category_badge?.label || policy.category}
                        </Badge>
                        <Badge variant="outline" className={policy.country_badge?.class_name}>
                            {policy.country_badge?.label || policy.country}
                        </Badge>
                        {policy.region && (
                            <Badge variant="outline" className="bg-slate-50 text-slate-700 border-slate-200">
                                <MapPin className="h-3 w-3 mr-1" />{policy.region}
                            </Badge>
                        )}
                    </div>
                    <h1 className="text-2xl font-bold mb-1">{policy.name}</h1>
                    {policy.name_en && (
                        <p className="text-sm text-muted-foreground">{policy.name_en}</p>
                    )}
                    {policy.provider && (
                        <div className="flex items-center gap-1.5 mt-3 text-sm text-muted-foreground">
                            <Building2 className="h-4 w-4" />
                            <span>实施机构: <strong className="text-foreground">{policy.provider}</strong></span>
                        </div>
                    )}
                </div>

                <div className="flex gap-3 px-6 py-3 bg-muted/30 border-t">
                    <Button
                        size="sm"
                        onClick={handleApply}
                        className="gap-1.5"
                        disabled={!policy.apply_action}
                    >
                        <Target className="h-3.5 w-3.5" />
                        加入追踪看板
                    </Button>
                    {policy.url && (
                        <Button size="sm" variant="outline" onClick={() => window.open(policy.url, "_blank")} className="gap-1.5">
                            <ExternalLink className="h-3.5 w-3.5" />
                            查阅官方原文
                        </Button>
                    )}
                </div>
            </Card>

            <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
                {policy.amount_range_label && (
                    <Card className="bg-emerald-50/50 border-emerald-200/50">
                        <CardContent className="p-4">
                            <div className="flex items-center gap-1.5 text-xs text-emerald-600 font-medium mb-1">
                                <DollarSign className="h-3.5 w-3.5" />资助金额
                            </div>
                            <div className="font-bold text-emerald-700">
                                {policy.amount_range_label}
                            </div>
                        </CardContent>
                    </Card>
                )}
                {policy.equity_required_label && (
                    <Card className="bg-orange-50/50 border-orange-200/50">
                        <CardContent className="p-4">
                            <div className="flex items-center gap-1.5 text-xs text-orange-600 font-medium mb-1">
                                <Percent className="h-3.5 w-3.5" />股权要求
                            </div>
                            <div className="font-bold text-orange-700">
                                {policy.equity_required_label}
                            </div>
                        </CardContent>
                    </Card>
                )}
                {policy.cycle && (
                    <Card className="bg-blue-50/50 border-blue-200/50">
                        <CardContent className="p-4">
                            <div className="flex items-center gap-1.5 text-xs text-blue-600 font-medium mb-1">
                                <Calendar className="h-3.5 w-3.5" />申报周期
                            </div>
                            <div className="font-semibold text-blue-700 text-sm">{policy.cycle}</div>
                        </CardContent>
                    </Card>
                )}
                {policy.deadline && (
                    <Card className="bg-red-50/50 border-red-200/50">
                        <CardContent className="p-4">
                            <div className="flex items-center gap-1.5 text-xs text-red-600 font-medium mb-1">
                                <Clock className="h-3.5 w-3.5" />截止日期
                            </div>
                            <div className="font-semibold text-red-700 text-sm">{policy.deadline}</div>
                        </CardContent>
                    </Card>
                )}
            </div>

            {policy.description && (
                <Card>
                    <CardHeader className="pb-3">
                        <CardTitle className="text-base flex items-center gap-2">
                            <FileText className="h-4 w-4 text-primary" />
                            政策详情
                        </CardTitle>
                    </CardHeader>
                    <CardContent>
                        <p className="text-sm leading-relaxed text-muted-foreground">{policy.description}</p>
                    </CardContent>
                </Card>
            )}

            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                {policy.eligibility_items?.length > 0 && (
                    <Card className="border-yellow-200/50">
                        <CardHeader className="pb-3">
                            <CardTitle className="text-base flex items-center gap-2">
                                <AlertCircle className="h-4 w-4 text-yellow-600" />
                                申请条件
                            </CardTitle>
                        </CardHeader>
                        <CardContent>
                            <ul className="space-y-2">
                                {policy.eligibility_items.map((item: any, index: number) => (
                                    <li key={`${item.text}:${index}`} className={`flex items-start gap-2 text-sm ${item.is_sub_item ? "ml-5" : ""}`}>
                                        {item.is_sub_item ? (
                                            <span className="text-muted-foreground mt-0.5">•</span>
                                        ) : (
                                            <span className="mt-0.5 flex-shrink-0 w-5 h-5 rounded-full bg-yellow-100 text-yellow-700 flex items-center justify-center text-[10px] font-bold">
                                                {index + 1}
                                            </span>
                                        )}
                                        <span className="text-foreground/80">{item.text}</span>
                                    </li>
                                ))}
                            </ul>
                        </CardContent>
                    </Card>
                )}

                {policy.benefit_items?.length > 0 && (
                    <Card className="border-green-200/50">
                        <CardHeader className="pb-3">
                            <CardTitle className="text-base flex items-center gap-2">
                                <Sparkles className="h-4 w-4 text-green-600" />
                                优惠内容
                            </CardTitle>
                        </CardHeader>
                        <CardContent>
                            <ul className="space-y-2">
                                {policy.benefit_items.map((item: any, index: number) => (
                                    <li key={`${item.text}:${index}`} className={`flex items-start gap-2 text-sm ${item.is_sub_item ? "ml-5" : ""}`}>
                                        {item.is_sub_item ? (
                                            <span className="text-muted-foreground mt-0.5">•</span>
                                        ) : (
                                            <CheckCircle2 className="h-4 w-4 text-green-500 mt-0.5 flex-shrink-0" />
                                        )}
                                        <span className="text-foreground/80">{item.text}</span>
                                    </li>
                                ))}
                            </ul>
                        </CardContent>
                    </Card>
                )}
            </div>

            {policy.match_profile_items?.length > 0 && (
                <Card>
                    <CardHeader className="pb-3">
                        <CardTitle className="text-base flex items-center gap-2">
                            <Target className="h-4 w-4 text-primary" />
                            匹配画像要素
                        </CardTitle>
                    </CardHeader>
                    <CardContent>
                        <div className="grid grid-cols-2 md:grid-cols-3 gap-4">
                            {policy.match_profile_items.map((item: any) => (
                                <div key={`${item.label}:${item.value}`} className="flex items-start gap-2 text-sm">
                                    <ProfileIcon name={item.icon} />
                                    <div>
                                        <p className="font-medium text-xs text-muted-foreground mb-0.5">{item.label}</p>
                                        <p>{item.value}</p>
                                    </div>
                                </div>
                            ))}
                        </div>
                    </CardContent>
                </Card>
            )}
        </div>
    );
};

export default PolicyDetail;
