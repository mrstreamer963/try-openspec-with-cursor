export interface ContentSource {
  readText(path: string): Promise<string>;
  exists?(path: string): Promise<boolean>;
}

export class BundledContentSource implements ContentSource {
  private baseUrl: string;

  constructor(baseUrl = import.meta.env.BASE_URL) {
    this.baseUrl = baseUrl.endsWith('/') ? baseUrl : `${baseUrl}/`;
  }

  private url(path: string): string {
    const normalized = path.startsWith('/') ? path.slice(1) : path;
    return `${this.baseUrl}${normalized}`;
  }

  async readText(path: string): Promise<string> {
    const url = this.url(path);
    let response: Response;
    try {
      response = await fetch(url);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      throw new Error(`Failed to fetch (${url}): ${message}`);
    }
    if (!response.ok) {
      throw new Error(`Failed to fetch (${url}): HTTP ${response.status} ${response.statusText}`);
    }
    return response.text();
  }

  async exists(path: string): Promise<boolean> {
    try {
      const response = await fetch(this.url(path), { method: 'HEAD' });
      return response.ok;
    } catch {
      return false;
    }
  }
}

export class UserModContentSource implements ContentSource {
  constructor(
    private readModFile: (modId: string, filename: string) => Promise<string>,
    private modFileExists: (modId: string, filename: string) => Promise<boolean>,
  ) {}

  async readText(path: string): Promise<string> {
    const match = path.match(/^mods\/([^/]+)\/(.+)$/);
    if (!match) {
      throw new Error(`Invalid user mod path: ${path}`);
    }
    const [, modId, filename] = match;
    return this.readModFile(modId, filename);
  }

  async exists(path: string): Promise<boolean> {
    const match = path.match(/^mods\/([^/]+)\/(.+)$/);
    if (!match) return false;
    const [, modId, filename] = match;
    return this.modFileExists(modId, filename);
  }
}
