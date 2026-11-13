/** Keep a contiguous suffix of complete messages within the chat API limits. */
export function boundedChatHistory<T extends { content: string }>(messages: T[]): T[] {
  const encoder = new TextEncoder();
  let bytes = 0;
  let start = messages.length;
  while (start > 0 && messages.length - start < 40) {
    const size = encoder.encode(messages[start - 1].content).byteLength;
    if (bytes + size > 64_000) break;
    bytes += size;
    start -= 1;
  }
  return messages.slice(start);
}
