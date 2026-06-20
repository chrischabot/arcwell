export interface Env {
  AGENT_EDGE_SECRET: string;
  MAX_PAYLOAD_BYTES?: string;
}

type EdgeEventInput = {
  source?: unknown;
  idempotencyKey?: unknown;
  payload?: unknown;
  maxAgeSeconds?: unknown;
};

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    const url = new URL(request.url);
    if (request.method === "GET" && url.pathname === "/health") {
      return json({ ok: true, service: "agent-edge-inbox" });
    }
    if (request.method !== "POST" || url.pathname !== "/events") {
      return json({ error: "not_found" }, 404);
    }
    if (request.headers.get("x-agent-edge-secret") !== env.AGENT_EDGE_SECRET) {
      return json({ error: "unauthorized" }, 401);
    }
    const raw = await request.text();
    const maxBytes = Number(env.MAX_PAYLOAD_BYTES ?? "64000");
    if (new TextEncoder().encode(raw).byteLength > maxBytes) {
      return json({ error: "payload_too_large" }, 413);
    }
    let input: EdgeEventInput;
    try {
      input = JSON.parse(raw) as EdgeEventInput;
    } catch {
      return json({ error: "invalid_json" }, 400);
    }
    if (typeof input.source !== "string" || input.source.length === 0 || input.source.length > 200) {
      return json({ error: "invalid_source" }, 400);
    }
    if (
      typeof input.idempotencyKey !== "string" ||
      input.idempotencyKey.length === 0 ||
      input.idempotencyKey.length > 200
    ) {
      return json({ error: "invalid_idempotency_key" }, 400);
    }
    const requestedTtl =
      typeof input.maxAgeSeconds === "number" && Number.isFinite(input.maxAgeSeconds)
        ? input.maxAgeSeconds
        : 3600;
    const maxAgeSeconds = Math.max(60, Math.min(86400, Math.trunc(requestedTtl)));
    return json({
      accepted: true,
      source: input.source,
      idempotencyKey: input.idempotencyKey,
      maxAgeSeconds,
      payload: input.payload ?? null
    });
  }
};

function json(value: unknown, status = 200): Response {
  return new Response(JSON.stringify(value), {
    status,
    headers: {
      "content-type": "application/json; charset=utf-8",
      "cache-control": "no-store"
    }
  });
}
