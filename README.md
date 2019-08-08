# Example
## NOTE: REQUIRES NIGHTLY BUILD.

## Steps to run example:

1. Build Server. To do that, run in basic/server directory: (Server.dll will be output in target/debug folder)

```bash
cargo build
```

2. "Install" server by adding the following CLSIDs to the Registry:

```bash
Computer\HKEY_CLASSES_ROOT\CLSID\{b70be3a9-6531-423b-b0d6-3042a47d0678}
@="British Short Hair Cat Component"

Computer\HKEY_CLASSES_ROOT\CLSID\{b70be3a9-6531-423b-b0d6-3042a47d0678}\InprocServer32
@="path/to/your/server/dll/"
```

3. Run example. To do that, run in basic/client directory:

```bash
cargo run
```