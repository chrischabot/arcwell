export interface Env {
  ARCWELL_EDGE_SECRET: string;
  ARCWELL_EDGE_NEXT_SECRET?: string;
  TELEGRAM_WEBHOOK_SECRET?: string;
  EMAIL_ROUTES_JSON?: string;
  EMAIL_ALLOWED_SENDERS_JSON?: string;
  EMAIL_MAX_RAW_BYTES?: string;
  EMAIL_MAX_PREVIEW_CHARS?: string;
  EMAIL_REQUIRE_DMARC_PASS?: string;
  MAX_PAYLOAD_BYTES?: string;
  RATE_LIMIT_WINDOW_SECONDS?: string;
  RATE_LIMIT_MAX_EVENTS?: string;
  EDGE_DB?: D1Database;
}

type EdgeEventInput = {
  source?: unknown;
  idempotencyKey?: unknown;
  payload?: unknown;
  maxAgeSeconds?: unknown;
};

type LeaseInput = {
  leaseSeconds?: unknown;
};

type AckInput = {
  idempotencyKey?: unknown;
};

type NackInput = {
  idempotencyKey?: unknown;
  error?: unknown;
  retrySeconds?: unknown;
};

type InboundEmailMessage = {
  from: string;
  to: string;
  raw: ReadableStream<Uint8Array>;
  rawSize?: number;
  headers: Headers;
  setReject(reason: string): void;
};

type EmailRoute = {
  id?: unknown;
  recipient?: unknown;
  projectId?: unknown;
  allowedSenders?: unknown;
};

export type StoredEdgeEvent = {
  source: string;
  idempotencyKey: string;
  payload: unknown;
  status: "pending" | "leased" | "acked" | "failed" | "dead_lettered" | "expired";
  receivedAt: number;
  expiresAt: number;
  leasedUntil: number | null;
  attempts: number;
  maxAttempts: number;
  error: string | null;
};

export interface EdgeEventStore {
  enqueue(event: StoredEdgeEvent): Promise<{ event: StoredEdgeEvent; duplicate: boolean }>;
  checkRateLimit(key: string, now: number, windowSeconds: number, maxEvents: number): Promise<RateLimitResult>;
  lease(now: number, leaseSeconds: number): Promise<StoredEdgeEvent | null>;
  ack(idempotencyKey: string): Promise<StoredEdgeEvent | null>;
  nack(idempotencyKey: string, error: string, retrySeconds: number, now: number): Promise<StoredEdgeEvent | null>;
  list(now: number, limit: number): Promise<StoredEdgeEvent[]>;
}

type RateLimitResult = {
  allowed: boolean;
  limit: number;
  remaining: number;
  resetAt: number;
};

export default {
  async email(message: ForwardableEmailMessage, env: Env): Promise<void> {
    if (!env.EDGE_DB) {
      message.setReject("missing EDGE_DB binding");
      return;
    }
    const result = await handleEmail(message, env, new D1EdgeEventStore(env.EDGE_DB));
    if (!result.accepted) message.setReject(result.reason ?? "email_rejected");
  },

  async fetch(request: Request, env: Env): Promise<Response> {
    if (!env.EDGE_DB) {
      const url = new URL(request.url);
      if (request.method === "GET" && url.pathname === "/health") {
        return json({ ok: false, service: "arcwell-edge-inbox", error: "missing EDGE_DB binding" }, 503);
      }
      return json({ error: "missing_edge_db_binding" }, 503);
    }
    return handleRequest(request, env, new D1EdgeEventStore(env.EDGE_DB));
  }
};

