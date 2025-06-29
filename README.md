# NetCoreDbg Debug Adapter Extension for Zed

A Zed extension that provides debugging support for .NET applications using [netcoredbg](https://github.com/Samsung/netcoredbg).

Currently only support Mac arm64 due to an upstream [issue with netcoredbg](https://github.com/Samsung/netcoredbg/issues/197).

## Installation

### Development Installation

1. Clone this repository
2. Open Zed
3. Go to Extensions (Cmd+Shift+X on macOS)
4. Click "Install Dev Extension"
5. Select the cloned directory

## Configuration

You can use the Debugger UI to add a new debug configuration or edit `.zed/debug.json` directly.

Example `.zed/debug.json` to add a new launch debug configuration:

```json
[
  {
    "label": "Debug .NET Core App",
    "adapter": "netcoredbg",
    "request": "launch",
    "program": "${ZED_WORKTREE_ROOT}/apps/backend/Example/bin/Debug/net8.0/Example.dll",
    "cwd": "${ZED_WORKTREE_ROOT}/apps/backend/Example",
    "env": {
      "ASPNETCORE_ENVIRONMENT": "Development"
    },
    "build": {
      "command": "dotnet",
      "args": [
        "build",
        "${ZED_WORKTREE_ROOT}/apps/backend/Example/Example.csproj",
        "-p:Configuration=Debug",
        "-p:TargetFramework=net8.0"
      ]
    }
  }
]
```

Attach to a running process:

```json
{
  "label": "Attach .NET Core App",
  "adapter": "netcoredbg",
  "request": "attach",
  "processId": 62177
}
```

The extension automatically tries to download netcoredbg's executable. Or you can configure the path in your Zed settings:

```json
{
  "dap": {
    "netcoredbg": {
      "binary": "/path/to/netcoredbg",
      "args": []
    }
  }
}
```

## Why netcoredbg?

While Microsoft provides official debugging libraries for .NET Core (`Microsoft.VisualStudio.clrdbg`), these come with [restrictive licensing terms](https://github.com/dotnet/core/issues/505) that limit their use to specific IDEs like Visual Studio Code. This licensing restriction has prevented many third-party editors and IDEs from offering .NET debugging support.

**netcoredbg** solves this problem by providing an open-source alternative:

- **Truly Open Source**: Developed by Samsung under a permissive license
- **Cross-Platform**: Works on Windows, macOS, and Linux
- **IDE Agnostic**: Can be integrated into any editor or IDE that supports the Debug Adapter Protocol (DAP)
- **Full-Featured**: Provides comprehensive debugging capabilities for .NET Core applications

This makes netcoredbg the ideal choice for Zed.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License.
