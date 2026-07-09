export interface FeedbackInput {
  name: string;
  email: string;
  message: string;
  category: string;
}

const VALID_CATEGORIES = ['General', 'Bug Report', 'Feature Request', 'Other'] as const;

export type ValidationResult =
  | { valid: true; data: FeedbackInput }
  | { valid: false; status: number; error: string };

export function validateFeedback(body: unknown): ValidationResult {
  if (typeof body !== 'object' || body === null) {
    return { valid: false, status: 400, error: 'Name is required.' };
  }

  const data = body as Record<string, unknown>;

  if (typeof data.name !== 'string' || data.name.trim().length === 0) {
    return { valid: false, status: 400, error: 'Name is required.' };
  }

  if (typeof data.email !== 'string' || data.email.trim().length === 0) {
    return { valid: false, status: 400, error: 'Email is required.' };
  }

  if (!data.email.includes('@')) {
    return { valid: false, status: 400, error: 'Email must contain @.' };
  }

  if (typeof data.message !== 'string' || data.message.trim().length === 0) {
    return { valid: false, status: 400, error: 'Message is required.' };
  }

  if (!VALID_CATEGORIES.includes(data.category as typeof VALID_CATEGORIES[number])) {
    return { valid: false, status: 400, error: 'Invalid category.' };
  }

  return {
    valid: true,
    data: {
      name: data.name.trim(),
      email: data.email.trim(),
      message: data.message.trim(),
      category: data.category as string,
    },
  };
}
