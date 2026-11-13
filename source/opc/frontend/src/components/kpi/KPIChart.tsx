import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, ReferenceLine } from "recharts";

interface KPIChartProps {
  config: {
    id: string;
    name: string;
    target_value?: number | null;
    target_value_label?: string | null;
    chart_empty_message: string;
    chart_points: Array<{
      value: number;
      value_label: string;
      period: string;
      period_label: string;
      recorded_at?: string | null;
    }>;
  };
}

export const KPIChart = ({ config }: KPIChartProps) => {
  const chartData = config.chart_points.map(point => ({
    period: point.period_label,
    value: point.value,
    valueLabel: point.value_label,
  }));

  return (
    <Card>
      <CardHeader className="pb-2">
        <CardTitle className="text-base font-medium">
          {config.name}趋势
        </CardTitle>
      </CardHeader>
      <CardContent>
        {chartData.length === 0 ? (
          <div className="h-48 flex items-center justify-center text-muted-foreground text-sm">
            {config.chart_empty_message}
          </div>
        ) : (
          <ResponsiveContainer width="100%" height={200}>
            <LineChart data={chartData}>
              <CartesianGrid strokeDasharray="3 3" className="stroke-muted" />
              <XAxis 
                dataKey="period" 
                className="text-xs"
                stroke="hsl(var(--muted-foreground))"
              />
              <YAxis 
                className="text-xs"
                stroke="hsl(var(--muted-foreground))"
              />
              <Tooltip
                formatter={(_, __, item) => [item.payload.valueLabel, config.name]}
                contentStyle={{
                  backgroundColor: 'hsl(var(--background))',
                  border: '1px solid hsl(var(--border))',
                  borderRadius: '8px',
                }}
              />
              {config.target_value && (
                <ReferenceLine
                  y={config.target_value}
                  stroke="hsl(var(--primary))"
                  strokeDasharray="3 3"
                  label={{ value: config.target_value_label || '目标', position: 'right' }}
                />
              )}
              <Line
                type="monotone"
                dataKey="value"
                stroke="hsl(var(--primary))"
                strokeWidth={2}
                dot={{ fill: 'hsl(var(--primary))' }}
              />
            </LineChart>
          </ResponsiveContainer>
        )}
      </CardContent>
    </Card>
  );
};
