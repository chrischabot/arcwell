#!/usr/bin/env node

import dgram from "node:dgram";
import os from "node:os";
import { spawn } from "node:child_process";
import { pathToFileURL } from "node:url";
import { URL } from "node:url";

const SSDP_ADDR = "239.255.255.250";
const SSDP_PORT = 1900;
const LUMIN_UDP_PORT = 23456;
const DEFAULT_HTTP_TIMEOUT_MS = 5000;
const DEFAULT_MAX_BYTES = 2 * 1024 * 1024;

const SEARCH_TARGETS = [
  "urn:schemas-upnp-org:device:MediaRenderer:1",
  "urn:av-openhome-org:service:Product:1",
  "urn:av-openhome-org:service:Playlist:1",
  "urn:av-openhome-org:service:Volume:1",
  "urn:av-openhome-org:service:Info:1",
  "urn:av-openhome-org:service:Time:1",
  "urn:schemas-upnp-org:service:AVTransport:1",
  "urn:schemas-upnp-org:service:RenderingControl:1",
  "ssdp:all",
];

const UDP_COMMANDS = {
  play: [0x21, 0xde],
  pause: [0x22, 0xdd],
  previous: [0x27, 0xd8],
  next: [0x28, 0xd7],
  "volume-up": [0x14, 0xeb],
  "volume-down": [0x15, 0xea],
  mute: [0x18, 0xe7],
  unmute: [0x4e, 0xb1],
  "repeat-on": [0x32, 0xcd],
  "repeat-off": [0x34, 0xcb],
  "shuffle-on": [0x33, 0xcc],
  "shuffle-off": [0x4f, 0xb0],
  standby: [0x70, 0x8f],
  wake: [0x71, 0x8e],
};

function usage() {
  console.log(`Usage:
  lumin.mjs discover [--timeout-ms 3500] [--bind-address IP] [--json]
  lumin.mjs bonjour [--timeout-ms 5000] [--filter TEXT] [--json]
  lumin.mjs spotify-info --host <ip-or-host> [--port 43669] [--json]
  lumin.mjs inspect --location <device-xml-url> [--json]
  lumin.mjs services --location <device-xml-url> [--json]
  lumin.mjs actions --location <device-xml-url> [--service TEXT] [--json]
  lumin.mjs status --location <device-xml-url> [--json]
  lumin.mjs sources --location <device-xml-url> [--json]
  lumin.mjs power --location <device-xml-url> (--get | --standby true|false) [--confirm-standby] [--dry-run] [--json]
  lumin.mjs select-source --location <device-xml-url> (--index N | --system-name NAME) --confirm-source-select [--dry-run] [--json]
  lumin.mjs volume --location <device-xml-url> (--get | --set N | --mute true|false) [--confirm-volume] [--dry-run] [--json]
  lumin.mjs playback --location <device-xml-url> --action play|pause|stop|next|previous [--transport soap|udp] [--host HOST] [--dry-run] [--json]
  lumin.mjs playlist --location <device-xml-url> (--state | --read --id ID | --read-list --ids IDS | --repeat true|false | --shuffle true|false | --delete-id ID | --clear | --insert-uri URI --after-id ID [--metadata XML]) [--confirm-playlist-write] [--dry-run] [--json]
  lumin.mjs udp --host <ip-or-host> --command <name> [--dry-run] [--confirm-standby] [--json]
  lumin.mjs soap --location <device-xml-url> --service <service-type-substring> --action <action> [--arg Name=Value] [--dry-run] [--json]

Environment:
  LUMIN_P1_HOST       Default host for UDP commands.
  LUMIN_P1_LOCATION   Default UPnP device description URL.

UDP commands:
  ${Object.keys(UDP_COMMANDS).join(", ")}`);
}

function die(message, code = 1) {
  console.error(`Error: ${message}`);
  process.exit(code);
}

function asInt(value, fallback) {
  if (value == null || value === true) return fallback;
  const parsed = Number.parseInt(String(value), 10);
  return Number.isFinite(parsed) ? parsed : fallback;
}

function asBoolString(value, name) {
  if (value === true || value == null) throw new Error(`missing value for ${name}; use true or false`);
  const normalized = String(value).toLowerCase();
  if (["true", "1", "yes", "on"].includes(normalized)) return "true";
  if (["false", "0", "no", "off"].includes(normalized)) return "false";
  throw new Error(`invalid boolean for ${name}: ${value}`);
}

function parseHeaders(text) {
  const headers = {};
  for (const line of text.split(/\r?\n/)) {
    const idx = line.indexOf(":");
    if (idx > 0) headers[line.slice(0, idx).toLowerCase()] = line.slice(idx + 1).trim();
  }
  return headers;
}

