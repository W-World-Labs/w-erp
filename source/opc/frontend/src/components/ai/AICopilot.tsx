import { useState, useRef, useEffect, useCallback, useSyncExternalStore } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Card } from '@/components/ui/card';
import {
  Bot,
  X,
  Send,
  Sparkles,
  Loader2,
  Minimize2,
  Maximize2,
  MessageSquare,
  Trash2
} from 'lucide-react';
import { cn } from '@/lib/utils';
import { boundedChatHistory } from '@/lib/ai-chat-history';
import ReactMarkdown from 'react-markdown';
import { useLocation } from 'react-router-dom';
import { api, getCurrentCompanyId, type BackendAction } from '@/integrations/api/client';

import { useAuth } from '@/contexts/AuthContext';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';

interface ModelCatalog {
  models: { id: string }[];
  default_model: string;
}

function subscribeCompany(onChange: () => void) {
  window.addEventListener('erp-company-changed', onChange);
  return () => window.removeEventListener('erp-company-changed', onChange);
}

interface Message {
  role: 'user' | 'assistant';
  content: string;
}

interface QuickAction {
  label: string;
  prompt: string;
}

interface AiChatContext {
  chat_action: BackendAction | null;
  quick_actions: QuickAction[];
  empty_reply: string;
  error_prefix: string;
}

// 使用本地后端 AI 端点

export function AICopilot() {
  const { user } = useAuth();
  const companyId = useSyncExternalStore(subscribeCompany, getCurrentCompanyId);
  return <CompanyCopilot key={JSON.stringify([user?.id, companyId])} />;
}

