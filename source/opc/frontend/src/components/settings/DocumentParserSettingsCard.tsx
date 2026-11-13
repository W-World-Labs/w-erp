import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Label } from "@/components/ui/label";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { FileSearch, Loader2 } from "lucide-react";
import { useDocumentParserSettings } from "@/hooks/useDocumentParserSettings";

export function DocumentParserSettingsCard() {
  const s = useDocumentParserSettings();

  if (s.loading) {
    return (
      <Card>
        <CardHeader>
          <CardTitle>文档识别 (OCR)</CardTitle>
        </CardHeader>
        <CardContent>加载中…</CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center gap-2">
          <FileSearch className="h-5 w-5 text-primary" />
          <div>
            <CardTitle>文档识别 (OCR)</CardTitle>
            <CardDescription>
              本地 Ollama 或 OpenAI 兼容视觉端点（vLLM / w-maas），填好即生效
            </CardDescription>
          </div>
        </div>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="space-y-2">
          <Label>后端</Label>
          <Select value={s.backend} onValueChange={s.setBackend}>
            <SelectTrigger>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {(s.config?.backends || []).map((b) => (
                <SelectItem key={b.id} value={b.id}>
                  {b.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
        {s.backend === "openai-vision" ? (
          <div className="grid gap-4 md:grid-cols-2">
            <div className="space-y-2 md:col-span-2">
              <Label>Base URL</Label>
              <Input
                value={s.baseUrl}
                onChange={(e) => s.setBaseUrl(e.target.value)}
                placeholder="http://host:8000/v1"
              />
            </div>
            <div className="space-y-2">
              <Label>
                API Key
                {s.config?.api_key_set ? `（当前 ${s.config.api_key_masked}）` : ""}
              </Label>
              <Input
                type="password"
                value={s.apiKey}
                onChange={(e) => s.setApiKey(e.target.value)}
                placeholder="留空保持原 Key"
              />
            </div>
            <div className="space-y-2">
              <Label>模型</Label>
              <Input
                value={s.model}
                onChange={(e) => s.setModel(e.target.value)}
                placeholder="mimo-v2-omni"
              />
            </div>
          </div>
        ) : (
          <div className="grid gap-4 md:grid-cols-2">
            <div className="space-y-2">
              <Label>Ollama URL</Label>
              <Input value={s.ollamaUrl} onChange={(e) => s.setOllamaUrl(e.target.value)} />
            </div>
            <div className="space-y-2">
              <Label>Ollama 模型</Label>
              <Input value={s.ollamaModel} onChange={(e) => s.setOllamaModel(e.target.value)} />
            </div>
          </div>
        )}
        <div className="flex gap-2 items-center">
          <Button onClick={() => void s.save()} disabled={s.saving}>
            {s.saving && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
            保存
          </Button>
          <Button variant="outline" onClick={() => void s.test()} disabled={s.testing}>
            {s.testing && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
            测试连接
          </Button>
          <span className="text-xs text-muted-foreground">
            {s.config?.enabled ? "已启用" : "未配置完整"}
          </span>
        </div>
        {s.testResult && (
          <p className={`text-sm ${s.testResult.ok ? "text-green-600" : "text-destructive"}`}>
            {s.testResult.ok
              ? `✅ ${s.testResult.latency_ms} ms · ${s.testResult.model}`
              : `❌ ${s.testResult.error}`}
          </p>
        )}
      </CardContent>
    </Card>
  );
}