async function runCommand(command, args, timeoutMs, input = null) {
  return await new Promise((resolve) => {
    const child = spawn(command, args, { stdio: ["pipe", "pipe", "pipe"] });
    let stdout = "";
    let stderr = "";
    let settled = false;
    let timedOut = false;
    let terminateTimer = null;
    let killTimer = null;
    function finish(result) {
      if (settled) return;
      settled = true;
      clearTimeout(timer);
      clearTimeout(terminateTimer);
      clearTimeout(killTimer);
      resolve(result);
    }
    const timer = setTimeout(() => {
      timedOut = true;
      child.kill("SIGTERM");
      terminateTimer = setTimeout(() => {
        child.kill("SIGKILL");
      }, 500);
      killTimer = setTimeout(() => {
        finish({ ok: false, stdout, stderr, code: null, timedOut });
      }, 1500);
    }, timeoutMs);
    child.stdout.on("data", (chunk) => { stdout += chunk.toString(); });
    child.stderr.on("data", (chunk) => { stderr += chunk.toString(); });
    child.on("error", (error) => {
      finish({ ok: false, stdout, stderr: error.message, timedOut });
    });
    child.on("close", (code) => {
      finish({ ok: !timedOut && (code === 0 || code === null || code === 130), stdout, stderr, code, timedOut });
    });
    if (input == null) child.stdin.end();
    else child.stdin.end(input);
  });
}

function parseDnsSdBrowse(output, service) {
  const rows = [];
  for (const line of output.split(/\r?\n/)) {
    const match = line.match(/\s+(Add|Rmv)\s+\S+\s+\d+\s+(\S+)\s+(\S+)\s+(.+)$/);
    if (!match || match[1] !== "Add") continue;
    rows.push({
      service,
      domain: match[2],
      serviceType: match[3],
      name: match[4].trim(),
      raw: line,
    });
  }
  return rows;
}

async function bonjour(options) {
  const timeoutMs = asInt(options["timeout-ms"], 5000);
  const services = ["_airplay._tcp", "_raop._tcp", "_spotify-connect._tcp", "_http._tcp"];
  const filter = options.filter ? String(options.filter).toLowerCase() : null;
  const rows = [];
  for (const service of services) {
    const result = await runCommand("dns-sd", ["-B", service, "local"], timeoutMs);
    if (!result.ok && result.stderr) {
      rows.push({ service, error: result.stderr });
      continue;
    }
    for (const row of parseDnsSdBrowse(result.stdout, service)) {
      if (!filter || row.name.toLowerCase().includes(filter)) rows.push(row);
    }
  }
  return rows;
}

function localIpv4Addresses() {
  const addresses = [];
  for (const entries of Object.values(os.networkInterfaces())) {
    for (const entry of entries || []) {
      if (entry.family === "IPv4" && !entry.internal) addresses.push(entry.address);
    }
  }
  return addresses;
}

async function discover(options) {
  const timeoutMs = asInt(options["timeout-ms"], 3500);
  const targets = options.st ? parseRepeatedArgs(options, "st").map(String) : SEARCH_TARGETS;
  const bindAddresses = options["bind-address"]
    ? parseRepeatedArgs(options, "bind-address").map(String)
    : [undefined, ...localIpv4Addresses()];
  const all = [];
  for (const bindAddress of bindAddresses) {
    try {
      all.push(...await discoverOnSocket({ targets, timeoutMs, bindAddress }));
    } catch (error) {
      all.push({
        error: error?.message || String(error),
        bindAddress: bindAddress || "default",
      });
    }
  }
  return dedupeDiscoveries(all);
}

async function discoverOnSocket({ targets, timeoutMs, bindAddress }) {
  const seen = new Map();
  const socket = dgram.createSocket({ type: "udp4", reuseAddr: true });

  await new Promise((resolve, reject) => {
    socket.on("error", reject);
    socket.on("message", (msg, rinfo) => {
      const headers = parseHeaders(msg.toString());
      const key = `${headers.usn || ""}|${headers.location || ""}|${headers.st || ""}`;
      if (!seen.has(key)) {
        seen.set(key, {
          from: `${rinfo.address}:${rinfo.port}`,
          address: rinfo.address,
          bindAddress: bindAddress || "default",
          st: headers.st || null,
          usn: headers.usn || null,
          server: headers.server || null,
          location: headers.location || null,
        });
      }
    });
    socket.bind(0, bindAddress, () => {
      socket.setBroadcast(true);
      if (bindAddress) {
        try {
          socket.setMulticastInterface(bindAddress);
        } catch {
          // Some systems reject setting a multicast interface even when bind works.
        }
      }
      for (const st of targets) {
        const req = [
          "M-SEARCH * HTTP/1.1",
          `HOST: ${SSDP_ADDR}:${SSDP_PORT}`,
          'MAN: "ssdp:discover"',
          "MX: 2",
          `ST: ${st}`,
          "",
          "",
        ].join("\r\n");
        socket.send(Buffer.from(req), SSDP_PORT, SSDP_ADDR);
      }
      setTimeout(resolve, timeoutMs);
    });
  }).finally(() => socket.close());

  return [...seen.values()];
}

