import assert from "node:assert/strict";
import http from "node:http";
import { spawnSync } from "node:child_process";
import { test } from "node:test";

import {
  fetchText,
  makeUdpPacket,
  parseDevice,
  parseDnsSdBrowse,
  parseScpd,
  parseSourceXml,
  defaultInputArgs,
  resolveAgainst,
  runCommand,
  soapEnvelope,
} from "./lumin.mjs";

const SCRIPT = new URL("./lumin.mjs", import.meta.url).pathname;

test("CLAIM: official UDP packet bytes match the LUMIN protocol map", () => {
  assert.equal(
    makeUdpPacket("pause").toString("hex").match(/.{1,2}/g).join(" "),
    "55 00 00 40 00 08 00 04 00 04 00 d8 2a 22 dd aa",
  );
  assert.equal(
    makeUdpPacket("volume-down").toString("hex").match(/.{1,2}/g).join(" "),
    "55 00 00 40 00 08 00 04 00 04 00 d8 2a 15 ea aa",
  );
});

test("CLAIM: standby write is gated even in dry-run mode", () => {
  const result = spawnSync(process.execPath, [
    SCRIPT,
    "udp",
    "--host",
    "192.0.2.1",
    "--command",
    "standby",
    "--dry-run",
  ], { encoding: "utf8" });
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /standby requires --confirm-standby/);
});

test("CLAIM: device XML parsing extracts services without executing source text", () => {
  const device = parseDevice(`
    <root><device>
      <friendlyName>&lt;script&gt;ignore&lt;/script&gt; P1</friendlyName>
      <manufacturer>LUMIN</manufacturer>
      <modelName>P1</modelName>
      <UDN>uuid:p1</UDN>
      <serviceList>
        <service>
          <serviceType>urn:av-openhome-org:service:Product:1</serviceType>
          <serviceId>urn:av-openhome-org:serviceId:Product</serviceId>
          <controlURL>/Product/control</controlURL>
          <eventSubURL>/Product/event</eventSubURL>
          <SCPDURL>/Product/scpd.xml</SCPDURL>
        </service>
      </serviceList>
    </device></root>
  `, "http://192.0.2.50:1234/device.xml");
  assert.equal(device.friendlyName, "<script>ignore</script> P1");
  assert.equal(device.services[0].serviceType, "urn:av-openhome-org:service:Product:1");
});

test("CLAIM: SCPD parsing captures action inputs and outputs", () => {
  const actions = parseScpd(`
    <scpd><actionList>
      <action><name>SetVolume</name><argumentList>
        <argument><name>Value</name><direction>in</direction><relatedStateVariable>Volume</relatedStateVariable></argument>
      </argumentList></action>
      <action><name>Volume</name><argumentList>
        <argument><name>Value</name><direction>out</direction><relatedStateVariable>Volume</relatedStateVariable></argument>
      </argumentList></action>
    </actionList></scpd>
  `);
  assert.deepEqual(actions.map((action) => action.name), ["SetVolume", "Volume"]);
  assert.deepEqual(actions[0].arguments[0], {
    name: "Value",
    direction: "in",
    relatedStateVariable: "Volume",
  });
});

test("CLAIM: Bonjour browse parsing extracts LUMIN service instances", () => {
  const rows = parseDnsSdBrowse(`
21:54:27.406  Add        3  16 local.               _airplay._tcp.       LUMIN P1
21:54:31.417  Add        3  16 local.               _raop._tcp.          32F8C8AF6B16@LUMIN P1
  `, "_airplay._tcp");
  assert.equal(rows.length, 2);
  assert.equal(rows[0].name, "LUMIN P1");
  assert.equal(rows[1].name, "32F8C8AF6B16@LUMIN P1");
});

test("CLAIM: SOAP wrapper args are filtered to descriptor-declared inputs", () => {
  const [action] = parseScpd(`
    <scpd><actionList><action><name>SetVolume</name><argumentList>
      <argument><name>DesiredVolume</name><direction>in</direction></argument>
    </argumentList></action></actionList></scpd>
  `);
  assert.deepEqual(defaultInputArgs(action, {
    Value: "40",
    DesiredVolume: "40",
    InstanceID: "0",
    Channel: "Master",
  }), { DesiredVolume: "40" });
});

test("CLAIM: common UPnP AVTransport defaults include play speed", () => {
  const [action] = parseScpd(`
    <scpd><actionList><action><name>Play</name><argumentList>
      <argument><name>InstanceID</name><direction>in</direction></argument>
      <argument><name>Speed</name><direction>in</direction></argument>
    </argumentList></action></actionList></scpd>
  `);
  assert.deepEqual(defaultInputArgs(action), { InstanceID: "0", Speed: "1" });
});

test("CLAIM: SOAP envelopes XML-escape attacker-controlled argument values", () => {
  const body = soapEnvelope("urn:av-openhome-org:service:Playlist:1", "Insert", {
    Uri: "http://example.test/a?x=1&y=<bad>",
    Metadata: "\"quoted\" 'single'",
  });
  assert.match(body, /x=1&amp;y=&lt;bad&gt;/);
  assert.match(body, /&quot;quoted&quot; &apos;single&apos;/);
  assert.doesNotMatch(body, /y=<bad>/);
});

test("CLAIM: service URL resolution refuses cross-host descriptor redirects by default", () => {
  assert.equal(
    resolveAgainst("http://192.0.2.50:1234/device.xml", "/Product/control"),
    "http://192.0.2.50:1234/Product/control",
  );
  assert.throws(
    () => resolveAgainst("http://192.0.2.50:1234/device.xml", "http://169.254.169.254/latest"),
    /refusing cross-host service URL/,
  );
});

test("CLAIM: SourceXml parsing preserves source attributes for read/select workflows", () => {
  const sources = parseSourceXml(`&lt;SourceList&gt;
    &lt;Source Name=&quot;Playlist&quot; Type=&quot;Playlist&quot; Visible=&quot;true&quot; SystemName=&quot;playlist&quot;/&gt;
    &lt;Source Name=&quot;HDMI ARC&quot; Type=&quot;Digital&quot; Visible=&quot;true&quot; SystemName=&quot;hdmi-arc&quot;/&gt;
  &lt;/SourceList&gt;`);
  assert.equal(sources.length, 2);
  assert.equal(sources[1].name, "HDMI ARC");
  assert.equal(sources[1].systemName, "hdmi-arc");
});

test("CLAIM: HTTP fetches enforce response-size limits", async () => {
  const server = http.createServer((req, res) => {
    res.writeHead(200, { "Content-Type": "text/xml" });
    res.end("x".repeat(128));
  });
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
  const { port } = server.address();
  try {
    await assert.rejects(
      fetchText(`http://127.0.0.1:${port}/device.xml`, { "max-bytes": 16 }),
      /response too large/,
    );
  } finally {
    await new Promise((resolve) => server.close(resolve));
  }
});

test("CLAIM: external command timeouts do not hang on stubborn children", async () => {
  const started = Date.now();
  const result = await runCommand(process.execPath, [
    "-e",
    "process.on('SIGTERM', () => {}); setInterval(() => {}, 1000);",
  ], 50);
  assert.equal(result.ok, false);
  assert.equal(result.timedOut, true);
  assert.ok(Date.now() - started < 2000);
});
