import { HttpClient } from '../http.js';

export class FgaService {
  constructor(private http: HttpClient) {}

  // --- Tuples (tenant-scoped) ---

  createTuple(data: Record<string, any>): Promise<any> {
    return this.http.post('/api/authz/tuples', data);
  }

  deleteTuple(data: Record<string, any>): Promise<any> {
    return this.http.delete('/api/authz/tuples', data);
  }

  queryTuples(data: Record<string, any>): Promise<any> {
    return this.http.post('/api/authz/tuples/query', data);
  }

  getObjectTuples(tenantId: string, namespace: string, objectId: string): Promise<any> {
    return this.http.get(`/api/authz/tuples/by-object/${tenantId}/${namespace}/${objectId}`);
  }

  getSubjectTuples(tenantId: string, subjectType: string, subjectId: string): Promise<any> {
    return this.http.get(`/api/authz/tuples/by-subject/${tenantId}/${subjectType}/${subjectId}`);
  }

  // --- Checks ---

  check(data: Record<string, any>): Promise<any> {
    return this.http.post('/api/authz/check', data);
  }

  expand(tenantId: string, namespace: string, objectId: string, relation: string): Promise<any> {
    return this.http.get(`/api/authz/expand/${tenantId}/${namespace}/${objectId}/${relation}`);
  }

  forwardAuth(data: Record<string, any>): Promise<any> {
    return this.http.post('/authz/forward-auth', data);
  }

  // --- Stores ---

  createStore(name: string, description?: string): Promise<any> {
    const body: Record<string, any> = { name };
    if (description) body.description = description;
    return this.http.post('/api/fga/stores', body);
  }

  listStores(params?: Record<string, any>): Promise<any> {
    return this.http.get('/api/fga/stores', params);
  }

  getStore(storeId: string): Promise<any> {
    return this.http.get(`/api/fga/stores/${storeId}`);
  }

  updateStore(storeId: string, data: Record<string, any>): Promise<any> {
    return this.http.patch(`/api/fga/stores/${storeId}`, data);
  }

  deleteStore(storeId: string): Promise<any> {
    return this.http.delete(`/api/fga/stores/${storeId}`);
  }

  // --- Models ---

  writeModel(storeId: string, schema: any, createdBy?: string): Promise<any> {
    const body: Record<string, any> = { schema };
    if (createdBy) body.created_by = createdBy;
    return this.http.post(`/api/fga/stores/${storeId}/models`, body);
  }

  listModels(storeId: string): Promise<any> {
    return this.http.get(`/api/fga/stores/${storeId}/models`);
  }

  getCurrentModel(storeId: string): Promise<any> {
    return this.http.get(`/api/fga/stores/${storeId}/models/current`);
  }

  getModelVersion(storeId: string, version: string): Promise<any> {
    return this.http.get(`/api/fga/stores/${storeId}/models/${version}`);
  }

  // --- API Keys ---

  createApiKey(storeId: string, data: Record<string, any>): Promise<any> {
    return this.http.post(`/api/fga/stores/${storeId}/api-keys`, data);
  }

  listApiKeys(storeId: string): Promise<any> {
    return this.http.get(`/api/fga/stores/${storeId}/api-keys`);
  }

  revokeApiKey(storeId: string, keyId: string): Promise<any> {
    return this.http.delete(`/api/fga/stores/${storeId}/api-keys/${keyId}`);
  }

  // --- Store operations ---

  storeCheck(storeId: string, data: Record<string, any>): Promise<any> {
    return this.http.post(`/api/fga/stores/${storeId}/check`, data);
  }

  readStoreTuples(storeId: string, params?: Record<string, any>): Promise<any> {
    return this.http.get(`/api/fga/stores/${storeId}/tuples`, params);
  }

  writeStoreTuples(storeId: string, data: Record<string, any>): Promise<any> {
    return this.http.post(`/api/fga/stores/${storeId}/tuples`, data);
  }
}
