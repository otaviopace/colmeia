# colmeia

----
Hive (in portuguese). Attempt to make an interop layer to connect to [dat](https://github.com/datrs/) on [hyperswarm](https://github.com/hyperswarm) and legacy infra as well.

- [x] `colmeia-mdns`
  - [x] `Locator`: stream to find dat members in the network
  - [ ] `Announcer`: stream that announces a dat in the network
  - [ ] `Mdns`: announces and find dat in the network
- [ ] `colmeia-dht`: Interop with hypwerswarm dht infrastructure
- [ ] `colmeia-dns`: DNS locator to resolve a hostname into a dat hash
- [ ] `colmeia-network`: Network pool manager, based on mdns and dht members

## Utils

```sh
RUST_LOG=debug cargo run --bin colmeia-mdns -- dat://460f04cf12c3b9833e5a0d3dd8eea05eab59dd8c1438a7454afe9630b9b4f8bd
```