function dedupeDiscoveries(rows) {
  const seen = new Map();
  for (const row of rows) {
    const key = row.error
      ? `error|${row.bindAddress}|${row.error}`
      : `${row.usn || ""}|${row.location || ""}|${row.st || ""}`;
    if (!seen.has(key)) seen.set(key, row);
  }
  return [...seen.values()].sort((a, b) =>
    `${a.location || ""}${a.st || ""}${a.error || ""}`.localeCompare(`${b.location || ""}${b.st || ""}${b.error || ""}`),
  );
}

function ensureHttpUrl(value, label) {
  let url;
  try {
    url = new URL(value);
  } catch {
    throw new Error(`${label} is not a valid URL: ${value}`);
  }
  if (!["http:", "https:"].includes(url.protocol)) {
    throw new Error(`${label} must use http or https`);
  }
  return url;
}

async function fetchText(urlValue, options = {}) {
  const url = ensureHttpUrl(urlValue, "URL");
  const timeoutMs = asInt(options["http-timeout-ms"], DEFAULT_HTTP_TIMEOUT_MS);
  const maxBytes = asInt(options["max-bytes"], DEFAULT_MAX_BYTES);
  try {
    return await fetchTextNative(url, timeoutMs, maxBytes);
  } catch (error) {
    if (options["no-curl-fallback"]) throw error;
    return fetchTextCurl(url, timeoutMs, maxBytes, error);
  }
}

async function fetchTextNative(url, timeoutMs, maxBytes) {
  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), timeoutMs);
  try {
    const res = await fetch(url, { redirect: "follow", signal: controller.signal });
    if (!res.ok) throw new Error(`fetch failed for ${url.href}: ${res.status} ${res.statusText}`);
    const bytes = Buffer.from(await res.arrayBuffer());
    if (bytes.length > maxBytes) {
      throw new Error(`fetch response too large for ${url.href}: ${bytes.length} bytes > ${maxBytes}`);
    }
    return { text: bytes.toString("utf8"), finalUrl: res.url };
  } finally {
    clearTimeout(timeout);
  }
}

async function fetchTextCurl(url, timeoutMs, maxBytes, nativeError) {
  const seconds = Math.max(1, Math.ceil(timeoutMs / 1000));
  const result = await runCommand("curl", [
    "--silent",
    "--show-error",
    "--fail-with-body",
    "--location",
    "--max-time",
    String(seconds),
    "--max-redirs",
    "3",
    "--max-filesize",
    String(maxBytes),
    url.href,
  ], timeoutMs + 1000);
  if (!result.ok) {
    throw new Error(`fetch failed (${nativeError?.message || nativeError}); curl fallback failed: ${result.stderr || result.stdout || result.code}`);
  }
  const bytes = Buffer.from(result.stdout, "utf8");
  if (bytes.length > maxBytes) {
    throw new Error(`fetch response too large for ${url.href}: ${bytes.length} bytes > ${maxBytes}`);
  }
  return { text: result.stdout, finalUrl: url.href, transport: "curl-fallback" };
}

async function postText(urlValue, headers, body, options = {}) {
  const url = ensureHttpUrl(urlValue, "URL");
  const timeoutMs = asInt(options["http-timeout-ms"], DEFAULT_HTTP_TIMEOUT_MS);
  const maxBytes = asInt(options["max-bytes"], DEFAULT_MAX_BYTES);
  try {
    return await postTextNative(url, headers, body, timeoutMs, maxBytes);
  } catch (error) {
    if (options["no-curl-fallback"]) throw error;
    return postTextCurl(url, headers, body, timeoutMs, maxBytes, error);
  }
}

async function postTextNative(url, headers, body, timeoutMs, maxBytes) {
  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), timeoutMs);
  try {
    const res = await fetch(url, {
      method: "POST",
      headers,
      body,
      signal: controller.signal,
    });
    const bytes = Buffer.from(await res.arrayBuffer());
    if (bytes.length > maxBytes) {
      throw new Error(`SOAP response too large for ${url.href}: ${bytes.length} bytes > ${maxBytes}`);
    }
    return {
      status: res.status,
      ok: res.ok,
      text: bytes.toString("utf8"),
      finalUrl: res.url,
    };
  } finally {
    clearTimeout(timeout);
  }
}