function CompanyCopilot() {
  const [isOpen, setIsOpen] = useState(false);
  const [isMinimized, setIsMinimized] = useState(false);
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [model, setModel] = useState('');
  const [catalog, setCatalog] = useState<ModelCatalog | null>(null);
  const [catalogError, setCatalogError] = useState(false);
  const [catalogAttempt, setCatalogAttempt] = useState(0);
  const [chatContext, setChatContext] = useState<AiChatContext | null>(null);
  const scrollRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const location = useLocation();

  useEffect(() => {
    let active = true;
    api
      .get<AiChatContext>('/api/ai/chat-context')
      .then((context) => {
        if (active) {
          setChatContext(context);
        }
      })
      .catch((error) => {
        console.error('AI chat context error:', error);
      });
    return () => {
      active = false;
    };
  }, []);

  useEffect(() => {
    let active = true;
    setCatalog(null);
    setCatalogError(false);
    setModel('');
    api.get<ModelCatalog>('/api/ai/models').then((result) => {
      if (!result || !Array.isArray(result.models) || typeof result.default_model !== 'string'
        || result.models.some((entry) => !entry || typeof entry.id !== 'string' || !entry.id.trim())) {
        throw new Error('无效模型目录');
      }
      if (active) setCatalog(result);
    }).catch(() => {
      if (active) setCatalogError(true);
    });
    return () => { active = false; };
  }, [catalogAttempt]);

  // 滚动到底部
  const scrollToBottom = useCallback(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, []);

  useEffect(() => {
    scrollToBottom();
  }, [messages, scrollToBottom]);

  // AI 对话
  const streamChat = async (userMessage: string) => {
    const newMessages: Message[] = [...messages, { role: 'user', content: userMessage }];
    setMessages(newMessages);
    setInput('');
    setIsLoading(true);

    try {
      if (!chatContext?.chat_action) {
        throw new Error('后端未返回 AI 对话动作');
      }
      const response = await api.runAction<{ reply: string; success: boolean }>(chatContext.chat_action, {
        message: userMessage,
        model,
        path: location.pathname,
        history: boundedChatHistory(messages).map(m => ({ role: m.role, content: m.content })),
      });

      const reply = response?.reply || chatContext.empty_reply || '抱歉，无法获取回复。';
      setMessages(prev => [...prev, { role: 'assistant', content: reply }]);
    } catch (error) {
      console.error('Chat error:', error);
      const errorPrefix = chatContext?.error_prefix || '抱歉，遇到了一些问题';
      setMessages(prev => [
        ...prev.slice(0, -1),
        { role: 'assistant', content: `${errorPrefix}：${error instanceof Error ? error.message : '未知错误'}。请稍后再试。` }
      ]);
    } finally {
      setIsLoading(false);
    }
  };

  const handleSend = () => {
    if (!input.trim() || isLoading) return;
    streamChat(input.trim());
  };

  const handleQuickAction = (prompt: string) => {
    if (!isLoading) streamChat(prompt);
  };

  const clearChat = () => {
    setMessages([]);
  };

  if (!isOpen) {
    return (
      <Button
        aria-label="打开 AI 小助"
        onClick={() => setIsOpen(true)}
        className="fixed bottom-6 right-6 h-14 w-14 rounded-full shadow-lg bg-gradient-to-r from-primary to-primary/80 hover:from-primary/90 hover:to-primary/70 z-50"
        size="icon"
      >
        <Sparkles className="h-6 w-6" />
      </Button>
    );
  }

  return (
    <Card className={cn(
      "fixed z-50 shadow-2xl transition-all duration-300 border-primary/20",
      isMinimized
        ? "bottom-6 right-6 w-72 h-14"
        : "bottom-6 right-6 w-96 h-[600px] max-h-[80vh]"
    )}>
      {/* Header */}
      <div className="flex items-center justify-between p-3 border-b bg-gradient-to-r from-primary/10 to-primary/5 rounded-t-lg">
        <div className="flex items-center gap-2">
          <div className="h-8 w-8 rounded-full bg-gradient-to-r from-primary to-primary/80 flex items-center justify-center">
            <Bot className="h-4 w-4 text-primary-foreground" />
          </div>
          <div>
            <h3 className="font-semibold text-sm">AI 小助</h3>
            {!isMinimized && (
              <p className="text-xs text-muted-foreground">您的智能副驾驶</p>
            )}
          </div>
        </div>
        <div className="flex items-center gap-1">
          {!isMinimized && messages.length > 0 && (
            <Button variant="ghost" size="icon" className="h-7 w-7" onClick={clearChat} disabled={isLoading} aria-label="清空对话">
              <Trash2 className="h-3.5 w-3.5" />
            </Button>
          )}
          <Button
            variant="ghost"
            size="icon"
            className="h-7 w-7"
            onClick={() => setIsMinimized(!isMinimized)}
          >
            {isMinimized ? <Maximize2 className="h-3.5 w-3.5" /> : <Minimize2 className="h-3.5 w-3.5" />}
          </Button>
          <Button
            variant="ghost"
            size="icon"
            className="h-7 w-7"
            onClick={() => setIsOpen(false)}
          >
            <X className="h-3.5 w-3.5" />
          </Button>
        </div>
      </div>

      {!isMinimized && (
        <>
          <div className="px-3 pt-3 space-y-1">
            <Select value={model ? `model:${model}` : 'default'}
              onValueChange={(value) => setModel(value === 'default' ? '' : value.slice(6))}
              disabled={!catalog || isLoading}>
              <SelectTrigger aria-label="对话模型"><SelectValue /></SelectTrigger>
              <SelectContent>
                <SelectItem value="default">服务端默认{catalog?.default_model ? `（${catalog.default_model}）` : ''}</SelectItem>
                {catalog?.models.map(({ id }) => (
                  <SelectItem key={id} value={`model:${id}`}>{id}</SelectItem>
                ))}
              </SelectContent>
            </Select>
            {catalogError ? (
              <div role="alert" className="text-xs text-destructive">
                模型目录加载失败，将使用服务端默认。
                <Button variant="link" size="sm" onClick={() => setCatalogAttempt((n) => n + 1)}>重试模型目录</Button>
              </div>
            ) : !catalog ? (
              <p role="status" className="text-xs text-muted-foreground">正在加载模型目录，当前使用服务端默认。</p>
            ) : catalog.models.length === 0 ? (
              <p className="text-xs text-muted-foreground">目录暂无可选模型，使用服务端默认。</p>
            ) : null}
          </div>
          {/* Messages */}
          <ScrollArea className="flex-1 p-4 h-[calc(100%-220px)]" ref={scrollRef}>
            {messages.length === 0 ? (
              <div className="space-y-4">
                <div className="text-center py-6">
                  <div className="h-16 w-16 mx-auto mb-4 rounded-full bg-gradient-to-r from-primary/20 to-primary/10 flex items-center justify-center">
                    <MessageSquare className="h-8 w-8 text-primary" />
                  </div>
                  <h4 className="font-medium mb-2">你好！我是 AI 小助 👋</h4>
                  <p className="text-sm text-muted-foreground">
                    我可以帮你分析数据、解答问题、提供建议
                  </p>
                </div>

                <div className="space-y-2">
                  <p className="text-xs text-muted-foreground font-medium">快捷操作</p>
                  <div className="grid grid-cols-2 gap-2">
                    {(chatContext?.quick_actions || []).map((action, idx) => (
                      <Button
                        key={idx}
                        variant="outline"
                        size="sm"
                        className="h-auto py-2 px-3 text-xs justify-start"
                        onClick={() => handleQuickAction(action.prompt)}
                      >
                        {action.label}
                      </Button>
                    ))}
                  </div>
                </div>
              </div>
            ) : (
              <div className="space-y-4">
                {messages.map((msg, idx) => (
                  <div
                    key={idx}
                    className={cn(
                      "flex gap-2",
                      msg.role === 'user' ? "justify-end" : "justify-start"
                    )}
                  >
                    {msg.role === 'assistant' && (
                      <div className="h-7 w-7 rounded-full bg-gradient-to-r from-primary to-primary/80 flex items-center justify-center flex-shrink-0">
                        <Bot className="h-3.5 w-3.5 text-primary-foreground" />
                      </div>
                    )}
                    <div
                      className={cn(
                        "max-w-[80%] rounded-lg px-3 py-2 text-sm",
                        msg.role === 'user'
                          ? "bg-primary text-primary-foreground"
                          : "bg-muted"
                      )}
                    >
                      {msg.role === 'assistant' ? (
                        <div className="prose prose-sm dark:prose-invert max-w-none [&>p]:mb-2 [&>ul]:mb-2 [&>ol]:mb-2 [&>p:last-child]:mb-0">
                          <ReactMarkdown>
                            {msg.content || '...'}
                          </ReactMarkdown>
                        </div>
                      ) : (
                        msg.content
                      )}
                    </div>
                  </div>
                ))}
                {isLoading && messages[messages.length - 1]?.role === 'user' && (
                  <div className="flex gap-2 justify-start">
                    <div className="h-7 w-7 rounded-full bg-gradient-to-r from-primary to-primary/80 flex items-center justify-center flex-shrink-0">
                      <Bot className="h-3.5 w-3.5 text-primary-foreground" />
                    </div>
                    <div className="bg-muted rounded-lg px-3 py-2">
                      <Loader2 className="h-4 w-4 animate-spin" />
                    </div>
                  </div>
                )}
              </div>
            )}
          </ScrollArea>

          {/* Input */}
          <div className="p-3 border-t">
            <div className="flex gap-2">
              <Input
                ref={inputRef}
                value={input}
                onChange={(e) => setInput(e.target.value)}
                onKeyDown={(e) => e.key === 'Enter' && !e.shiftKey && handleSend()}
                placeholder="问我任何问题..."
                className="flex-1"
                disabled={isLoading}
              />
              <Button
                aria-label="发送消息"
                onClick={handleSend}
                disabled={!input.trim() || isLoading}
                size="icon"
              >
                {isLoading ? (
                  <Loader2 className="h-4 w-4 animate-spin" />
                ) : (
                  <Send className="h-4 w-4" />
                )}
              </Button>
            </div>
          </div>
        </>
      )}
    </Card>
  );
}
