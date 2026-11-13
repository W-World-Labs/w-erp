import { describe, expect, it } from 'vitest';
import { boundedChatHistory } from './ai-chat-history';

describe('chat history request limits', () => {
  it('retains the most recent 40 complete messages without changing displayed history', () => {
    const messages = Array.from({ length: 46 }, (_, index) => ({ content: `turn ${index}` }));
    expect(boundedChatHistory(messages)).toEqual(messages.slice(6));
    expect(messages).toHaveLength(46);
  });

  it('counts UTF-8 bytes and includes the exact byte boundary without splitting unicode', () => {
    const messages = [{ content: 'old' }, { content: '😀'.repeat(15_000) }, { content: '中'.repeat(1000) + 'a'.repeat(1000) }];
    const history = boundedChatHistory(messages);
    expect(history).toEqual(messages.slice(1));
    expect(history.reduce((sum, m) => sum + new TextEncoder().encode(m.content).length, 0)).toBe(64_000);
  });

  it('omits an oversized message and everything before it, keeping recent whole messages', () => {
    const messages = [{ content: 'old' }, { content: '中'.repeat(22_000) }, { content: 'recent' }];
    expect(boundedChatHistory(messages)).toEqual([{ content: 'recent' }]);
    expect(boundedChatHistory(messages.slice(0, 2))).toEqual([]);
  });
});
