export class CoreAuthError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'CoreAuthError';
  }
}

export class ApiError extends CoreAuthError {
  public readonly statusCode: number;
  public readonly error: string;

  constructor(statusCode: number, error: string, message: string) {
    super(`[${statusCode}] ${error}: ${message}`);
    this.name = 'ApiError';
    this.statusCode = statusCode;
    this.error = error;
  }
}

export class AuthenticationError extends ApiError {
  constructor(error = 'unauthorized', message = 'Authentication required') {
    super(401, error, message);
    this.name = 'AuthenticationError';
  }
}

export class ForbiddenError extends ApiError {
  constructor(error = 'forbidden', message = 'Insufficient permissions') {
    super(403, error, message);
    this.name = 'ForbiddenError';
  }
}

export class NotFoundError extends ApiError {
  constructor(error = 'not_found', message = 'Resource not found') {
    super(404, error, message);
    this.name = 'NotFoundError';
  }
}

export class ConflictError extends ApiError {
  constructor(error = 'conflict', message = 'Resource already exists') {
    super(409, error, message);
    this.name = 'ConflictError';
  }
}

export class ValidationError extends ApiError {
  constructor(error = 'validation_error', message = 'Invalid request') {
    super(400, error, message);
    this.name = 'ValidationError';
  }
}

export class RateLimitError extends ApiError {
  constructor(error = 'rate_limited', message = 'Rate limit exceeded') {
    super(429, error, message);
    this.name = 'RateLimitError';
  }
}
