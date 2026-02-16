import { HttpClient } from '../http.js';

export class WebhooksService {
  constructor(private http: HttpClient) {}

  create(orgId: string, data: Record<string, any>): Promise<any> {
    return this.http.post(`/api/organizations/${orgId}/webhooks`, data);
  }

  list(orgId: string): Promise<any> {
    return this.http.get(`/api/organizations/${orgId}/webhooks`);
  }

  get(orgId: string, webhookId: string): Promise<any> {
    return this.http.get(`/api/organizations/${orgId}/webhooks/${webhookId}`);
  }

  update(orgId: string, webhookId: string, data: Record<string, any>): Promise<any> {
    return this.http.put(`/api/organizations/${orgId}/webhooks/${webhookId}`, data);
  }

  delete(orgId: string, webhookId: string): Promise<any> {
    return this.http.delete(`/api/organizations/${orgId}/webhooks/${webhookId}`);
  }

  rotateSecret(orgId: string, webhookId: string): Promise<any> {
    return this.http.post(`/api/organizations/${orgId}/webhooks/${webhookId}/rotate-secret`);
  }

  test(orgId: string, webhookId: string, eventType?: string): Promise<any> {
    const body: Record<string, any> = {};
    if (eventType) body.event_type = eventType;
    return this.http.post(`/api/organizations/${orgId}/webhooks/${webhookId}/test`, body);
  }

  listDeliveries(orgId: string, webhookId: string, params?: Record<string, any>): Promise<any> {
    return this.http.get(`/api/organizations/${orgId}/webhooks/${webhookId}/deliveries`, params);
  }

  getDelivery(orgId: string, webhookId: string, deliveryId: string): Promise<any> {
    return this.http.get(`/api/organizations/${orgId}/webhooks/${webhookId}/deliveries/${deliveryId}`);
  }

  retryDelivery(orgId: string, webhookId: string, deliveryId: string): Promise<any> {
    return this.http.post(`/api/organizations/${orgId}/webhooks/${webhookId}/deliveries/${deliveryId}/retry`);
  }

  listEventTypes(): Promise<any> {
    return this.http.get('/api/webhooks/event-types');
  }
}