export async function handleEmail(
  message: InboundEmailMessage,
  env: Env,
  store: EdgeEventStore
): Promise<{ accepted: boolean; duplicate: boolean; reason: string | null; idempotencyKey?: string }> {
  const maxRawBytes = clampNumber(env.EMAIL_MAX_RAW_BYTES ?? env.MAX_PAYLOAD_BYTES, 1024, 512000, 128000);
  if (typeof message.rawSize === "number" && message.rawSize > maxRawBytes) {
    return { accepted: false, duplicate: false, reason: "email_raw_too_large" };
  }
  const routes = parseEmailRoutes(env.EMAIL_ROUTES_JSON);
  if (routes.length === 0) return { accepted: false, duplicate: false, reason: "email_routes_not_configured" };
  const recipient = normalizeEmailAddress(message.to);
  if (!recipient) return { accepted: false, duplicate: false, reason: "missing_email_recipient" };
  const route = routes.find((candidate) => normalizeEmailAddress(String(candidate.recipient ?? "")) === recipient);
  if (!route) return { accepted: false, duplicate: false, reason: "unauthorized_email_route" };

  const trustedSender = normalizeEmailAddress(message.from);
  if (!trustedSender) return { accepted: false, duplicate: false, reason: "missing_trusted_email_sender" };
  const allowedSenders = normalizeEmailAllowedSenders(route.allowedSenders, env.EMAIL_ALLOWED_SENDERS_JSON);
  if (!emailSenderAllowed(trustedSender, allowedSenders)) {
    return { accepted: false, duplicate: false, reason: "unauthorized_email_sender" };
  }

  const raw = await readStreamText(message.raw, maxRawBytes);
  if ("reason" in raw) return { accepted: false, duplicate: false, reason: raw.reason };
  const parsed = parseMimeMessage(raw.value, message.headers);
  const messageId = normalizeHeaderToken(parsed.headers.get("message-id"));
  if (!messageId) return { accepted: false, duplicate: false, reason: "missing_email_message_id" };
  const requireDmarcPass = env.EMAIL_REQUIRE_DMARC_PASS !== "false";
  const auth = parseAuthenticationResults(parsed.headers.get("authentication-results"));
  if (requireDmarcPass && auth.dmarc !== "pass") {
    return { accepted: false, duplicate: false, reason: "email_sender_authentication_failed" };
  }

  const previewChars = clampNumber(env.EMAIL_MAX_PREVIEW_CHARS, 100, 20000, 4000);
  const subject = safeHeaderValue(parsed.headers.get("subject") ?? "(no subject)").slice(0, 180) || "(no subject)";
  const headerFrom = normalizeEmailAddress(parsed.headers.get("from") ?? "");
  const warnings = [];
  if (headerFrom && headerFrom !== trustedSender) warnings.push("header_from_is_display_only_and_differs_from_trusted_sender");
  if (parsed.trackingLinks.length > 0) warnings.push("tracking_links_preserved_as_unfetched_evidence");
  const idempotencyKey = `email:message:${await sha256Hex(messageId, 32)}`;
  const now = Date.now();
  const limited = await enforceRateLimit(store, env, `source:email:recipient:${recipient}`, now);
  if (limited) return { accepted: false, duplicate: false, reason: "email_rate_limited" };
  const routeId = typeof route.id === "string" && route.id.length > 0 ? route.id : recipient;
  const { duplicate } = await store.enqueue({
    source: "email",
    idempotencyKey,
    payload: {
      provider: "cloudflare_email_routing",
      messageId,
      receivedAt: new Date(now).toISOString(),
      routeId,
      projectId: typeof route.projectId === "string" ? route.projectId : null,
      trustedSender,
      headerFrom,
      recipient,
      subject,
      sanitizedText: parsed.text.slice(0, previewChars),
      auth,
      warnings,
      trackingLinks: parsed.trackingLinks
    },
    status: "pending",
    receivedAt: now,
    expiresAt: now + 24 * 60 * 60 * 1000,
    leasedUntil: null,
    attempts: 0,
    maxAttempts: 3,
    error: null
  });
  return { accepted: true, duplicate, reason: null, idempotencyKey };
}

export async function handleRequest(request: Request, env: Env, store: EdgeEventStore): Promise<Response> {
  const url = new URL(request.url);
  if (request.method === "GET" && url.pathname === "/health") {
    return json({ ok: true, service: "arcwell-edge-inbox", durable: true });
  }
  if (request.method === "POST" && url.pathname === "/telegram/webhook") {
    if (!authorized(request, env) && !authorizedTelegram(request, env)) {
      return json({ error: "unauthorized" }, 401);
    }
    return enqueueTelegramUpdate(request, env, store);
  }
  if (!authorized(request, env)) {
    return json({ error: "unauthorized" }, 401);
  }
  if (request.method === "POST" && url.pathname === "/events") {
    return enqueueEvent(request, env, store);
  }
  if (request.method === "POST" && url.pathname === "/drain/lease") {
    const input = await readJson<LeaseInput>(request, env);
    if ("response" in input) return input.response;
    const leaseSeconds = clampNumber(input.value.leaseSeconds, 1, 900, 120);
    const event = await store.lease(Date.now(), leaseSeconds);
    return json({ event });
  }
  if (request.method === "POST" && url.pathname === "/drain/ack") {
    const input = await readJson<AckInput>(request, env);
    if ("response" in input) return input.response;
    const idempotencyKey = validString(input.value.idempotencyKey, "idempotency_key", 200);
    if ("response" in idempotencyKey) return idempotencyKey.response;
    const event = await store.ack(idempotencyKey.value);
    return json({ ok: event !== null, event });
  }
  if (request.method === "POST" && url.pathname === "/drain/nack") {
    const input = await readJson<NackInput>(request, env);
    if ("response" in input) return input.response;
    const idempotencyKey = validString(input.value.idempotencyKey, "idempotency_key", 200);
    if ("response" in idempotencyKey) return idempotencyKey.response;
    const error = validString(input.value.error, "error", 2000);
    if ("response" in error) return error.response;
    const retrySeconds = clampNumber(input.value.retrySeconds, 1, 3600, 60);
    const event = await store.nack(idempotencyKey.value, error.value, retrySeconds, Date.now());
    return json({ ok: event !== null, event });
  }
  if (request.method === "GET" && url.pathname === "/events") {
    const limit = clampNumber(url.searchParams.get("limit"), 1, 100, 25);
    return json({ events: await store.list(Date.now(), limit) });
  }
  return json({ error: "not_found" }, 404);
}