async function postTextCurl(url, headers, body, timeoutMs, maxBytes, nativeError) {
  const seconds = Math.max(1, Math.ceil(timeoutMs / 1000));
  const args = [
    "--silent",
    "--show-error",
    "--fail-with-body",
    "--location",
    "--request",
    "POST",
    "--max-time",
    String(seconds),
    "--max-redirs",
    "3",
    "--max-filesize",
    String(maxBytes),
  ];
  for (const [key, value] of Object.entries(headers)) {
    args.push("--header", `${key}: ${value}`);
  }
  args.push("--data-binary", "@-", url.href);
  const result = await runCommand("curl", args, timeoutMs + 1000, body);
  if (!result.ok) {
    throw new Error(`POST failed (${nativeError?.message || nativeError}); curl fallback failed: ${result.stderr || result.stdout || result.code}`);
  }
  const bytes = Buffer.from(result.stdout, "utf8");
  if (bytes.length > maxBytes) {
    throw new Error(`SOAP response too large for ${url.href}: ${bytes.length} bytes > ${maxBytes}`);
  }
  return {
    status: 200,
    ok: true,
    text: result.stdout,
    finalUrl: url.href,
    transport: "curl-fallback",
  };
}

function textOf(xml, tag) {
  const match = xml.match(new RegExp(`<(?:[^:>]+:)?${tag}\\b[^>]*>([\\s\\S]*?)</(?:[^:>]+:)?${tag}>`, "i"));
  return match ? decodeXml(match[1].trim()) : null;
}

function decodeXml(value) {
  return String(value)
    .replaceAll("&lt;", "<")
    .replaceAll("&gt;", ">")
    .replaceAll("&quot;", '"')
    .replaceAll("&apos;", "'")
    .replaceAll("&amp;", "&");
}

function serviceBlocks(xml) {
  return [...xml.matchAll(/<service\b[^>]*>([\s\S]*?)<\/service>/gi)].map((m) => m[1]);
}

function actionBlocks(xml) {
  return [...xml.matchAll(/<action\b[^>]*>([\s\S]*?)<\/action>/gi)].map((m) => m[1]);
}

function argumentBlocks(xml) {
  return [...xml.matchAll(/<argument\b[^>]*>([\s\S]*?)<\/argument>/gi)].map((m) => m[1]);
}

function parseDevice(xml, location) {
  const services = serviceBlocks(xml).map((block) => ({
    serviceType: textOf(block, "serviceType"),
    serviceId: textOf(block, "serviceId"),
    controlURL: textOf(block, "controlURL"),
    eventSubURL: textOf(block, "eventSubURL"),
    SCPDURL: textOf(block, "SCPDURL"),
  }));
  return {
    location,
    friendlyName: textOf(xml, "friendlyName"),
    manufacturer: textOf(xml, "manufacturer"),
    modelName: textOf(xml, "modelName"),
    modelNumber: textOf(xml, "modelNumber"),
    UDN: textOf(xml, "UDN"),
    services,
  };
}

function parseScpd(xml) {
  return actionBlocks(xml).map((block) => ({
    name: textOf(block, "name"),
    arguments: argumentBlocks(block).map((argBlock) => ({
      name: textOf(argBlock, "name"),
      direction: textOf(argBlock, "direction"),
      relatedStateVariable: textOf(argBlock, "relatedStateVariable"),
    })).filter((arg) => arg.name),
  })).filter((action) => action.name);
}

function resolveAgainst(base, maybePath, options = {}) {
  if (!maybePath) return null;
  const resolved = new URL(maybePath, base);
  const baseUrl = new URL(base);
  if (!options["allow-cross-host-service-urls"] && resolved.host !== baseUrl.host) {
    throw new Error(`refusing cross-host service URL ${resolved.href}; pass --allow-cross-host-service-urls if this device descriptor is trusted`);
  }
  return resolved.href;
}

function print(data, json) {
  if (json) {
    console.log(JSON.stringify(data, null, 2));
    return;
  }
  if (Array.isArray(data)) {
    if (data.length === 0) {
      console.log("No devices found.");
      return;
    }
    for (const item of data) {
      if (item.error) {
        console.log(`error on ${item.bindAddress}: ${item.error}`);
        continue;
      }
      console.log(`${item.address || item.from} ${item.st || ""}`);
      if (item.location) console.log(`  location: ${item.location}`);
      if (item.usn) console.log(`  usn: ${item.usn}`);
      if (item.server) console.log(`  server: ${item.server}`);
    }
    return;
  }
  console.log(JSON.stringify(data, null, 2));
}

