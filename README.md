testing
========


"testbin" contains a hyper client/server with separate binaries for client and server, as well as a multicall.


|          | client  | server  | total | multicall   |   |
|----------|---------|---------|-------|-------------|---|
| debug    | 21M     | 19M     | 40M   | 22M  (55%)  |   |
| release  | 4.8M    | 4.7M    | 9.5M  | 5M   (52%)  |   |
| stripped | 1.4M    | 1.3M    | 2.7M  | 1.6M (59%)  |   |