async function enqueueEvent(request: Request, env: Env, store: EdgeEventStore): Promise<Response> {
  const input = await readJson<EdgeEventInput>(request, env);
  if ("response" in input) return input.response;
  const source = validString(input.value.source, "source", 200);
  if ("response" in source) return source.response;
  const idempotencyKey = validString(input.value.idempotencyKey, "idempotency_key", 200);
  if ("response" in idempotencyKey) return idempotencyKey.response;
  const maxAgeSeconds = clampNumber(input.value.maxAgeSeconds, 60, 86400, 3600);
  const now = Date.now();
  const limited = await enforceRateLimit(store, env, `source:${source.value}`, now);
  if (limited) return limited;
  const { event, duplicate } = await store.enqueue({
    source: source.value,
    idempotencyKey: idempotencyKey.value,
    payload: input.value.payload ?? null,
    status: "pending",
    receivedAt: now,
    expiresAt: now + maxAgeSeconds * 1000,
    leasedUntil: null,
    attempts: 0,
    maxAttempts: 3,
    error: null
  });
  return json({
    accepted: true,
    duplicate,
    source: event.source,
    idempotencyKey: event.idempotencyKey,
    status: event.status,
    expiresAt: event.expiresAt
  });
}

async function enqueueTelegramUpdate(request: Request, env: Env, store: EdgeEventStore): Promise<Response> {
  const input = await readJson<Record<string, unknown>>(request, env);
  if ("response" in input) return input.response;
  const update = normalizeTelegramUpdate(input.value);
  if ("response" in update) return update.response;
  const now = Date.now();
  const limited = await enforceRateLimit(store, env, `source:telegram:chat:${update.value.chatId}`, now);
  if (limited) return limited;
  const { event, duplicate } = await store.enqueue({
    source: "telegram",
    idempotencyKey: `telegram:update:${update.value.updateId}`,
    payload: update.value,
    status: "pending",
    receivedAt: now,
    expiresAt: now + 24 * 60 * 60 * 1000,
    leasedUntil: null,
    attempts: 0,
    maxAttempts: 3,
    error: null
  });
  return json({
    accepted: true,
    duplicate,
    source: event.source,
    idempotencyKey: event.idempotencyKey,
    status: event.status
  });
}

function normalizeTelegramUpdate(value: Record<string, unknown>): { value: Record<string, unknown> } | { response: Response } {
  const updateId = value.update_id;
  if (typeof updateId !== "number" || !Number.isInteger(updateId)) {
    return { response: json({ error: "invalid_update_id" }, 400) };
  }
  const message = objectValue(value.message) ?? objectValue(value.edited_message);
  if (!message) {
    return { response: json({ error: "unsupported_telegram_update" }, 400) };
  }
  const chat = objectValue(message.chat);
  const from = objectValue(message.from);
  const text = typeof message.text === "string" ? message.text : typeof message.caption === "string" ? message.caption : null;
  if (!chat || text === null) {
    return { response: json({ error: "unsupported_telegram_message" }, 400) };
  }
  const chatId = chat.id;
  if (typeof chatId !== "number" && typeof chatId !== "string") {
    return { response: json({ error: "invalid_chat_id" }, 400) };
  }
  const messageId = message.message_id;
  if (typeof messageId !== "number" && typeof messageId !== "string") {
    return { response: json({ error: "invalid_message_id" }, 400) };
  }
  return {
    value: {
      updateId,
      chatId,
      messageId,
      senderId: from?.id ?? null,
      username: typeof from?.username === "string" ? from.username : null,
      date: message.date ?? null,
      text
    }
  };
}

