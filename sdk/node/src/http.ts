import {
  ApiError,
  AuthenticationError,
  ConflictError,
  CoreAuthError,
  ForbiddenError,
  NotFoundError,
  RateLimitError,
  ValidationError,
} from './errors.js';

export class HttpClient {
  private baseUrl: string;
  private token: string | null;

  constructor(baseUrl: string, token?: string) {
    this.baseUrl = baseUrl.replace(/\/+$/, '');
    this.token = token ?? null;
  }

  setToken(token: string): void {
    this.token = token;
  }

  clearToken(): void {
    this.token = null;
  }

  private headers(): Record<string, string> {
    const h: Record<string, string> = { 'Content-Type': 'application/json' };
    if (this.token) h['Authorization'] = `Bearer ${this.token}`;
    return h;
  }

  private formHeaders(): Record<string, string> {
    const h: Record<string, string> = { 'Content-Type': 'application/x-www-form-urlencoded' };
    if (this.token) h['Authorization'] = `Bearer ${this.token}`;
    return h;
  }

  private async handleResponse(resp: Response): Promise<any> {
    if (resp.status === 204) return null;
    if (resp.ok) {
      const text = await resp.text();
      return text ? JSON.parse(text) : null;
    }
    let error = '', message = '';
    try {
      const body = await resp.json();
      error = body.error ?? '';
      message = body.message ?? '';
    } catch {
      message = await resp.text().catch(() => '');
    }
    const map: Record<number, new (e: string, m: string) => ApiError> = {
      400: ValidationError,
      401: AuthenticationError,
      403: ForbiddenError,
      404: NotFoundError,
      409: ConflictError,
      429: RateLimitError,
    };
    const Cls = map[resp.status];
    if (Cls) throw new Cls(error, message);
    throw new ApiError(resp.status, error, message);
  }

  private buildQuery(params?: Record<string, any>): string {
    if (!params) return '';
    const entries = Object.entries(params).filter(([, v]) => v != null);
    if (!entries.length) return '';
    return '?' + new URLSearchParams(entries.map(([k, v]) => [k, String(v)])).toString();
  }

  async get(path: string, params?: Record<string, any>): Promise<any> {
    const url = `${this.baseUrl}${path}${this.buildQuery(params)}`;
    try {
      const resp = await fetch(url, { method: 'GET', headers: this.headers() });
      return this.handleResponse(resp);
    } catch (e) {
      if (e instanceof ApiError) throw e;
      throw new CoreAuthError(`Request failed: ${e}`);
    }
  }

  async post(path: string, json?: any): Promise<any> {
    const url = `${this.baseUrl}${path}`;
    try {
      const resp = await fetch(url, {
        method: 'POST',
        headers: this.headers(),
        body: json != null ? JSON.stringify(json) : undefined,
      });
      return this.handleResponse(resp);
    } catch (e) {
      if (e instanceof ApiError) throw e;
      throw new CoreAuthError(`Request failed: ${e}`);
    }
  }

  async postForm(path: string, data: Record<string, string>): Promise<any> {
    const url = `${this.baseUrl}${path}`;
    try {
      const resp = await fetch(url, {
        method: 'POST',
        headers: this.formHeaders(),
        body: new URLSearchParams(data).toString(),
      });
      return this.handleResponse(resp);
    } catch (e) {
      if (e instanceof ApiError) throw e;
      throw new CoreAuthError(`Request failed: ${e}`);
    }
  }

  async put(path: string, json?: any): Promise<any> {
    const url = `${this.baseUrl}${path}`;
    try {
      const resp = await fetch(url, {
        method: 'PUT',
        headers: this.headers(),
        body: json != null ? JSON.stringify(json) : undefined,
      });
      return this.handleResponse(resp);
    } catch (e) {
      if (e instanceof ApiError) throw e;
      throw new CoreAuthError(`Request failed: ${e}`);
    }
  }

  async patch(path: string, json?: any): Promise<any> {
    const url = `${this.baseUrl}${path}`;
    try {
      const resp = await fetch(url, {
        method: 'PATCH',
        headers: this.headers(),
        body: json != null ? JSON.stringify(json) : undefined,
      });
      return this.handleResponse(resp);
    } catch (e) {
      if (e instanceof ApiError) throw e;
      throw new CoreAuthError(`Request failed: ${e}`);
    }
  }

  async delete(path: string, json?: any): Promise<any> {
    const url = `${this.baseUrl}${path}`;
    try {
      const resp = await fetch(url, {
        method: 'DELETE',
        headers: this.headers(),
        body: json != null ? JSON.stringify(json) : undefined,
      });
      return this.handleResponse(resp);
    } catch (e) {
      if (e instanceof ApiError) throw e;
      throw new CoreAuthError(`Request failed: ${e}`);
    }
  }
}
