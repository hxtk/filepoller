# File Poller

Synchronize local file with HTTP server. It is assumed that the remote server provides a meaningful HTTP Last-Modified header, but it is not assumed that the remote server respects If-Modified-Since headers (instead, we HEAD the file before sending a GET).

The use case this application was designed to support is keeping the latest GEOCOLOR imagery from GOES-16 in the location Gnome uses for my desktop backgorund.

All error conditions are either recovered or reported; this application does not unwrap except where it can be statically guaranteed to succeed. The application should never fail without explaining why and it should never overwrite a local copy with something that was corrupted in transport. If it does, consider it an error and please file a bug report.

# Usage

`filepoller <url> <output file>`

# FAQ

- Why not async?

Part of the justification for this program was checking if the rust compiler approved of the patterns I planned to use in some C code, which obviously does not have async.

- Why not `GET IF-MODIFIED-SINCE`?

This application was written with the purpose of synchronizing my desktop background to the latest GOES-16 GEOCOLOR imagery, and the GOES-16 server doesn't seem to respect the IF-MODIFIED_SINCE header in my testing. `HEAD`ing the file before `GET`ting it was a compromise. If I ever do more work on this program I may attempt to detect whether the server respects `IF-MODIFIED-SINCE` and handle the polling accordingly, but my initial focus was on my own use-case.
