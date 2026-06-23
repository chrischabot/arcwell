# LUMIN P1 Protocol Notes

Retrieval date: 2026-06-22.

## Confirmed Surfaces

- Official LUMIN P1 specs list `UPnP AV protocol with audio streaming extension
  (OpenHome)`, Roon Ready, Qobuz Connect, TIDAL Connect, Spotify Connect,
  AirPlay-compatible playback, gapless playback, and on-device playlists.
- The official LUMIN network-control PDF says OpenHome is the full
  playlist/playback-control protocol.
- The same PDF documents a simple firmware 7.0+ UDP command channel for a
  subset of remote-control functions.
- The LUMIN App page says the app uses UPnP AV/OpenHome for its main browsing,
  playlist, metadata, and volume experience. It also says Spotify, Roon,
  QQMusic, and AirPlay largely bypass the LUMIN App.

## Live P1 Observations

From the user's LAN on 2026-06-22, the P1 was reachable at `192.168.0.8`.
Bonjour advertised `LUMIN P1` over AirPlay/RAOP and Spotify Connect, and
Spotify zeroconf reported `remoteName: LUMIN P1`, `deviceType: SPEAKER`,
`brandDisplayName: LUMIN`, and `modelDisplayName: P1`.

The AirPlay endpoint on port `7000` responded as `AirTunes/366.0`, and the
Spotify Connect endpoint on port `43669` served `/zc?action=getInfo&version=2`.
Repeated SSDP/OpenHome discovery attempts from this host returned no
`LOCATION`, so OpenHome SOAP control remains fixture-tested but not live-proven
against this P1 until the device description URL is captured.

## UDP Shortcut Protocol

Send a 16-byte UDP packet to the player IP on port `23456`.

Packet template:

```text
55 00 00 40 00 08 00 04 00 04 00 d8 2a <cmd0> <cmd1> aa
```

Command bytes:

```text
play        21 de
pause       22 dd
previous    27 d8
next        28 d7
volume-up   14 eb
volume-down 15 ea
mute        18 e7
unmute      4e b1
repeat-on   32 cd
repeat-off  34 cb
shuffle-on  33 cc
shuffle-off 4f b0
standby     70 8f
wake        71 8e
```

Use UDP as a shortcut/fallback. It has no documented response body or state
model, so follow with OpenHome/UPnP reads when possible.

## OpenHome/UPnP Model

Discovery starts with SSDP. Useful search targets:

```text
ssdp:all
urn:schemas-upnp-org:device:MediaRenderer:1
urn:av-openhome-org:service:Product:1
urn:av-openhome-org:service:Playlist:1
```

Fetch the response `LOCATION` URL, parse the device description, then inspect
services and service descriptors (`SCPDURL`). Bind to the discovered
`controlURL`; do not hard-code paths.

Prioritize these service families when present:

- OpenHome `Product`: source/capability discovery and likely source selection.
- OpenHome `Playlist` / `Transport`: playlist and playback control.
- OpenHome `Volume`: volume/mute where exposed.
- OpenHome `Info` / `Time`: now-playing/status where exposed.
- Standard UPnP `AVTransport` and `RenderingControl`: fallback behavior.

## Skill Command Coverage

The bundled script now exposes:

- Discovery and inspection: `discover`, `inspect`, `services`, `actions`.
- Fallback device presence: `bonjour --filter lumin` and `spotify-info --host`
  for AirPlay/RAOP/Spotify Connect identity when SSDP is silent.
- Read paths: `status`, `sources`, `volume --get`, `playlist --state`,
  `playlist --read`, `playlist --read-list`, `power --get`.
- Playback writes: `playback --action play|pause|stop|next|previous` via SOAP,
  or UDP where explicitly requested.
- Standby writes: `power --standby true|false --confirm-standby` via Product
  service, or the official UDP standby/wake shortcut when explicit.
- Volume writes: `volume --set N --confirm-volume`, `volume --mute true|false`.
- Source writes: `select-source --index N` or `--system-name NAME`, both gated
  by `--confirm-source-select` and dependent on actual Product service actions.
- Playlist writes: repeat/shuffle toggles, insert, delete-id, and clear. Insert,
  delete-id, and clear require `--confirm-playlist-write`.
- Raw fallback: `soap --service ... --action ... --arg Name=Value`.

Safety constraints:

- Descriptor URLs are restricted to the same host as the device description by
  default. Use `--allow-cross-host-service-urls` only after inspecting the
  device descriptor manually.
- HTTP/SOAP responses are size-limited.
- `standby`, absolute volume, source selection, and destructive playlist writes
  are confirmation-gated.
- Typed SOAP wrappers inspect SCPD action descriptors and only send declared
  input arguments.
- Spotify Connect `/zc?action=getInfo` is read-only identity/Spotify metadata,
  not the OpenHome/LUMIN control plane.

## Unknown Until Device XML Is Captured

- Exact service/action names for P1 input switching.
- Whether P1-specific settings such as Leedh, MQA, output, polarity, or volume
  range are public OpenHome actions or app-private behavior.
- Whether Arcwell can initiate Connect sessions directly through LUMIN-local
  APIs. Treat TIDAL/Qobuz/Spotify/Roon/AirPlay handoff as separate until proven.
