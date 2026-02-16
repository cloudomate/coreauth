import { HttpClient } from '../http.js';

export class MfaService {
  constructor(private http: HttpClient) {}

  enrollTotp(): Promise<any> {
    return this.http.post('/api/mfa/enroll/totp');
  }

  verifyTotp(methodId: string, code: string): Promise<any> {
    return this.http.post(`/api/mfa/totp/${methodId}/verify`, { code });
  }

  enrollSms(phoneNumber: string): Promise<any> {
    return this.http.post('/api/mfa/enroll/sms', { phone_number: phoneNumber });
  }

  verifySms(methodId: string, code: string): Promise<any> {
    return this.http.post(`/api/mfa/sms/${methodId}/verify`, { code });
  }

  resendSms(methodId: string): Promise<any> {
    return this.http.post(`/api/mfa/sms/${methodId}/resend`);
  }

  listMethods(): Promise<any> {
    return this.http.get('/api/mfa/methods');
  }

  deleteMethod(methodId: string): Promise<any> {
    return this.http.delete(`/api/mfa/methods/${methodId}`);
  }

  regenerateBackupCodes(): Promise<any> {
    return this.http.post('/api/mfa/backup-codes/regenerate');
  }

  enrollTotpWithToken(enrollmentToken: string): Promise<any> {
    return this.http.post('/api/mfa/enroll-with-token/totp', { enrollment_token: enrollmentToken });
  }

  verifyTotpWithToken(methodId: string, enrollmentToken: string, code: string): Promise<any> {
    return this.http.post(`/api/mfa/verify-with-token/totp/${methodId}`, {
      enrollment_token: enrollmentToken,
      code,
    });
  }
}