function makeUdpPacket(command) {
  const bytes = UDP_COMMANDS[command];
  if (!bytes) throw new Error(`unknown UDP command: ${command}`);
  return Buffer.from([
    0x55, 0x00, 0x00, 0x40,
    0x00, 0x08, 0x00, 0x04,
    0x00, 0x04, 0x00, 0xd8,
    0x2a, bytes[0], bytes[1], 0xaa,
  ]);
}

async function sendUdp(options) {
  const host = options.host || process.env.LUMIN_P1_HOST;
  const command = options.command;
  if (!host) throw new Error("missing --host or LUMIN_P1_HOST");
  if (!command) throw new Error("missing --command");
  if (command === "standby" && !options["confirm-standby"]) {
    throw new Error("standby requires --confirm-standby");
  }
  const packet = makeUdpPacket(command);
  const result = {
    host,
    port: LUMIN_UDP_PORT,
    command,
    packetHex: packet.toString("hex").match(/.{1,2}/g).join(" "),
    dryRun: Boolean(options["dry-run"]),
  };
  if (options["dry-run"]) return result;

  await new Promise((resolve, reject) => {
    const socket = dgram.createSocket("udp4");
    socket.on("error", reject);
    socket.send(packet, LUMIN_UDP_PORT, host, (error) => {
      socket.close();
      if (error) reject(error);
      else resolve();
    });
  });
  return { ...result, sent: true };
}

async function inspect(options) {
  const location = options.location || process.env.LUMIN_P1_LOCATION;
  if (!location) throw new Error("missing --location or LUMIN_P1_LOCATION");
  const { text, finalUrl } = await fetchText(location, options);
  const device = parseDevice(text, finalUrl);
  device.services = device.services.map((service) => ({
    ...service,
    controlURLResolved: resolveAgainst(finalUrl, service.controlURL, options),
    eventSubURLResolved: resolveAgainst(finalUrl, service.eventSubURL, options),
    SCPDURLResolved: resolveAgainst(finalUrl, service.SCPDURL, options),
  }));
  return device;
}

async function spotifyInfo(options) {
  const host = options.host || process.env.LUMIN_P1_HOST;
  if (!host) throw new Error("spotify-info requires --host or LUMIN_P1_HOST");
  const port = asInt(options.port, 43669);
  const url = `http://${host}:${port}/zc?action=getInfo&version=2`;
  const { text } = await fetchText(url, options);
  try {
    return JSON.parse(text);
  } catch {
    return { raw: text };
  }
}

function parseRepeatedArgs(options, name) {
  const value = options[name];
  if (value == null) return [];
  return Array.isArray(value) ? value : [value];
}

function parseArgsMulti(argv) {
  const out = { _: [] };
  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (!arg.startsWith("--")) {
      out._.push(arg);
      continue;
    }
    const key = arg.slice(2);
    const next = argv[i + 1];
    const value = next == null || next.startsWith("--") ? true : next;
    if (out[key] == null) out[key] = value;
    else if (Array.isArray(out[key])) out[key].push(value);
    else out[key] = [out[key], value];
    if (value !== true) i += 1;
  }
  return out;
}

function soapEnvelope(serviceType, action, args) {
  const body = Object.entries(args)
    .map(([key, value]) => `<${key}>${escapeXml(value)}</${key}>`)
    .join("");
  return `<?xml version="1.0" encoding="utf-8"?>` +
    `<s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/" ` +
    `s:encodingStyle="http://schemas.xmlsoap.org/soap/encoding/">` +
    `<s:Body><u:${action} xmlns:u="${serviceType}">${body}</u:${action}></s:Body></s:Envelope>`;
}

function escapeXml(value) {
  return String(value)
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&apos;");
}

function findService(device, needles) {
  const values = Array.isArray(needles) ? needles : [needles];
  for (const needle of values.map((value) => String(value).toLowerCase())) {
    const service = device.services.find((candidate) =>
      `${candidate.serviceType || ""} ${candidate.serviceId || ""}`.toLowerCase().includes(needle),
    );
    if (service) return service;
  }
  return null;
}

async function serviceActions(service, options) {
  if (!service?.SCPDURLResolved) return [];
  const { text } = await fetchText(service.SCPDURLResolved, options);
  return parseScpd(text);
}

async function actions(options) {
  const device = await inspect(options);
  const serviceNeedle = options.service ? String(options.service).toLowerCase() : null;
  const rows = [];
  for (const service of device.services) {
    if (serviceNeedle && !`${service.serviceType || ""} ${service.serviceId || ""}`.toLowerCase().includes(serviceNeedle)) {
      continue;
    }
    rows.push({
      serviceType: service.serviceType,
      serviceId: service.serviceId,
      controlURL: service.controlURLResolved,
      SCPDURL: service.SCPDURLResolved,
      actions: await serviceActions(service, options),
    });
  }
  return rows;
}

function actionByName(actionsList, name) {
  return actionsList.find((action) => action.name === name);
}