function parseEmailRoutes(value: unknown): EmailRoute[] {
  if (typeof value !== "string" || value.trim().length === 0) return [];
  try {
    const parsed = JSON.parse(value) as unknown;
    return Array.isArray(parsed) ? parsed.filter((route) => objectValue(route)) as EmailRoute[] : [];
  } catch {
    return [];
  }
}

function normalizeEmailAllowedSenders(routeAllowed: unknown, globalAllowedJson: unknown): string[] {
  const routeValues = Array.isArray(routeAllowed) ? routeAllowed : null;
  if (routeValues) return routeValues.map(emailSenderRuleString).filter((value) => value.length > 0);
  if (typeof globalAllowedJson !== "string" || globalAllowedJson.trim().length === 0) return [];
  try {
    const parsed = JSON.parse(globalAllowedJson) as unknown;
    return Array.isArray(parsed) ? parsed.map(emailSenderRuleString).filter((value) => value.length > 0) : [];
  } catch {
    return [];
  }
}

function emailSenderRuleString(value: unknown): string {
  if (typeof value === "string") return value.trim().toLowerCase();
  const object = objectValue(value);
  if (!object) return "";
  if (typeof object.address === "string") return object.address.trim().toLowerCase();
  if (typeof object.domain === "string") return `@${object.domain.trim().toLowerCase().replace(/^@/, "")}`;
  return "";
}

function emailSenderAllowed(sender: string, rules: string[]): boolean {
  if (rules.length === 0) return false;
  const senderDomain = sender.split("@")[1] ?? "";
  return rules.some((rule) => {
    const normalized = normalizeEmailAddress(rule);
    if (normalized) return normalized === sender;
    const domain = rule.replace(/^@/, "");
    return domain.length > 0 && senderDomain === domain;
  });
}

async function readStreamText(
  stream: ReadableStream<Uint8Array>,
  maxBytes: number
): Promise<{ value: string } | { reason: string }> {
  const reader = stream.getReader();
  const chunks: Uint8Array[] = [];
  let total = 0;
  while (true) {
    const next = await reader.read();
    if (next.done) break;
    total += next.value.byteLength;
    if (total > maxBytes) return { reason: "email_raw_too_large" };
    chunks.push(next.value);
  }
  const merged = new Uint8Array(total);
  let offset = 0;
  for (const chunk of chunks) {
    merged.set(chunk, offset);
    offset += chunk.byteLength;
  }
  return { value: new TextDecoder().decode(merged) };
}

function parseMimeMessage(raw: string, fallbackHeaders: Headers): { headers: Map<string, string>; text: string; trackingLinks: string[] } {
  const [headerBlock, body] = splitHeaderBody(raw);
  const headers = parseHeaders(headerBlock);
  fallbackHeaders.forEach((value, key) => {
    if (!headers.has(key.toLowerCase())) headers.set(key.toLowerCase(), value);
  });
  const contentType = headers.get("content-type") ?? "text/plain";
  const text = sanitizeEmailText(extractMimeText(body, contentType, headers, 0)).slice(0, 20000);
  return { headers, text, trackingLinks: findTrackingLinks(text) };
}

function extractMimeText(body: string, contentType: string, headers: Map<string, string>, depth: number): string {
  if (depth > 6) return "";
  const transferEncoding = (headers.get("content-transfer-encoding") ?? "").toLowerCase();
  const decodedBody = decodeTransferEncoding(body, transferEncoding);
  const boundary = contentType.match(/boundary="?([^";]+)"?/i)?.[1];
  if (/multipart\//i.test(contentType) && boundary) {
    const texts: string[] = [];
    const htmls: string[] = [];
    for (const part of splitMimeParts(decodedBody, boundary)) {
      const [partHeadersRaw, partBody] = splitHeaderBody(part);
      const partHeaders = parseHeaders(partHeadersRaw);
      const partContentType = partHeaders.get("content-type") ?? "text/plain";
      const extracted = extractMimeText(partBody, partContentType, partHeaders, depth + 1);
      if (/text\/html/i.test(partContentType)) htmls.push(extracted);
      else if (extracted.trim().length > 0) texts.push(extracted);
    }
    return texts.length > 0 ? texts.join("\n\n") : htmls.join("\n\n");
  }
  if (/text\/html/i.test(contentType)) return htmlToText(decodedBody);
  if (/text\/plain/i.test(contentType) || contentType === "text/plain") return decodedBody;
  return "";
}

