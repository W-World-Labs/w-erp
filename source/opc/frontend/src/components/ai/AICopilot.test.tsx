import { act, fireEvent, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, expect, it, vi } from 'vitest';
import { AICopilot } from './AICopilot';

const mocks = vi.hoisted(() => ({ get: vi.fn(), runAction: vi.fn(), company: 'A' }));
vi.mock('@/integrations/api/client', () => ({
  api: { get: mocks.get, runAction: mocks.runAction }, getCurrentCompanyId: () => mocks.company,
}));
vi.mock('@/contexts/AuthContext', () => ({ useAuth: () => ({ user: { id: 'user' } }) }));
vi.mock('react-router-dom', () => ({ useLocation: () => ({ pathname: '/dashboard' }) }));
const context = { chat_action: { path: '/api/ai/chat', method: 'POST' }, quick_actions: [], empty_reply: '', error_prefix: '' };
const catalog = { models: [{ id: 'maas/model-a' }, { id: 'maas/model-b' }], default_model: 'maas/default' };

beforeEach(() => {
  vi.clearAllMocks();
  mocks.company = 'A';
  mocks.get.mockImplementation(async (path) => path.endsWith('chat-context') ? context : catalog);
  mocks.runAction.mockResolvedValue({ success: true, reply: '回答' });
  vi.stubGlobal('ResizeObserver', class { observe() {} unobserve() {} disconnect() {} });
  vi.stubGlobal('IntersectionObserver', class { observe() {} unobserve() {} disconnect() {} });
  HTMLElement.prototype.scrollIntoView = vi.fn();
  HTMLElement.prototype.hasPointerCapture = vi.fn(() => false);
  HTMLElement.prototype.setPointerCapture = vi.fn();
  HTMLElement.prototype.releasePointerCapture = vi.fn();
});
async function open() {
  fireEvent.click(screen.getByRole('button', { name: '打开 AI 小助' }));
  await waitFor(() => expect(screen.getByRole('combobox', { name: '对话模型' })).toBeEnabled());
}
async function chooseModel() {
  const user = userEvent.setup();
  await user.click(screen.getByRole('combobox', { name: '对话模型' }));
  await user.click(screen.getByRole('option', { name: 'maas/model-b', exact: true }));
}
async function send() {
  fireEvent.change(screen.getByPlaceholderText('问我任何问题...'), { target: { value: '分析本月收入' } });
  fireEvent.click(screen.getByRole('button', { name: '发送消息' }));
  await waitFor(() => expect(mocks.runAction).toHaveBeenCalled());
}

it('sends an empty model by default and the selected MaaS id for this conversation', async () => {
  render(<AICopilot />);
  await open();
  await send();
  expect(mocks.runAction).toHaveBeenLastCalledWith(context.chat_action, expect.objectContaining({ model: '' }));
  await chooseModel();
  await send();
  expect(mocks.runAction).toHaveBeenLastCalledWith(context.chat_action, expect.objectContaining({ model: 'maas/model-b' }));
});

it('distinguishes directory failure from empty success, permits default chat, and retries', async () => {
  mocks.get.mockImplementation((path) => path.endsWith('chat-context') ? Promise.resolve(context) : Promise.reject(new Error('offline')));
  render(<AICopilot />);
  fireEvent.click(screen.getByRole('button', { name: '打开 AI 小助' }));
  expect(await screen.findByRole('alert')).toHaveTextContent('模型目录加载失败，将使用服务端默认');
  expect(screen.queryByText('目录暂无可选模型，使用服务端默认。')).not.toBeInTheDocument();
  await send();
  expect(mocks.runAction).toHaveBeenLastCalledWith(context.chat_action, expect.objectContaining({ model: '' }));
  mocks.get.mockResolvedValue(catalog);
  fireEvent.click(screen.getByRole('button', { name: '重试模型目录' }));
  await waitFor(() => expect(screen.getByRole('combobox')).toBeEnabled());
  expect(screen.queryByRole('alert')).not.toBeInTheDocument();
});

it('clears selection and history and reloads the directory on company change', async () => {
  render(<AICopilot />);
  await open();
  await chooseModel();
  await send();
  await screen.findByText('回答');
  act(() => { mocks.company = 'B'; window.dispatchEvent(new Event('erp-company-changed')); });
  await open();
  expect(screen.queryByText('回答')).not.toBeInTheDocument();
  expect(screen.getByRole('combobox')).toHaveTextContent('服务端默认');
  mocks.runAction.mockClear();
  await send();
  expect(mocks.runAction).toHaveBeenLastCalledWith(context.chat_action, expect.objectContaining({ model: '', history: [] }));
  expect(mocks.get.mock.calls.filter(([path]) => path === '/api/ai/models')).toHaveLength(2);
});

it('does not apply a previous company directory arriving after a switch', async () => {
  let resolve!: (value: typeof catalog) => void;
  mocks.get.mockImplementation((path) => path.endsWith('chat-context') ? Promise.resolve(context)
    : mocks.company === 'A' ? new Promise((done) => { resolve = done; })
    : Promise.resolve({ models: [], default_model: 'company-b-default' }));
  render(<AICopilot />);
  act(() => { mocks.company = 'B'; window.dispatchEvent(new Event('erp-company-changed')); });
  await open();
  await act(async () => { resolve(catalog); });
  expect(screen.getByRole('combobox')).toHaveTextContent('company-b-default');
  expect(screen.queryByText('maas/default')).not.toBeInTheDocument();
});


it('bounds the actual request history and disables clearing while awaiting a reply', async () => {
  mocks.runAction.mockResolvedValueOnce({ success: true, reply: '中'.repeat(22_000) });
  render(<AICopilot />);
  await open();
  await send();
  await waitFor(() => expect(screen.getByRole('button', { name: '清空对话' })).toBeEnabled());
  mocks.runAction.mockClear();
  let resolve!: (value: { success: boolean; reply: string }) => void;
  mocks.runAction.mockReturnValue(new Promise((done) => { resolve = done; }));
  await send();
  expect(mocks.runAction).toHaveBeenLastCalledWith(context.chat_action, expect.objectContaining({ history: [] }));
  expect(screen.getByRole('button', { name: '清空对话' })).toBeDisabled();
  await act(async () => { resolve({ success: true, reply: '完成' }); });
  expect(screen.getByRole('button', { name: '清空对话' })).toBeEnabled();
});
