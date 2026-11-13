/**
 * Golden parity test for the OPC API client after migrating its fetch core
 * onto @w/http-web. Asserts the request the client builds (method / URL /
 * headers / body) and the error behaviour (bare Error, 401 redirect, auto
 * error-reporting, 204 handling) are byte-identical to the hand-rolled
 * `request<T>` it replaced.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const { mockReportError } = vi.hoisted(() => ({ mockReportError: vi.fn() }));
vi.mock('@/lib/errorReporter', () => ({ reportError: mockReportError }));

import { api } from '@/integrations/api/client';

let fetchMock: ReturnType<typeof vi.fn>;
let originalLocation: Location;

beforeEach(() => {
    mockReportError.mockReset();
    localStorage.clear();
    fetchMock = vi.fn();
    vi.stubGlobal('fetch', fetchMock);
    originalLocation = window.location;
    Object.defineProperty(window, 'location', {
        configurable: true,
        writable: true,
        value: { href: '/' },
    });
});

afterEach(() => {
    vi.unstubAllGlobals();
    Object.defineProperty(window, 'location', {
        configurable: true,
        writable: true,
        value: originalLocation,
    });
});

describe('OPC API client parity (@w/http-web)', () => {
    it('GET: sends the HttpOnly cookie through credentialed fetch', async () => {
        fetchMock.mockResolvedValue(new Response(JSON.stringify({ ok: 1 }), { status: 200 }));

        const result = await api.get<{ ok: number }>('/api/things');

        expect(result).toEqual({ ok: 1 });
        expect(fetchMock).toHaveBeenCalledTimes(1);
        const [url, init] = fetchMock.mock.calls[0];
        expect(url).toBe('/api/things'); // API_BASE === '' in test env → raw concat
        // api.get omits `method` (fetch defaults to GET) — byte-identical to the old client
        expect(init.method).toBeUndefined();
        expect(init.headers).toEqual({
            'Content-Type': 'application/json',
        });
        expect(init.credentials).toBe('include');
        expect(init.body).toBeUndefined();
    });

    it('GET: no session → no Authorization header', async () => {
        fetchMock.mockResolvedValue(new Response('{}', { status: 200 }));

        await api.get('/api/public');

        const [, init] = fetchMock.mock.calls[0];
        expect(init.headers).toEqual({ 'Content-Type': 'application/json' });
    });

    it('POST: JSON-stringifies a non-FormData body and uses cookie credentials', async () => {
        fetchMock.mockResolvedValue(new Response('{}', { status: 200 }));

        await api.post('/api/x', { a: 1, b: 'two' });

        const [url, init] = fetchMock.mock.calls[0];
        expect(url).toBe('/api/x');
        expect(init.method).toBe('POST');
        expect(init.headers['Content-Type']).toBe('application/json');
        expect(init.headers.Authorization).toBeUndefined();
        expect(init.credentials).toBe('include');
        expect(init.body).toBe(JSON.stringify({ a: 1, b: 'two' }));
    });

    it('POST: FormData body → no Content-Type header, body passed through', async () => {
        fetchMock.mockResolvedValue(new Response('{}', { status: 200 }));

        const fd = new FormData();
        fd.append('file', 'x');
        await api.post('/api/upload', fd);

        const [, init] = fetchMock.mock.calls[0];
        expect(init.headers['Content-Type']).toBeUndefined();
        expect(init.body).toBe(fd);
    });

    it('POST: falsy/omitted body → no body (api wrapper logic preserved)', async () => {
        fetchMock.mockResolvedValue(new Response('{}', { status: 200 }));

        await api.post('/api/noop');

        const [, init] = fetchMock.mock.calls[0];
        expect(init.method).toBe('POST');
        expect(init.body).toBeUndefined();
    });

    it('PUT / DELETE: method + JSON body', async () => {
        fetchMock.mockImplementation(() => Promise.resolve(new Response('{}', { status: 200 })));
        await api.put('/api/x/1', { a: 2 });
        let [, init] = fetchMock.mock.calls[0];
        expect(init.method).toBe('PUT');
        expect(init.body).toBe(JSON.stringify({ a: 2 }));

        await api.delete('/api/x/1');
        [, init] = fetchMock.mock.calls[1];
        expect(init.method).toBe('DELETE');
        expect(init.body).toBeUndefined();
    });

    it('non-2xx: throws bare Error(text) and auto-reports', async () => {
        fetchMock.mockResolvedValue(new Response('boom', { status: 500, statusText: 'ISE' }));

        await expect(api.get('/api/fail')).rejects.toThrowError('boom');

        let thrown: unknown;
        fetchMock.mockResolvedValue(new Response('boom', { status: 500, statusText: 'ISE' }));
        try {
            await api.get('/api/fail');
        } catch (e) {
            thrown = e;
        }
        expect(thrown).toBeInstanceOf(Error);
        expect((thrown as Error).message).toBe('boom');
        // bare Error, NOT the {code} ApiError — faithful to the old client
        expect((thrown as { code?: unknown }).code).toBeUndefined();

        expect(mockReportError).toHaveBeenCalledWith({
            error_type: 'api_error',
            error_message: 'boom',
            metadata: { endpoint: '/api/fail', method: 'GET', status: 500 },
        });
    });

    it('non-2xx: empty body → message "HTTP <status>"', async () => {
        fetchMock.mockResolvedValue(new Response('', { status: 404 }));
        await expect(api.get('/api/missing')).rejects.toThrowError('HTTP 404');
    });

    it('error on an /error_logs endpoint is NOT reported (recursion guard)', async () => {
        fetchMock.mockResolvedValue(new Response('x', { status: 500 }));
        await expect(api.get('/api/v1/error_logs')).rejects.toThrow();
        expect(mockReportError).not.toHaveBeenCalled();
    });

    it('401: redirects to /auth without creating a JavaScript-readable session', async () => {
        fetchMock.mockResolvedValue(new Response('nope', { status: 401 }));

        await expect(api.get('/api/secure')).rejects.toThrowError('Unauthorized');

        expect(localStorage.getItem('session_id')).toBeNull();
        expect(window.location.href).toBe('/auth');
        expect(mockReportError).not.toHaveBeenCalled();
    });

    it('204 No Content → resolves undefined', async () => {
        fetchMock.mockResolvedValue(new Response(null, { status: 204 }));
        await expect(api.get('/api/none')).resolves.toBeUndefined();
    });
});