function splitMimeParts(body: string, boundary: string): string[] {
  return body
    .split(`--${boundary}`)
    .slice(1)
    .map((part) => part.replace(/^\r?\n/, ""))
    .filter((part) => !part.startsWith("--") && part.trim().length > 0);
}

function splitHeaderBody(value: string): [string, string] {
  const match = value.match(/\r?\n\r?\n/);
  if (!match || typeof match.index !== "number") return ["", value];
  return [value.slice(0, match.index), value.slice(match.index + match[0].length)];
}

function parseHeaders(block: string): Map<string, string> {
  const headers = new Map<string, string>();
  let current = "";
  for (const line of block.split(/\r?\n/)) {
    if (/^[ \t]/.test(line) && current) {
      headers.set(current, `${headers.get(current) ?? ""} ${line.trim()}`.trim());
      continue;
    }
    const index = line.indexOf(":");
    if (index <= 0) continue;
    current = line.slice(0, index).toLowerCase();
    headers.set(current, line.slice(index + 1).trim());
  }
  return headers;
}

function decodeTransferEncoding(body: string, encoding: string): string {
  if (encoding === "base64") {
    try {
      const binary = atob(body.replace(/\s+/g, ""));
      const bytes = Uint8Array.from(binary, (char) => char.charCodeAt(0));
      return new TextDecoder().decode(bytes);
    } catch {
      return "";
    }
  }
  if (encoding === "quoted-printable") {
    return body
      .replace(/=\r?\n/g, "")
      .replace(/=([0-9A-F]{2})/gi, (_, hex: string) => String.fromCharCode(parseInt(hex, 16)));
  }
  return body;
}

function parseAuthenticationResults(value: string | undefined): { dmarc: string; spf: string; dkim: string } {
  const text = (value ?? "").toLowerCase();
  return {
    dmarc: text.match(/\bdmarc=(pass|fail|none|neutral|temperror|permerror)\b/)?.[1] ?? "unknown",
    spf: text.match(/\bspf=(pass|fail|none|neutral|temperror|permerror)\b/)?.[1] ?? "unknown",
    dkim: text.match(/\bdkim=(pass|fail|none|neutral|temperror|permerror)\b/)?.[1] ?? "unknown"
  };
}

function normalizeHeaderToken(value: string | undefined): string | null {
  if (typeof value !== "string") return null;
  const token = value.trim().replace(/^<|>$/g, "").toLowerCase();
  if (!token || token.length > 300 || /[\r\n]/.test(token)) return null;
  return token;
}