function defaultInputArgs(action, overrides = {}) {
  const args = {};
  for (const arg of action?.arguments || []) {
    if (arg.direction !== "in") continue;
    if (Object.prototype.hasOwnProperty.call(overrides, arg.name)) {
      args[arg.name] = overrides[arg.name];
    } else if (arg.name === "InstanceID") {
      args[arg.name] = "0";
    } else if (arg.name === "Channel") {
      args[arg.name] = "Master";
    } else if (arg.name === "Speed") {
      args[arg.name] = "1";
    }
  }
  return args;
}

async function callServiceAction({ location, serviceNeedle, actionName, args = {}, options }) {
  const device = await inspect({ ...options, location });
  const service = findService(device, serviceNeedle);
  if (!service) throw new Error(`service not found: ${serviceNeedle}`);
  const actionList = await serviceActions(service, options);
  const action = actionByName(actionList, actionName);
  if (!action) throw new Error(`action ${actionName} not found on ${service.serviceType || service.serviceId}`);
  return callSoap(service, action.name, defaultInputArgs(action, args), options);
}

async function callSoap(service, action, args, options) {
  if (!service.controlURLResolved || !service.serviceType) {
    throw new Error("matched service has no control URL or service type");
  }
  const body = soapEnvelope(service.serviceType, action, args);
  const request = {
    url: service.controlURLResolved,
    serviceType: service.serviceType,
    action,
    args,
    body,
    dryRun: Boolean(options["dry-run"]),
  };
  if (options["dry-run"]) return request;

  const response = await postText(service.controlURLResolved, {
    "Content-Type": 'text/xml; charset="utf-8"',
    SOAPAction: `"${service.serviceType}#${action}"`,
  }, body, options);
  return {
    ...request,
    status: response.status,
    ok: response.ok,
    response: response.text,
    transport: response.transport,
  };
}

async function soap(options) {
  const device = await inspect(options);
  const serviceNeedle = String(options.service || "").toLowerCase();
  const action = options.action;
  if (!serviceNeedle) throw new Error("missing --service");
  if (!action) throw new Error("missing --action");
  const service = findService(device, serviceNeedle);
  if (!service) throw new Error(`service not found: ${options.service}`);

  const args = {};
  for (const pair of parseRepeatedArgs(options, "arg")) {
    const idx = String(pair).indexOf("=");
    if (idx <= 0) throw new Error(`invalid --arg, expected Name=Value: ${pair}`);
    args[String(pair).slice(0, idx)] = String(pair).slice(idx + 1);
  }
  return callSoap(service, action, args, options);
}

async function safeRead(device, serviceNeedle, actionName, options, args = {}) {
  const service = findService(device, serviceNeedle);
  if (!service) return { action: actionName, skipped: "service not found" };
  const actionList = await serviceActions(service, options);
  const action = actionByName(actionList, actionName);
  if (!action) return { action: actionName, skipped: "action not found" };
  const callArgs = defaultInputArgs(action, args);
  if ([...Object.keys(callArgs)].some((name) => callArgs[name] == null)) {
    return { action: actionName, skipped: "missing required input argument" };
  }
  const result = await callSoap(service, actionName, callArgs, options);
  return {
    serviceType: service.serviceType,
    action: actionName,
    status: result.status,
    ok: result.ok,
    response: result.response,
  };
}

async function status(options) {
  const device = await inspect(options);
  const candidates = [
    ["Product", "Standby"],
    ["Product", "SourceXml"],
    ["Product", "SourceIndex"],
    ["Product", "Source"],
    ["Playlist", "TransportState"],
    ["Playlist", "Id"],
    ["Playlist", "Repeat"],
    ["Playlist", "Shuffle"],
    ["Volume", "Volume"],
    ["Volume", "Mute"],
    ["Info", "Details"],
    ["Time", "Seconds"],
    ["AVTransport", "GetTransportInfo"],
    ["RenderingControl", "GetVolume"],
    ["RenderingControl", "GetMute"],
  ];
  const reads = [];
  for (const [serviceNeedle, actionName] of candidates) {
    try {
      reads.push(await safeRead(device, serviceNeedle, actionName, options));
    } catch (error) {
      reads.push({ serviceNeedle, action: actionName, error: error?.message || String(error) });
    }
  }
  return { device: summarizeDevice(device), reads };
}

function summarizeDevice(device) {
  return {
    location: device.location,
    friendlyName: device.friendlyName,
    manufacturer: device.manufacturer,
    modelName: device.modelName,
    modelNumber: device.modelNumber,
    UDN: device.UDN,
  };
}

