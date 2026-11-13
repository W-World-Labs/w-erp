import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { globalOpcHubs, OpcHub, OpcCalculatorResult } from "@/integrations/api/client";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
    Globe2, TrendingUp, DollarSign, Building2, Shield, Plane, Calculator, MapPin,
    type LucideIcon,
} from "lucide-react";
import { toast } from "sonner";

type DisplayBadge = {
    label: string;
    variant?: "default" | "secondary" | "destructive" | "outline";
    icon?: string;
    class_name?: string;
};

const iconMap: Record<string, LucideIcon> = {
    Plane,
};

const BackendBadge = ({ badge }: { badge?: DisplayBadge | null }) => {
    if (!badge) return null;
    const Icon = badge.icon ? iconMap[badge.icon] : null;
    return (
        <Badge variant={badge.variant || "outline"} className={badge.class_name || "text-xs"}>
            {Icon && <Icon className="h-3 w-3 mr-1" />}
            {badge.label}
        </Badge>
    );
};

const GlobalOpcHub = () => {
    const [revenue, setRevenue] = useState<number>(60000);
    const [colIndex, setColIndex] = useState<number>(50);
    const [calcResults, setCalcResults] = useState<OpcCalculatorResult[] | null>(null);
    const [isCalculating, setIsCalculating] = useState(false);

    const { data: hubs, isLoading } = useQuery({
        queryKey: ["global-opc-hubs"],
        queryFn: async () => {
            const res = await globalOpcHubs.list();
            return res?.data || [];
        },
    });

    const handleCalculate = async () => {
        setIsCalculating(true);
        try {
            const results = await globalOpcHubs.calculate(revenue, colIndex);
            setCalcResults(results);
            toast.success("税务沙盘推演完毕");
        } catch (e: any) {
            toast.error("计算失败: " + e.message);
        } finally {
            setIsCalculating(false);
        }
    };
    const bestResult = calcResults?.find((r) => r.is_best);

    return (
        <div className="space-y-6">
            {/* Header */}
            <div className="flex items-center justify-between">
                <div>
                    <h1 className="text-2xl font-bold flex items-center gap-2">
                        <Globe2 className="h-7 w-7 text-blue-500" />
                        全球 OPC 出海沙盘
                    </h1>
                    <p className="text-muted-foreground mt-1">
                        对比 5 大单人公司圣地的税制、生活成本与创始人实际到手利润
                    </p>
                </div>
            </div>

            {/* Hub Info Cards */}
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-5 gap-4">
                {isLoading ? (
                    Array.from({ length: 5 }).map((_, i) => (
                        <Card key={i} className="animate-pulse">
                            <CardHeader className="pb-2"><div className="h-6 bg-muted rounded w-3/4" /></CardHeader>
                            <CardContent><div className="h-20 bg-muted rounded" /></CardContent>
                        </Card>
                    ))
                ) : (
                    hubs?.map((hub: OpcHub) => (
                        <Card key={hub.id} className="hover:shadow-md transition-shadow">
                            <CardHeader className="pb-2">
                                <CardTitle className="text-lg flex items-center gap-2">
                                    <span className="text-2xl">{hub.flag}</span>
                                    {hub.city}
                                </CardTitle>
                                <CardDescription className="text-xs">{hub.country_code} · {hub.currency}</CardDescription>
                            </CardHeader>
                            <CardContent className="space-y-2 text-sm">
                                <div className="flex justify-between">
                                    <span className="text-muted-foreground">企业税</span>
                                    <span className="font-mono">{hub.corporate_tax_rate_label}</span>
                                </div>
                                <div className="flex justify-between">
                                    <span className="text-muted-foreground">分红税</span>
                                    <span className="font-mono">{hub.dividend_tax_rate_label}</span>
                                </div>
                                <div className="flex justify-between">
                                    <span className="text-muted-foreground">房租/月</span>
                                    <span className="font-mono">{hub.rent_label}</span>
                                </div>
                                <div className="flex flex-wrap gap-1 mt-2">
                                    {hub.feature_badges?.map((badge) => (
                                        <BackendBadge key={badge.label} badge={badge} />
                                    ))}
                                </div>
                            </CardContent>
                        </Card>
                    ))
                )}
            </div>

            {/* Calculator */}
            <Card>
                <CardHeader>
                    <CardTitle className="flex items-center gap-2">
                        <Calculator className="h-5 w-5 text-orange-500" />
                        OPC 税务沙盘推演
                    </CardTitle>
                    <CardDescription>
                        输入你的预估年收入和当前所在地生活成本指数，一键对比各地创始人到手利润
                    </CardDescription>
                </CardHeader>
                <CardContent>
                    <div className="flex flex-col sm:flex-row gap-4 items-end">
                        <div className="flex-1">
                            <Label>年预估收入 (USD)</Label>
                            <Input
                                type="number"
                                value={revenue}
                                onChange={(e) => setRevenue(Number(e.target.value))}
                                placeholder="60000"
                            />
                        </div>
                        <div className="flex-1">
                            <Label>当前所在地 COL 指数 (NYC=100)</Label>
                            <Input
                                type="number"
                                value={colIndex}
                                onChange={(e) => setColIndex(Number(e.target.value))}
                                placeholder="50"
                            />
                        </div>
                        <Button onClick={handleCalculate} disabled={isCalculating} className="shrink-0">
                            {isCalculating ? "推演中..." : "🧮 开始推演"}
                        </Button>
                    </div>
                </CardContent>
            </Card>

            {/* Results */}
            {calcResults && (
                <div className="space-y-4">
                    {/* Best pick highlight */}
                    {bestResult && (
                        <Card className="border-green-500/50 bg-green-50/50 dark:bg-green-900/10">
                            <CardContent className="pt-4">
                                <div className="flex items-center gap-3">
                                    <div className="text-3xl">🏆</div>
                                    <div>
                                        <p className="text-sm text-muted-foreground">按到手利润排名第一</p>
                                        <p className="text-xl font-bold text-green-700 dark:text-green-400">
                                            {bestResult.city} — 年净到手 {bestResult.net_take_home_label}
                                            <span className="text-sm ml-2 font-normal">({bestResult.take_home_percentage_label})</span>
                                        </p>
                                    </div>
                                </div>
                            </CardContent>
                        </Card>
                    )}

                    {/* Detailed comparison cards */}
                    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                        {calcResults.map((r) => (
                                    <Card key={r.hub_id} className="hover:shadow-lg transition-shadow">
                                        <CardHeader className="pb-3">
                                            <div className="flex items-center justify-between">
                                                <CardTitle className="flex items-center gap-2">
                                                    <MapPin className="h-4 w-4" /> {r.city}
                                                </CardTitle>
                                                <Badge className={r.tax_type_badge?.class_name || "bg-gray-100 text-gray-800"}>
                                                    {r.tax_type_badge?.label || r.tax_sandbox_type}
                                                </Badge>
                                            </div>
                                        </CardHeader>
                                        <CardContent className="space-y-3">
                                            <div className="grid grid-cols-2 gap-3 text-sm">
                                                <div className="space-y-1">
                                                    <span className="text-muted-foreground flex items-center gap-1">
                                                        <DollarSign className="h-3 w-3" /> 年净到手
                                                    </span>
                                                    <p className="text-lg font-bold text-green-600 dark:text-green-400">{r.net_take_home_label}</p>
                                                </div>
                                                <div className="space-y-1">
                                                    <span className="text-muted-foreground flex items-center gap-1">
                                                        <TrendingUp className="h-3 w-3" /> 到手率
                                                    </span>
                                                    <p className="text-lg font-bold">{r.take_home_percentage_label}</p>
                                                </div>
                                                <div className="space-y-1">
                                                    <span className="text-muted-foreground flex items-center gap-1">
                                                        <Shield className="h-3 w-3" /> MRR 生存线
                                                    </span>
                                                    <p className="font-mono">{r.mrr_survival_line_label}</p>
                                                </div>
                                                <div className="space-y-1">
                                                    <span className="text-muted-foreground flex items-center gap-1">
                                                        <Building2 className="h-3 w-3" /> 摩擦成本
                                                    </span>
                                                    <p className="font-mono">{r.friction_costs_label}</p>
                                                </div>
                                            </div>
                                            <div className="border-t pt-2 text-xs text-muted-foreground space-y-1">
                                                <div className="flex justify-between">
                                                    <span>生活开支</span><span>{r.living_costs_label}</span>
                                                </div>
                                                <div className="flex justify-between">
                                                    <span>税费合计</span><span className="text-red-500">{r.total_tax_paid_label}</span>
                                                </div>
                                                <div className="flex justify-between">
                                                    <span>综合开销占比</span><span>{r.setup_and_maintenance_roi_label}</span>
                                                </div>
                                            </div>
                                        </CardContent>
                                    </Card>
                        ))}
                    </div>
                </div>
            )}
        </div>
    );
};

export default GlobalOpcHub;
