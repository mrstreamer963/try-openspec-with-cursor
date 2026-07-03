import { afterEach, describe, expect, it, vi } from 'vitest';
import { createWebResourceManager } from './webResourceManager';

function mockFetchResponse(status: number, contentType: string | null): Response {
  return {
    status,
    ok: status >= 200 && status < 300,
    statusText: status === 404 ? 'Not Found' : 'OK',
    headers: { get: (name: string) => (name === 'content-type' ? contentType : null) },
    text: async () => '',
  } as Response;
}

describe('createWebResourceManager bundled exists', () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('returns false when HEAD gets SPA HTML fallback', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async () => mockFetchResponse(200, 'text/html')),
    );
    const resources = createWebResourceManager('/');
    expect(await resources.exists('bundled', 'mods/hardmode/statuses.yaml')).toBe(false);
  });

  it('returns false when HEAD gets 404', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async () => mockFetchResponse(404, null)),
    );
    const resources = createWebResourceManager('/');
    expect(await resources.exists('bundled', 'mods/hardmode/statuses.yaml')).toBe(false);
  });

  it('returns true when HEAD gets a real yaml asset', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async () => mockFetchResponse(200, 'text/yaml')),
    );
    const resources = createWebResourceManager('/');
    expect(await resources.exists('bundled', 'base/needs.yaml')).toBe(true);
  });
});