function parseSourceXml(xmlValue) {
  const xml = decodeXml(xmlValue || "");
  return [...xml.matchAll(/<Source\b([^>]*)\/?>/gi)].map((match, index) => {
    const attrs = {};
    for (const attr of match[1].matchAll(/([A-Za-z0-9_:-]+)="([^"]*)"/g)) {
      attrs[attr[1]] = decodeXml(attr[2]);
    }
    return {
      index,
      name: attrs.Name || attrs.name || null,
      type: attrs.Type || attrs.type || null,
      visible: attrs.Visible || attrs.visible || null,
      systemName: attrs.SystemName || attrs.systemName || null,
      rawAttributes: attrs,
    };
  });
}

function firstTagValue(xml, tag) {
  return textOf(xml, tag);
}

async function sources(options) {
  const location = options.location || process.env.LUMIN_P1_LOCATION;
  const result = await callServiceAction({
    location,
    serviceNeedle: ["Product"],
    actionName: "SourceXml",
    options,
  });
  const sourceXml = firstTagValue(result.response || "", "Value")
    || firstTagValue(result.response || "", "SourceXml")
    || result.response;
  return {
    status: result.status,
    ok: result.ok,
    sources: parseSourceXml(sourceXml),
    rawSourceXml: sourceXml,
  };
}

async function selectSource(options) {
  const location = options.location || process.env.LUMIN_P1_LOCATION;
  if (!options["confirm-source-select"]) throw new Error("source selection requires --confirm-source-select");
  if (options.index == null && options["system-name"] == null) {
    throw new Error("source selection requires --index or --system-name");
  }
  const device = await inspect(options);
  const service = findService(device, "Product");
  if (!service) throw new Error("Product service not found");
  const actionList = await serviceActions(service, options);
  if (options["system-name"] != null && actionByName(actionList, "SetSource")) {
    return callSoap(service, "SetSource", { SystemName: String(options["system-name"]) }, options);
  }
  if (options.index != null && actionByName(actionList, "SetSourceIndex")) {
    return callSoap(service, "SetSourceIndex", { Value: String(asInt(options.index, NaN)) }, options);
  }
  throw new Error("no supported source selection action found; inspect actions first");
}

async function power(options) {
  const location = options.location || process.env.LUMIN_P1_LOCATION;
  if (options.get) {
    return callServiceAction({ location, serviceNeedle: "Product", actionName: "Standby", options });
  }
  if (options.standby != null) {
    if (!options["confirm-standby"]) throw new Error("standby write requires --confirm-standby");
    return callServiceAction({
      location,
      serviceNeedle: "Product",
      actionName: "SetStandby",
      args: { Value: asBoolString(options.standby, "--standby") },
      options,
    });
  }
  throw new Error("power requires --get or --standby true|false");
}

async function volume(options) {
  const location = options.location || process.env.LUMIN_P1_LOCATION;
  if (options.get) {
    return {
      volume: await callServiceAction({ location, serviceNeedle: ["Volume", "RenderingControl"], actionName: "Volume", options }).catch(async () =>
        callServiceAction({ location, serviceNeedle: ["RenderingControl"], actionName: "GetVolume", args: { InstanceID: "0", Channel: "Master" }, options })),
      mute: await callServiceAction({ location, serviceNeedle: ["Volume", "RenderingControl"], actionName: "Mute", options }).catch(async () =>
        callServiceAction({ location, serviceNeedle: ["RenderingControl"], actionName: "GetMute", args: { InstanceID: "0", Channel: "Master" }, options })),
    };
  }
  if (options.set != null) {
    if (!options["confirm-volume"]) throw new Error("absolute volume set requires --confirm-volume");
    const value = asInt(options.set, NaN);
    if (!Number.isInteger(value) || value < 0 || value > 100) throw new Error("--set must be an integer from 0 to 100");
    return callServiceAction({ location, serviceNeedle: ["Volume", "RenderingControl"], actionName: "SetVolume", args: { Value: String(value), DesiredVolume: String(value), InstanceID: "0", Channel: "Master" }, options });
  }
  if (options.mute != null) {
    const value = asBoolString(options.mute, "--mute");
    return callServiceAction({ location, serviceNeedle: ["Volume", "RenderingControl"], actionName: "SetMute", args: { Value: value, DesiredMute: value, InstanceID: "0", Channel: "Master" }, options });
  }
  throw new Error("volume requires --get, --set, or --mute");
}

async function playback(options) {
  const action = options.action;
  if (!["play", "pause", "stop", "next", "previous"].includes(action)) {
    throw new Error("playback --action must be play, pause, stop, next, or previous");
  }
  if (options.transport === "udp") {
    if (action === "stop") throw new Error("LUMIN UDP shortcut protocol does not define stop");
    return sendUdp({ ...options, command: action });
  }
  const actionName = { play: "Play", pause: "Pause", stop: "Stop", next: "Next", previous: "Previous" }[action];
  return callServiceAction({
    location: options.location || process.env.LUMIN_P1_LOCATION,
    serviceNeedle: ["Playlist", "Transport", "AVTransport"],
    actionName,
    options,
  });
}

