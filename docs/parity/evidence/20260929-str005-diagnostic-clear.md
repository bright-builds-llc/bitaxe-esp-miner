# Captured self-test core cleared; current recovery passed

The archived Capture001 dump was compared byte-for-byte with the on-device core
before erasure. The official clear tool erased only the admitted 974,848-byte
core partition, verified every readback byte as `FF`, returned to the exact
installed image and released its child group and serial owner. The private raw
dump and decoder products remain sealed.

| Boundary                    | Result                                                                                                          |
| --------------------------- | --------------------------------------------------------------------------------------------------------------- |
| Installed source / full ELF | `ce8f015811b93385c5aab0bac7bcdf6307452b31` / `453d2fa3bbe2b58bcffcbf2019ab69c7968d1ae90620685c8139a061a3325c31` |
| Preserved raw SHA-256       | `9db6ac9b2cbb52be6493a6e8f397388bac1a69a36ad17e5acf71369fd4de1644`                                              |
| Fresh pre-clear recovery    | Boot22, idle, restored, no pending accounting                                                                   |
| Clear producer              | Code0, complete, exact raw match, full erased readback, exact-image return                                      |
| Child and cleanup           | Worker/group release and fresh physical/serial absence verified                                                 |
| Fresh post-clear recovery   | Boot23, same image, idle, restoration and preservation confirmed                                                |
| Accounting                  | Next21/last20/charged2,100,000 ms; original charged240,000 ms; both pending=false                               |

The private pre-clear recovery seal is
`a836759ee555a11c3d394b3055ee8f955934ca1f1b6bcca735fb47d1d4720e4d`.
The archive-bound clear seal is
`30ed731be9a6e698ae8d56d13aaf5e0c47354d60e3f766261d22467ef9948d82`.
The independent post-clear recovery seal is
`ab3dc48ef94d0fc9c8c0137728950d21880131ac769afffe867e36a6d0ec2d0c`.
All evidence roots are under ignored owner-only
`scratch/str005-share-diagnostic/` directories. No dump, debugger output,
network endpoint, pool identity or credential is included here.

The clear and generic acquisition gates are consumed and disabled. No Start,
grant, mining or renewal occurred. The accepted-share task remains active;
Share001's panic cause and historical resource gap are unresolved, and parity
remains **90/95**. A new bounded Start/status reproduction needs its own
published contract and fresh measured accounting.
