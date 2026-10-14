# MCP

`apelles-automation` implements an MCP server over stdio with two backends:

- **Headless:** an in-process `Headless` session owns the engine and performs file I/O.
- **Bridge:** `BridgeClient` forwards tools to a desktop application started with `apelles --control <port>`.

Start the headless server with:

```sh
apelles-cli mcp --automation-read-root /work/project --automation-write-root /work/project
```

Bridge to a running desktop application with the same private token file used by the app:

```sh
apelles-cli mcp --bridge 127.0.0.1:7878 --control-token-file /private/path/control.token
```

Tools cover session/document operations, command discovery and execution, batching, and—when bridged—UI inspection and control. Tool schemas improve correctness but are not authorization boundaries.

## From an installed release

The release packages ship `apelles-cli` alongside the desktop app, so no build is needed:

| Install | CLI |
|---|---|
| Windows (MSI) | `C:\Program Files\Apelles\apelles-cli.exe` by default (wherever you installed it otherwise), not on `PATH` |
| Linux (deb, rpm) | `/usr/bin/apelles-cli` |
| macOS | the separate `apelles-cli-<version>-macos-<arch>.zip` release asset (the `.app` holds only the desktop app) |

```sh
# Windows, default install folder
claude mcp add apelles -- "C:\Program Files\Apelles\apelles-cli.exe" mcp --automation-read-root <dir> --automation-write-root <dir>
# Linux, or macOS with the CLI unzipped onto PATH
claude mcp add apelles -- apelles-cli mcp --automation-read-root <dir> --automation-write-root <dir>
```

## Security notes

Stdio MCP does not open a network listener. Its client receives only the read and write roots
explicitly granted at launch; omitting a root omits that authority. Bridge mode authenticates its
loopback control connection with a bearer token before invoking any control method and uses the
roots configured on the desktop process.

The current server does not issue scoped sessions or distinguish trusted tools from high-impact tools. Authentication proves possession of the token; it does not restrict an authenticated client to document-only access. Do not expose either transport through an untrusted broker, shell, proxy, or remote service without adding an external policy boundary.

Filesystem capabilities are implemented. General tool and method capabilities remain
[proposed](../security/automation-security.md).

The five shared core tools are `command_list`, `command_run`, `command_batch`, `doc_inspect`
and `render_preview`. Existing documented tools, including `doc_render_preview`, remain
listed. `apelles://document` and `apelles://commands` expose live JSON. Direct headless
Render Video supports progress and cancellation using the granted write root; see
[`docs/mcp.md`](../../../docs/mcp.md) for arguments, cleanup and transport behavior.
