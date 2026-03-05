# Dolt Port Conflict Between Projects

## Symptom

`bd` commands fail with:
```
Error: failed to open database: database "wai" not found on Dolt server at 127.0.0.1:3307
```

## Root Cause

Multiple projects share the default dolt port (3307). When two projects' dolt servers run simultaneously, one occupies the port and serves the wrong data directory for the other.

## Diagnosis

Check which project's data dir the server on port 3307 is actually serving:

```bash
readlink /proc/$(pgrep -d',' dolt | tr ',' '\n' | head -1)/cwd
# or walk all dolt pids:
for pid in $(pgrep dolt); do echo "PID $pid: $(readlink /proc/$pid/cwd)"; done
```

Also check the port file:
```bash
cat .beads/dolt-server.port
```

## Fix

1. Assign a unique port to this project:
   ```bash
   bd dolt set port 13421
   ```

2. Update the port file so bd reads the correct port:
   ```bash
   echo "13421" > .beads/dolt-server.port
   ```

3. Start the server with the correct data dir (if bd dolt start picks up the wrong one):
   ```bash
   cd .beads/dolt && dolt sql-server -H 127.0.0.1 -P 13421 --data-dir . &
   ```

4. Do NOT kill the other project's server — just redirect this project to its own port.

## Current State

- `wai` project: port 13421
- `nayra` project: port 3307 (default)
- `fabbro` project: port 13420

## Prevention

Each project should use a distinct port from the start. Set with `bd dolt set port <N>` immediately after `bd init`.

