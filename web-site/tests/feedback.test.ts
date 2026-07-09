import { describe, it, expect } from 'vitest';
import { validateFeedback } from '../src/lib/feedback';

describe('validateFeedback', () => {
  it('valid submission', () => {
    const result = validateFeedback({
      name: 'John Doe',
      email: 'john@example.com',
      message: 'Great site!',
      category: 'General',
    });
    expect(result).toEqual({
      valid: true,
      data: {
        name: 'John Doe',
        email: 'john@example.com',
        message: 'Great site!',
        category: 'General',
      },
    });
  });

  it('missing name', () => {
    const result = validateFeedback({
      email: 'john@example.com',
      message: 'Great site!',
      category: 'General',
    });
    expect(result).toEqual({
      valid: false,
      status: 400,
      error: 'Name is required.',
    });
  });

  it('empty name (whitespace only)', () => {
    const result = validateFeedback({
      name: '   ',
      email: 'john@example.com',
      message: 'Great site!',
      category: 'General',
    });
    expect(result).toEqual({
      valid: false,
      status: 400,
      error: 'Name is required.',
    });
  });

  it('missing email', () => {
    const result = validateFeedback({
      name: 'John Doe',
      message: 'Great site!',
      category: 'General',
    });
    expect(result).toEqual({
      valid: false,
      status: 400,
      error: 'Email is required.',
    });
  });

  it('email without @', () => {
    const result = validateFeedback({
      name: 'John Doe',
      email: 'notanemail',
      message: 'Great site!',
      category: 'General',
    });
    expect(result).toEqual({
      valid: false,
      status: 400,
      error: 'Email must contain @.',
    });
  });

  it('missing message', () => {
    const result = validateFeedback({
      name: 'John Doe',
      email: 'john@example.com',
      category: 'General',
    });
    expect(result).toEqual({
      valid: false,
      status: 400,
      error: 'Message is required.',
    });
  });

  it('empty message (whitespace only)', () => {
    const result = validateFeedback({
      name: 'John Doe',
      email: 'john@example.com',
      message: '   ',
      category: 'General',
    });
    expect(result).toEqual({
      valid: false,
      status: 400,
      error: 'Message is required.',
    });
  });

  it('invalid category', () => {
    const result = validateFeedback({
      name: 'John Doe',
      email: 'john@example.com',
      message: 'Great site!',
      category: 'Spam',
    });
    expect(result).toEqual({
      valid: false,
      status: 400,
      error: 'Invalid category.',
    });
  });
});
