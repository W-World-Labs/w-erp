import { act, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { InvoiceDialog } from './InvoiceDialog';

const mocks = vi.hoisted(() => ({
  parseFile: vi.fn(), runAction: vi.fn(), success: vi.fn(),
  options: {
    invoice_type_options: [
      { value: 'receivable', label: '应收' },
      { value: 'payable', label: '应付' },
    ],
    client_options: [], default_invoice_type: 'receivable',
  },
}));
vi.mock('@tanstack/react-query', () => ({ useQuery: () => ({ data: mocks.options }) }));
vi.mock('@/hooks/useDocumentParser', () => ({
  useDocumentParser: () => ({ parseFile: mocks.parseFile, isParsing: false, error: null, enabled: true }),
}));
vi.mock('@/integrations/api/client', () => ({
  api: { runAction: mocks.runAction }, files: { upload: async () => ({ id: 'attachment' }) },
}));
vi.mock('sonner', () => ({ toast: { success: mocks.success, error: vi.fn() } }));

const props = { open: true, onOpenChange: vi.fn(), onSuccess: vi.fn(), action: { path: '/api/invoices', method: 'POST' } as any };
const extracted = {
  invoice_number: 'TEST-001', amount: '100', tax_amount: '13', total_amount: '113',
  supplier: 'Seller Company', buyer: 'Buyer Company', invoice_type: 'input',
  invoice_date: '2025-01-01', parse_source: 'xml',
};
function selectFile(name = 'invoice.xml') {
  fireEvent.change(document.querySelector('#invoice-file')!, {
    target: { files: [new File(['synthetic invoice'], name, { type: 'application/xml' })] },
  });
}
async function parse() {
  selectFile();
  fireEvent.click(screen.getByRole('button', { name: 'AI 智能识别' }));
  await waitFor(() => expect(mocks.success).toHaveBeenCalled());
}

beforeEach(() => {
  vi.clearAllMocks();
  mocks.parseFile.mockResolvedValue(extracted);
  mocks.runAction.mockResolvedValue({});
});

describe('OCR invoice confirmation', () => {
  it('saves gross and seller when a receivable form recognizes an input invoice', async () => {
    render(<InvoiceDialog {...props} />);
    await parse();
    expect(screen.getByLabelText('价税合计（元）*')).toHaveValue('113');
    expect(screen.getByLabelText('供应商名称 *')).toHaveValue('Seller Company');
    fireEvent.click(screen.getByRole('button', { name: '添加', exact: true }));
    await waitFor(() => expect(mocks.runAction).toHaveBeenCalledWith(props.action, expect.objectContaining({
      amount: '113', tax_amount: '13', supplier: 'Seller Company', invoice_type: 'payable',
    })));
  });

  it('selects buyer using the recognized output type and adds explicit net and tax when total is absent', async () => {
    mocks.parseFile.mockResolvedValue({ ...extracted, total_amount: undefined, invoice_type: 'output' });
    render(<InvoiceDialog {...props} defaultInvoiceType="payable" />);
    await parse();
    expect(screen.getByLabelText('客户名称')).toHaveValue('Buyer Company');
    expect(screen.getByLabelText('价税合计（元）*')).toHaveValue('113');
  });

  it('preserves an explicit zero gross without adding tax a second time', async () => {
    mocks.parseFile.mockResolvedValue({ ...extracted, total_amount: '0', amount: '0', tax_amount: '0' });
    render(<InvoiceDialog {...props} />);
    await parse();
    expect(screen.getByLabelText('价税合计（元）*')).toHaveValue('0');
  });

  it.each(['replace', 'remove', 'reopen'] as const)('ignores a late result after %s', async (change) => {
    let resolve!: (fields: typeof extracted) => void;
    mocks.parseFile.mockReturnValue(new Promise((done) => { resolve = done; }));
    const view = render(<InvoiceDialog {...props} />);
    selectFile('A.xml');
    fireEvent.click(screen.getByRole('button', { name: 'AI 智能识别' }));
    if (change === 'replace') selectFile('B.xml');
    if (change === 'remove') {
      const attachment = screen.getByText('A.xml').parentElement!;
      fireEvent.click(attachment.querySelector('button')!);
    }
    if (change === 'reopen') {
      view.rerender(<InvoiceDialog {...props} open={false} />);
      view.rerender(<InvoiceDialog {...props} />);
    }
    fireEvent.change(screen.getByLabelText('价税合计（元）*'), { target: { value: '789' } });
    await act(async () => { resolve(extracted); });
    expect(screen.getByLabelText('价税合计（元）*')).toHaveValue('789');
    expect(mocks.success).not.toHaveBeenCalled();
  });
});