async function playlist(options) {
  const location = options.location || process.env.LUMIN_P1_LOCATION;
  if (options.state) {
    const out = {};
    for (const actionName of ["Id", "IdArray", "TransportState", "Repeat", "Shuffle"]) {
      out[actionName] = await callServiceAction({ location, serviceNeedle: "Playlist", actionName, options }).catch((error) => ({ error: error.message }));
    }
    return out;
  }
  if (options.read) {
    if (options.id == null) throw new Error("playlist --read requires --id");
    return callServiceAction({ location, serviceNeedle: "Playlist", actionName: "Read", args: { Id: String(options.id) }, options });
  }
  if (options["read-list"]) {
    if (options.ids == null) throw new Error("playlist --read-list requires --ids");
    return callServiceAction({ location, serviceNeedle: "Playlist", actionName: "ReadList", args: { IdList: String(options.ids) }, options });
  }
  if (options.repeat != null) {
    return callServiceAction({ location, serviceNeedle: "Playlist", actionName: "SetRepeat", args: { Value: asBoolString(options.repeat, "--repeat") }, options });
  }
  if (options.shuffle != null) {
    return callServiceAction({ location, serviceNeedle: "Playlist", actionName: "SetShuffle", args: { Value: asBoolString(options.shuffle, "--shuffle") }, options });
  }
  if (options["delete-id"] != null) {
    if (!options["confirm-playlist-write"]) throw new Error("playlist delete requires --confirm-playlist-write");
    return callServiceAction({ location, serviceNeedle: "Playlist", actionName: "DeleteId", args: { Value: String(options["delete-id"]), Id: String(options["delete-id"]) }, options });
  }
  if (options.clear) {
    if (!options["confirm-playlist-write"]) throw new Error("playlist clear requires --confirm-playlist-write");
    return callServiceAction({ location, serviceNeedle: "Playlist", actionName: "DeleteAll", options });
  }
  if (options["insert-uri"] != null) {
    if (!options["confirm-playlist-write"]) throw new Error("playlist insert requires --confirm-playlist-write");
    const afterId = options["after-id"] == null ? "0" : String(options["after-id"]);
    return callServiceAction({
      location,
      serviceNeedle: "Playlist",
      actionName: "Insert",
      args: { AfterId: afterId, Uri: String(options["insert-uri"]), Metadata: String(options.metadata || "") },
      options,
    });
  }
  throw new Error("playlist requires a read or write option");
}

async function main() {
  const command = process.argv[2];
  const options = parseArgsMulti(process.argv.slice(3));
  if (!command || command === "help" || options.help) {
    usage();
    return;
  }
  try {
    if (command === "discover") return print(await discover(options), options.json);
    if (command === "bonjour") return print(await bonjour(options), options.json);
    if (command === "spotify-info") return print(await spotifyInfo(options), options.json);
    if (command === "inspect") return print(await inspect(options), options.json);
    if (command === "services") {
      const device = await inspect(options);
      const services = device.services.map((service) => ({
        serviceType: service.serviceType,
        serviceId: service.serviceId,
        controlURL: service.controlURLResolved,
        eventSubURL: service.eventSubURLResolved,
        SCPDURL: service.SCPDURLResolved,
      }));
      return print(services, options.json);
    }
    if (command === "actions") return print(await actions(options), options.json);
    if (command === "status") return print(await status(options), options.json);
    if (command === "sources") return print(await sources(options), options.json);
    if (command === "power") return print(await power(options), options.json);
    if (command === "select-source") return print(await selectSource(options), options.json);
    if (command === "volume") return print(await volume(options), options.json);
    if (command === "playback") return print(await playback(options), options.json);
    if (command === "playlist") return print(await playlist(options), options.json);
    if (command === "udp") return print(await sendUdp(options), options.json);
    if (command === "soap") return print(await soap(options), options.json);
    throw new Error(`unknown command: ${command}`);
  } catch (error) {
    die(error?.message || String(error));
  }
}

export {
  UDP_COMMANDS,
  actionByName,
  callSoap,
  bonjour,
  decodeXml,
  defaultInputArgs,
  discover,
  ensureHttpUrl,
  escapeXml,
  fetchText,
  inspect,
  makeUdpPacket,
  parseDevice,
  parseDnsSdBrowse,
  parseScpd,
  parseSourceXml,
  postText,
  power,
  resolveAgainst,
  runCommand,
  soapEnvelope,
  sources,
  spotifyInfo,
};

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  main();
}
