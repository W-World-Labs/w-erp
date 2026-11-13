import { useCallback, useEffect, useState } from "react";
import { api } from "@/integrations/api/client";
import { toast } from "sonner";

export type DocumentParserCfg = {
  backend: string;
  base_url: string;
  api_key_masked: string;
  api_key_set: boolean;
  model: string;
  ollama_url: string;
  ollama_model: string;
  enabled: boolean;
  backends: { id: string; label: string; description: string }[];
};

export type DocumentParserTestResult = {
  ok: boolean;
  latency_ms: number;
  model?: string;
  error?: string;
};

/** Backend-owned document-parser settings (mutations live here, not in components). */
export function useDocumentParserSettings() {
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [testing, setTesting] = useState(false);
  const [config, setConfig] = useState<DocumentParserCfg | null>(null);
  const [backend, setBackend] = useState("ollama-vision");
  const [baseUrl, setBaseUrl] = useState("");
  const [apiKey, setApiKey] = useState("");
  const [model, setModel] = useState("");
  const [ollamaUrl, setOllamaUrl] = useState("");
  const [ollamaModel, setOllamaModel] = useState("");
  const [testResult, setTestResult] = useState<DocumentParserTestResult | null>(null);

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const data = await api.get<DocumentParserCfg>("/api/v1/settings/document-parser");
      setConfig(data);
      setBackend(data.backend || "ollama-vision");
      setBaseUrl(data.base_url || "");
      setModel(data.model || "");
      setOllamaUrl(data.ollama_url || "");
      setOllamaModel(data.ollama_model || "");
      setApiKey("");
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : "加载失败";
      toast.error(message);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void load();
  }, [load]);

  const save = async () => {
    setSaving(true);
    try {
      const body: Record<string, unknown> = {
        backend,
        base_url: baseUrl,
        model,
        ollama_url: ollamaUrl,
        ollama_model: ollamaModel,
      };
      if (apiKey.trim()) body.api_key = apiKey.trim();
      else body.keep_api_key = true;
      const data = await api.put<DocumentParserCfg>("/api/v1/settings/document-parser", body);
      setConfig(data);
      setApiKey("");
      toast.success("文档识别配置已保存");
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : "保存失败";
      toast.error(message);
    } finally {
      setSaving(false);
    }
  };

  const test = async () => {
    setTesting(true);
    setTestResult(null);
    try {
      const result = await api.post<DocumentParserTestResult>(
        "/api/v1/settings/document-parser/test",
        {},
      );
      setTestResult(result);
      if (result.ok) toast.success(`连接成功（${result.latency_ms} ms）`);
      else toast.error(result.error || "测试失败");
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : "测试失败";
      toast.error(message);
    } finally {
      setTesting(false);
    }
  };

  return {
    loading,
    saving,
    testing,
    config,
    backend,
    setBackend,
    baseUrl,
    setBaseUrl,
    apiKey,
    setApiKey,
    model,
    setModel,
    ollamaUrl,
    setOllamaUrl,
    ollamaModel,
    setOllamaModel,
    testResult,
    save,
    test,
    reload: load,
  };
}
