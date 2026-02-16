import { HttpClient } from '../http.js';

export class OAuth2Service {
  constructor(private http: HttpClient) {}

  discovery(): Promise<any> {
    return this.http.get('/.well-known/openid-configuration');
  }

  jwks(): Promise<any> {
    return this.http.get('/.well-known/jwks.json');
  }

  authorize(
    clientId: string,
    redirectUri: string,
    responseType: string = 'code',
    scope: string = 'openid',
    params?: Record<string, string>,
  ): string {
    const query: Record<string, string> = {
      client_id: clientId,
      redirect_uri: redirectUri,
      response_type: responseType,
      scope,
      ...params,
    };
    const baseUrl = ((this.http as any).baseUrl as string).replace(/\/+$/, '');
    return `${baseUrl}/authorize?${new URLSearchParams(query).toString()}`;
  }

  token(grantType: string, params?: Record<string, string>): Promise<any> {
    const data: Record<string, string> = { grant_type: grantType, ...params };
    return this.http.postForm('/oauth/token', data);
  }

  userinfo(): Promise<any> {
    return this.http.get('/userinfo');
  }

  revoke(token: string, tokenTypeHint?: string): Promise<any> {
    const data: Record<string, string> = { token };
    if (tokenTypeHint) data.token_type_hint = tokenTypeHint;
    return this.http.postForm('/oauth/revoke', data);
  }

  introspect(token: string, tokenTypeHint?: string): Promise<any> {
    const data: Record<string, string> = { token };
    if (tokenTypeHint) data.token_type_hint = tokenTypeHint;
    return this.http.postForm('/oauth/introspect', data);
  }

  oidcLogout(params?: {
    id_token_hint?: string;
    post_logout_redirect_uri?: string;
    state?: string;
  }): Promise<any> {
    return this.http.get('/logout', params);
  }
}
