const FORBIDDEN_KEYS = [
  "password",
  "passwd",
  "secret",
  "token",
  "api_key",
  "apikey",
  "payment",
  "card",
  "private_message",
  "privateMessages",
]

const UNSUPPORTED_CLAIM_PATTERNS = [
  /\btop rated\b/i,
  /\b100%\s*job success\b/i,
  /\bearned\s+\$?\d+/i,
  /\bguarantee(?:d)?\s+results?\b/i,
  /\bavailable\s+24\/7\b/i,
]

export function assertNoSecrets(value: unknown, path = "input"): void {
  if (Array.isArray(value)) {
    value.forEach((item, index) => assertNoSecrets(item, `${path}[${index}]`))
    return
  }
  if (!value || typeof value !== "object") return
  for (const [key, nested] of Object.entries(value as Record<string, unknown>)) {
    if (FORBIDDEN_KEYS.some((forbidden) => key.toLowerCase().includes(forbidden.toLowerCase()))) {
      throw new Error(`Refusing to store sensitive field at ${path}.${key}`)
    }
    assertNoSecrets(nested, `${path}.${key}`)
  }
}

export function redactSecrets(text: string): string {
  return text
    .replace(/(password|token|secret|api[_-]?key)\s*[:=]\s*\S+/gi, "$1: [REDACTED]")
    .replace(/(\b\d{4}[- ]?){3}\d{4}\b/g, "[REDACTED_CARD]")
}

export function findUnsupportedClaims(text: string): string[] {
  return UNSUPPORTED_CLAIM_PATTERNS
    .filter((pattern) => pattern.test(text))
    .map((pattern) => `Potential unsupported claim: ${pattern.source}`)
}