function normalizeEmailAddress(value: string): string | null {
  const trimmed = value.trim();
  const bracketed = trimmed.match(/<([^<>]+)>/);
  const candidate = (bracketed ? bracketed[1] : trimmed).trim().replace(/^mailto:/i, "");
  if (!candidate || candidate.length > 320 || /[\r\n]/.test(candidate)) return null;
  const match = candidate.match(/^([A-Z0-9._%+\-']+)@([A-Z0-9.-]+\.[A-Z]{2,})$/i);
  if (!match) return null;
  return `${match[1].toLowerCase()}@${match[2].toLowerCase()}`;
}

function safeHeaderValue(value: string): string {
  return value.replace(/[\r\n]+/g, " ").trim();
}

function sanitizeEmailText(value: string): string {
  return value
    .replace(/\u0000/g, "")
    .replace(/\r\n/g, "\n")
    .replace(/[ \t]+\n/g, "\n")
    .replace(/\n{4,}/g, "\n\n\n")
    .trim();
}

function htmlToText(html: string): string {
  return html
    .replace(/<!--[\s\S]*?-->/g, " ")
    .replace(/<script\b[^>]*>[\s\S]*?<\/script>/gi, " ")
    .replace(/<style\b[^>]*>[\s\S]*?<\/style>/gi, " ")
    .replace(/<[^>]+>/g, " ")
    .replace(/&nbsp;/gi, " ")
    .replace(/&amp;/gi, "&")
    .replace(/&lt;/gi, "<")
    .replace(/&gt;/gi, ">")
    .replace(/&quot;/gi, "\"")
    .replace(/&#39;/gi, "'");
}

function findTrackingLinks(value: string): string[] {
  const links = value.match(/https?:\/\/[^\s"'<>]+/gi) ?? [];
  return links
    .filter((link) => /[?&](utm_[^=]+|fbclid|gclid|mc_cid|mc_eid)=/i.test(link) || /\/(track|tracking|open|click)\b/i.test(link))
    .slice(0, 20);
}

async function sha256Hex(value: string, length: number): Promise<string> {
  const digest = await crypto.subtle.digest("SHA-256", new TextEncoder().encode(value));
  return [...new Uint8Array(digest)].map((byte) => byte.toString(16).padStart(2, "0")).join("").slice(0, length);
}

function objectValue(value: unknown): Record<string, unknown> | null {
  return typeof value === "object" && value !== null && !Array.isArray(value) ? (value as Record<string, unknown>) : null;
}

function authorized(request: Request, env: Env): boolean {
  const provided = request.headers.get("x-arcwell-edge-secret");
  if (!provided) return false;
  return [env.ARCWELL_EDGE_SECRET, env.ARCWELL_EDGE_NEXT_SECRET]
    .filter((secret): secret is string => typeof secret === "string" && secret.length > 0)
    .some((secret) => provided === secret);
}

function authorizedTelegram(request: Request, env: Env): boolean {
  const configured = env.TELEGRAM_WEBHOOK_SECRET;
  if (typeof configured !== "string" || configured.length === 0) return false;
  return request.headers.get("x-telegram-bot-api-secret-token") === configured;
}

async function enforceRateLimit(store: EdgeEventStore, env: Env, key: string, now: number): Promise<Response | null> {
  const windowSeconds = clampNumber(env.RATE_LIMIT_WINDOW_SECONDS, 1, 3600, 60);
  const maxEvents = clampNumber(env.RATE_LIMIT_MAX_EVENTS, 1, 10000, 120);
  const result = await store.checkRateLimit(key, now, windowSeconds, maxEvents);
  if (result.allowed) return null;
  return json(
    {
      error: "rate_limited",
      limit: result.limit,
      remaining: result.remaining,
      resetAt: result.resetAt
    },
    429,
    { "retry-after": String(Math.max(1, Math.ceil((result.resetAt - now) / 1000))) }
  );
}

async function readJson<T>(request: Request, env: Env): Promise<{ value: T } | { response: Response }> {
  const raw = await request.text();
  const maxBytes = Number(env.MAX_PAYLOAD_BYTES ?? "64000");
  if (new TextEncoder().encode(raw).byteLength > maxBytes) {
    return { response: json({ error: "payload_too_large" }, 413) };
  }
  try {
    return { value: JSON.parse(raw) as T };
  } catch {
    return { response: json({ error: "invalid_json" }, 400) };
  }
}

function validString(value: unknown, label: string, maxLength: number): { value: string } | { response: Response } {
  if (typeof value !== "string" || value.length === 0 || value.length > maxLength) {
    return { response: json({ error: `invalid_${label}` }, 400) };
  }
  return { value };
}

function clampNumber(value: unknown, min: number, max: number, fallback: number): number {
  const parsed =
    typeof value === "number" ? value : typeof value === "string" && value.length > 0 ? Number(value) : fallback;
  if (!Number.isFinite(parsed)) return fallback;
  return Math.max(min, Math.min(max, Math.trunc(parsed)));
}

function json(value: unknown, status = 200, headers: Record<string, string> = {}): Response {
  return new Response(JSON.stringify(value), {
    status,
    headers: {
      "content-type": "application/json; charset=utf-8",
      "cache-control": "no-store",
      ...headers
    }
  });
}

export class MemoryEdgeEventStore implements EdgeEventStore {
  private events = new Map<string, StoredEdgeEvent>();
  private rateLimits = new Map<string, { windowStart: number; count: number }>();

  async enqueue(event: StoredEdgeEvent): Promise<{ event: StoredEdgeEvent; duplicate: boolean }> {
    const existing = this.events.get(event.idempotencyKey);
    if (existing) return { event: existing, duplicate: true };
    this.events.set(event.idempotencyKey, { ...event });
    return { event, duplicate: false };
  }

  async checkRateLimit(key: string, now: number, windowSeconds: number, maxEvents: number): Promise<RateLimitResult> {
    const windowMs = windowSeconds * 1000;
    const existing = this.rateLimits.get(key);
    const current =
      existing && existing.windowStart + windowMs > now ? existing : { windowStart: now, count: 0 };
    current.count += 1;
    this.rateLimits.set(key, current);
    const remaining = Math.max(0, maxEvents - current.count);
    return {
      allowed: current.count <= maxEvents,
      limit: maxEvents,
      remaining,
      resetAt: current.windowStart + windowMs
    };
  }

  async lease(now: number, leaseSeconds: number): Promise<StoredEdgeEvent | null> {
    this.expire(now);
    const candidates = [...this.events.values()]
      .filter(
        (event) =>
          (event.status === "pending" && event.attempts < event.maxAttempts) ||
          (event.status === "failed" &&
            event.attempts < event.maxAttempts &&
            (event.leasedUntil === null || event.leasedUntil <= now)) ||
          (event.status === "leased" && event.leasedUntil !== null && event.leasedUntil <= now)
      )
      .sort((a, b) => a.receivedAt - b.receivedAt);
    const event = candidates[0];
    if (!event) return null;
    event.status = "leased";
    event.attempts += 1;
    event.leasedUntil = now + leaseSeconds * 1000;
    event.error = null;
    return { ...event };
  }

  async ack(idempotencyKey: string): Promise<StoredEdgeEvent | null> {
    const event = this.events.get(idempotencyKey);
    if (!event) return null;
    event.status = "acked";
    event.leasedUntil = null;
    return { ...event };
  }

  async nack(idempotencyKey: string, error: string, retrySeconds: number, now: number): Promise<StoredEdgeEvent | null> {
    const event = this.events.get(idempotencyKey);
    if (!event) return null;
    event.error = error;
    event.leasedUntil = now + retrySeconds * 1000;
    event.status = event.attempts >= event.maxAttempts ? "dead_lettered" : "failed";
    return { ...event };
  }

  async list(now: number, limit: number): Promise<StoredEdgeEvent[]> {
    this.expire(now);
    return [...this.events.values()]
      .sort((a, b) => a.receivedAt - b.receivedAt)
      .slice(0, limit)
      .map((event) => ({ ...event }));
  }

  private expire(now: number): void {
    for (const event of this.events.values()) {
      if (event.expiresAt <= now && event.status !== "acked" && event.status !== "dead_lettered") {
        event.status = "expired";
        event.leasedUntil = null;
      }
    }
  }
}

export class D1EdgeEventStore implements EdgeEventStore {
  constructor(private readonly db: D1Database) {}

  async enqueue(event: StoredEdgeEvent): Promise<{ event: StoredEdgeEvent; duplicate: boolean }> {
    await this.ensureSchema();
    const existing = await this.get(event.idempotencyKey);
    if (existing) return { event: existing, duplicate: true };
    await this.db
      .prepare(
        `INSERT INTO edge_events
          (source, idempotency_key, payload_json, status, received_at, expires_at, leased_until, attempts, max_attempts, error)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, NULL, ?7, ?8, NULL)`
      )
      .bind(
        event.source,
        event.idempotencyKey,
        JSON.stringify(event.payload),
        event.status,
        event.receivedAt,
        event.expiresAt,
        event.attempts,
        event.maxAttempts
      )
      .run();
    return { event, duplicate: false };
  }

  async checkRateLimit(key: string, now: number, windowSeconds: number, maxEvents: number): Promise<RateLimitResult> {
    await this.ensureSchema();
    const windowMs = windowSeconds * 1000;
    const existing = await this.db
      .prepare("SELECT key, window_start, count FROM edge_rate_limits WHERE key = ?1")
      .bind(key)
      .first<RateLimitRow>();
    const windowStart = existing && existing.window_start + windowMs > now ? existing.window_start : now;
    const count = existing && existing.window_start === windowStart ? existing.count + 1 : 1;
    await this.db
      .prepare(
        `INSERT INTO edge_rate_limits (key, window_start, count)
         VALUES (?1, ?2, ?3)
         ON CONFLICT(key) DO UPDATE SET window_start = excluded.window_start, count = excluded.count`
      )
      .bind(key, windowStart, count)
      .run();
    return {
      allowed: count <= maxEvents,
      limit: maxEvents,
      remaining: Math.max(0, maxEvents - count),
      resetAt: windowStart + windowMs
    };
  }

  async lease(now: number, leaseSeconds: number): Promise<StoredEdgeEvent | null> {
    await this.ensureSchema();
    await this.expire(now);
    const row = await this.db
      .prepare(
        `SELECT * FROM edge_events
         WHERE attempts < max_attempts
           AND (
             status = 'pending'
             OR (status = 'failed' AND (leased_until IS NULL OR leased_until <= ?1))
             OR (status = 'leased' AND leased_until IS NOT NULL AND leased_until <= ?1)
           )
         ORDER BY received_at ASC
         LIMIT 1`
      )
      .bind(now)
      .first<EdgeEventRow>();
    if (!row || row.attempts >= row.max_attempts) return null;
    const leasedUntil = now + leaseSeconds * 1000;
    await this.db
      .prepare(
        `UPDATE edge_events
         SET status = 'leased', attempts = attempts + 1, leased_until = ?2, error = NULL
         WHERE idempotency_key = ?1`
      )
      .bind(row.idempotency_key, leasedUntil)
      .run();
    return this.get(row.idempotency_key);
  }

  async ack(idempotencyKey: string): Promise<StoredEdgeEvent | null> {
    await this.ensureSchema();
    await this.db
      .prepare("UPDATE edge_events SET status = 'acked', leased_until = NULL WHERE idempotency_key = ?1")
      .bind(idempotencyKey)
      .run();
    return this.get(idempotencyKey);
  }

  async nack(idempotencyKey: string, error: string, retrySeconds: number, now: number): Promise<StoredEdgeEvent | null> {
    await this.ensureSchema();
    const event = await this.get(idempotencyKey);
    if (!event) return null;
    const status = event.attempts >= event.maxAttempts ? "dead_lettered" : "failed";
    const leasedUntil = status === "failed" ? now + retrySeconds * 1000 : null;
    await this.db
      .prepare("UPDATE edge_events SET status = ?2, leased_until = ?3, error = ?4 WHERE idempotency_key = ?1")
      .bind(idempotencyKey, status, leasedUntil, error.slice(0, 2000))
      .run();
    return this.get(idempotencyKey);
  }

  async list(now: number, limit: number): Promise<StoredEdgeEvent[]> {
    await this.ensureSchema();
    await this.expire(now);
    const result = await this.db.prepare("SELECT * FROM edge_events ORDER BY received_at ASC LIMIT ?1").bind(limit).all<EdgeEventRow>();
    return result.results.map(eventFromRow);
  }

  private async get(idempotencyKey: string): Promise<StoredEdgeEvent | null> {
    const row = await this.db
      .prepare("SELECT * FROM edge_events WHERE idempotency_key = ?1")
      .bind(idempotencyKey)
      .first<EdgeEventRow>();
    return row ? eventFromRow(row) : null;
  }

  private async expire(now: number): Promise<void> {
    await this.db
      .prepare(
        `UPDATE edge_events
         SET status = 'expired', leased_until = NULL
         WHERE expires_at <= ?1 AND status NOT IN ('acked', 'dead_lettered', 'expired')`
      )
      .bind(now)
      .run();
  }

  private async ensureSchema(): Promise<void> {
    await this.db
      .prepare(
        `CREATE TABLE IF NOT EXISTS edge_events (
          source TEXT NOT NULL,
          idempotency_key TEXT PRIMARY KEY,
          payload_json TEXT NOT NULL,
          status TEXT NOT NULL,
          received_at INTEGER NOT NULL,
          expires_at INTEGER NOT NULL,
          leased_until INTEGER,
          attempts INTEGER NOT NULL DEFAULT 0,
          max_attempts INTEGER NOT NULL DEFAULT 3,
          error TEXT
        )`
      )
      .run();
    await this.db
      .prepare(
        `CREATE TABLE IF NOT EXISTS edge_rate_limits (
          key TEXT PRIMARY KEY,
          window_start INTEGER NOT NULL,
          count INTEGER NOT NULL
        )`
      )
      .run();
  }
}

type RateLimitRow = {
  key: string;
  window_start: number;
  count: number;
};

type EdgeEventRow = {
  source: string;
  idempotency_key: string;
  payload_json: string;
  status: StoredEdgeEvent["status"];
  received_at: number;
  expires_at: number;
  leased_until: number | null;
  attempts: number;
  max_attempts: number;
  error: string | null;
};

function eventFromRow(row: EdgeEventRow): StoredEdgeEvent {
  return {
    source: row.source,
    idempotencyKey: row.idempotency_key,
    payload: JSON.parse(row.payload_json) as unknown,
    status: row.status,
    receivedAt: row.received_at,
    expiresAt: row.expires_at,
    leasedUntil: row.leased_until,
    attempts: row.attempts,
    maxAttempts: row.max_attempts,
    error: row.error
  };
}
