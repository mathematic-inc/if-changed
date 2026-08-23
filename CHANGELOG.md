# Changelog

## [0.4.0](https://github.com/mathematic-inc/if-changed/compare/v0.3.7...v0.4.0) (2026-08-23)


### ⚠ BREAKING CHANGES

* reverse pattern matching
* allow pathspec for files

### Features

* Allow pathspec for files ([cc0c7bb](https://github.com/mathematic-inc/if-changed/commit/cc0c7bbac95091d03dc54471f7a3b4839eff75a9))
* Initial commit ([8377a79](https://github.com/mathematic-inc/if-changed/commit/8377a790f561b98807d54238f39ab724b2278b74))
* Reverse pattern matching ([83ef2c8](https://github.com/mathematic-inc/if-changed/commit/83ef2c843f20a9f810b77d4df15460baffe44df6))


### Bug Fixes

* Allow missing `from-ref` ([#8](https://github.com/mathematic-inc/if-changed/issues/8)) ([dc44e5d](https://github.com/mathematic-inc/if-changed/commit/dc44e5d9234476ad540935730840a6ed94fe5d32))
* Allow missing HEAD ([322b95a](https://github.com/mathematic-inc/if-changed/commit/322b95a7ac4f855f94d49c32f4d0b165f3a9ad4c))
* Check deleted file pragmas ([f63ae2b](https://github.com/mathematic-inc/if-changed/commit/f63ae2bf839701e856e9126c9d88568a9115e38e))
* **deps:** Update cargo ([#29](https://github.com/mathematic-inc/if-changed/issues/29)) ([2216d27](https://github.com/mathematic-inc/if-changed/commit/2216d279ce3291a53732ef44636c4a25a2b6d183))
* **deps:** Update rust crate bstr to v1.10.0 ([#160](https://github.com/mathematic-inc/if-changed/issues/160)) ([191697d](https://github.com/mathematic-inc/if-changed/commit/191697d9d9efd80dcb392d7e2317d21528eea21c))
* **deps:** Update rust crate clap to v4.5.10 ([#153](https://github.com/mathematic-inc/if-changed/issues/153)) ([e0cd360](https://github.com/mathematic-inc/if-changed/commit/e0cd3606fe7c0ef0d503ff670ed5e2a74f8fbcff))
* **deps:** Update rust crate clap to v4.5.11 ([#156](https://github.com/mathematic-inc/if-changed/issues/156)) ([3820278](https://github.com/mathematic-inc/if-changed/commit/382027806e2a905402677f817a0f27e3ebe059a5))
* **deps:** Update rust crate clap to v4.5.12 ([#175](https://github.com/mathematic-inc/if-changed/issues/175)) ([dc488ab](https://github.com/mathematic-inc/if-changed/commit/dc488ab921c75d017bda6de79d9e9a934e234105))
* **deps:** Update rust crate clap to v4.5.13 ([#176](https://github.com/mathematic-inc/if-changed/issues/176)) ([ce8b0c5](https://github.com/mathematic-inc/if-changed/commit/ce8b0c55286bdea0f22ccc6da39714c48aa8854d))
* **deps:** Update rust crate clap to v4.5.14 ([#190](https://github.com/mathematic-inc/if-changed/issues/190)) ([43dc86a](https://github.com/mathematic-inc/if-changed/commit/43dc86a5596a44a9c91f00b167b13b244deae3fd))
* **deps:** Update rust crate clap to v4.5.15 ([#192](https://github.com/mathematic-inc/if-changed/issues/192)) ([d0de4b3](https://github.com/mathematic-inc/if-changed/commit/d0de4b323a2150c887cbf1598c18a9f414c86b55))
* **deps:** Update rust crate clap to v4.5.16 ([#204](https://github.com/mathematic-inc/if-changed/issues/204)) ([a78c405](https://github.com/mathematic-inc/if-changed/commit/a78c40535c0a603551f5f3f4b278ce59a8349287))
* **deps:** Update rust crate clap to v4.5.18 ([#264](https://github.com/mathematic-inc/if-changed/issues/264)) ([2e9ddfd](https://github.com/mathematic-inc/if-changed/commit/2e9ddfdc62d1df3f4fe76524788a37af1e4efdcb))
* **deps:** Update rust crate clap to v4.5.5 ([#69](https://github.com/mathematic-inc/if-changed/issues/69)) ([a0aa685](https://github.com/mathematic-inc/if-changed/commit/a0aa6851486453da301d04b0e806cbc309f3cfb4))
* **deps:** Update rust crate clap to v4.5.6 ([#70](https://github.com/mathematic-inc/if-changed/issues/70)) ([fc8d6d1](https://github.com/mathematic-inc/if-changed/commit/fc8d6d1a8df78da216adf4f08552ac41c8cd5d59))
* **deps:** Update rust crate clap to v4.5.7 ([#78](https://github.com/mathematic-inc/if-changed/issues/78)) ([330338b](https://github.com/mathematic-inc/if-changed/commit/330338b464d77e07020234ab8c967e165f3f5541))
* **deps:** Update rust crate clap to v4.5.8 ([#121](https://github.com/mathematic-inc/if-changed/issues/121)) ([8247bbc](https://github.com/mathematic-inc/if-changed/commit/8247bbcf7d5e1d9c6f7a7baa64775cdcfd5bfa9d))
* **deps:** Update rust crate clap to v4.5.9 ([#132](https://github.com/mathematic-inc/if-changed/issues/132)) ([9eb0029](https://github.com/mathematic-inc/if-changed/commit/9eb0029cc479038e51d2cc2a51602ed34e9f5f11))
* **deps:** Update rust crate git2 to 0.19.0 ([#89](https://github.com/mathematic-inc/if-changed/issues/89)) ([75c3646](https://github.com/mathematic-inc/if-changed/commit/75c3646cc174ce7a9c38f92eb9bf40d674339d8c))
* **deps:** Update rust crate git2 to 0.20.0 [security] ([#274](https://github.com/mathematic-inc/if-changed/issues/274)) ([cea82c7](https://github.com/mathematic-inc/if-changed/commit/cea82c7ce1cecf9759a33ccdfc642b05421a0575))
* **deps:** Update rust crate git2 to 0.21.0 ([#340](https://github.com/mathematic-inc/if-changed/issues/340)) ([fbcc105](https://github.com/mathematic-inc/if-changed/commit/fbcc1053375b1f382f4cc7a0afd4513d21f862eb))
* Include untracked files ([88969c9](https://github.com/mathematic-inc/if-changed/commit/88969c96533bda683302d5a5d2eb3e2199d9a5d3))
* Overhaul CI workflows, update license to Apache-2.0 only, and minor code cleanup ([#275](https://github.com/mathematic-inc/if-changed/issues/275)) ([ca574be](https://github.com/mathematic-inc/if-changed/commit/ca574be65438fd4d3faaaf47580c1c161c48e2d5))
* Read pre-commit refs from env ([049c387](https://github.com/mathematic-inc/if-changed/commit/049c38740e6c9657e9fa8a8d9c2dae88ea27f315))
* **release:** Add missing toolchain input to rust-toolchain steps ([#277](https://github.com/mathematic-inc/if-changed/issues/277)) ([aba1140](https://github.com/mathematic-inc/if-changed/commit/aba11405ee3585a1d2fc781fc04fbe89e8609e8b))
* Skip changed directories ([#413](https://github.com/mathematic-inc/if-changed/issues/413)) ([2e4bddc](https://github.com/mathematic-inc/if-changed/commit/2e4bddcc667f726d0de836fc5e141e7586003b96))
* Skip deleted diff paths ([1eb334d](https://github.com/mathematic-inc/if-changed/commit/1eb334d52286ca88d74c36d29901af2412de8881))

## [0.3.7](https://github.com/mathematic-inc/if-changed/compare/v0.3.6...v0.3.7) (2026-07-22)


### Bug Fixes

* Skip changed directories ([#413](https://github.com/mathematic-inc/if-changed/issues/413)) ([edff21e](https://github.com/mathematic-inc/if-changed/commit/edff21e429533a29594bfb6c28bba02946f48b7a))

## [0.3.6](https://github.com/mathematic-inc/if-changed/compare/v0.3.5...v0.3.6) (2026-07-04)


### Bug Fixes

* Check deleted file pragmas ([b94a9c8](https://github.com/mathematic-inc/if-changed/commit/b94a9c81b43c711f2f5e80369ba358db3f854328))

## [0.3.5](https://github.com/mathematic-inc/if-changed/compare/v0.3.4...v0.3.5) (2026-07-04)


### Bug Fixes

* **deps:** Update rust crate git2 to 0.21.0 ([#340](https://github.com/mathematic-inc/if-changed/issues/340)) ([f1923da](https://github.com/mathematic-inc/if-changed/commit/f1923daeed89517cb791a57855cc5274d0413fc0))
* Skip deleted diff paths ([580e945](https://github.com/mathematic-inc/if-changed/commit/580e94525e8b918863b4d707d0f268bb7065ec94))

## [0.3.4](https://github.com/mathematic-inc/if-changed/compare/v0.3.3...v0.3.4) (2026-03-11)


### Bug Fixes

* **deps:** Update rust crate git2 to 0.20.0 [security] ([#274](https://github.com/mathematic-inc/if-changed/issues/274)) ([65c7663](https://github.com/mathematic-inc/if-changed/commit/65c76639acbf305195be3ca2021bf3531b8b4fa9))

## [0.3.3](https://github.com/mathematic-inc/if-changed/compare/v0.3.2...v0.3.3) (2026-03-11)


### Bug Fixes

* **deps:** Update rust crate bstr to v1.10.0 ([#160](https://github.com/mathematic-inc/if-changed/issues/160)) ([ff1b235](https://github.com/mathematic-inc/if-changed/commit/ff1b235b04c73720f21a0ed2ae4866a91e5178b5))
* **deps:** Update rust crate clap to v4.5.10 ([#153](https://github.com/mathematic-inc/if-changed/issues/153)) ([85718b5](https://github.com/mathematic-inc/if-changed/commit/85718b5934cbf6c6fa376a3613ca34f8eae43468))
* **deps:** Update rust crate clap to v4.5.11 ([#156](https://github.com/mathematic-inc/if-changed/issues/156)) ([7113414](https://github.com/mathematic-inc/if-changed/commit/7113414533033004bdeb595c8e889be3c3de030d))
* **deps:** Update rust crate clap to v4.5.12 ([#175](https://github.com/mathematic-inc/if-changed/issues/175)) ([c23bb05](https://github.com/mathematic-inc/if-changed/commit/c23bb059aed84693f157f5cd4c954465c0a5f423))
* **deps:** Update rust crate clap to v4.5.13 ([#176](https://github.com/mathematic-inc/if-changed/issues/176)) ([92b7d4c](https://github.com/mathematic-inc/if-changed/commit/92b7d4cb63453834ed5d07df9c2cc04427c6cbb3))
* **deps:** Update rust crate clap to v4.5.14 ([#190](https://github.com/mathematic-inc/if-changed/issues/190)) ([5942444](https://github.com/mathematic-inc/if-changed/commit/59424448659456940cc688e019050e2a49dab12e))
* **deps:** Update rust crate clap to v4.5.15 ([#192](https://github.com/mathematic-inc/if-changed/issues/192)) ([9feded3](https://github.com/mathematic-inc/if-changed/commit/9feded3229a97956ca8455e37afb3aaf0878d935))
* **deps:** Update rust crate clap to v4.5.16 ([#204](https://github.com/mathematic-inc/if-changed/issues/204)) ([90fb9a1](https://github.com/mathematic-inc/if-changed/commit/90fb9a17b9a7d3564ab583594546e6ec3cd3d62a))
* **deps:** Update rust crate clap to v4.5.18 ([#264](https://github.com/mathematic-inc/if-changed/issues/264)) ([2871b71](https://github.com/mathematic-inc/if-changed/commit/2871b71252d86fa03fdfe91aed5302369056adb0))
* **deps:** Update rust crate clap to v4.5.8 ([#121](https://github.com/mathematic-inc/if-changed/issues/121)) ([e217226](https://github.com/mathematic-inc/if-changed/commit/e217226d22efeb7477cfc9bc6e4b899d3d6be042))
* **deps:** Update rust crate clap to v4.5.9 ([#132](https://github.com/mathematic-inc/if-changed/issues/132)) ([f4eb9bf](https://github.com/mathematic-inc/if-changed/commit/f4eb9bf50310efb46fd4b8bae5a6cbb8ad1b8253))
* **deps:** Update rust crate git2 to 0.19.0 ([#89](https://github.com/mathematic-inc/if-changed/issues/89)) ([130f53a](https://github.com/mathematic-inc/if-changed/commit/130f53a2e5bd3e971543bfe8ec9cd4a5d69a0732))
* Overhaul CI workflows, update license to Apache-2.0 only, and minor code cleanup ([#275](https://github.com/mathematic-inc/if-changed/issues/275)) ([8ca9947](https://github.com/mathematic-inc/if-changed/commit/8ca9947b965819d03298f0d0ec6a78bf73783a51))

## [0.3.2](https://github.com/mathematic-inc/if-changed/compare/v0.3.1...v0.3.2) (2024-06-13)


### Bug Fixes

* **deps:** Update cargo ([#29](https://github.com/mathematic-inc/if-changed/issues/29)) ([c5b6b82](https://github.com/mathematic-inc/if-changed/commit/c5b6b822e917a5b61c0049fccaa6d6c0249c4e11))
* **deps:** Update rust crate clap to v4.5.5 ([#69](https://github.com/mathematic-inc/if-changed/issues/69)) ([8eb3430](https://github.com/mathematic-inc/if-changed/commit/8eb3430d69feff75c8338c1ec0f5ffc76b0dc7ec))
* **deps:** Update rust crate clap to v4.5.6 ([#70](https://github.com/mathematic-inc/if-changed/issues/70)) ([23a0547](https://github.com/mathematic-inc/if-changed/commit/23a05473dfde5878bcfab355bb9b7fcfbae90d1f))
* **deps:** Update rust crate clap to v4.5.7 ([#78](https://github.com/mathematic-inc/if-changed/issues/78)) ([16382d0](https://github.com/mathematic-inc/if-changed/commit/16382d04655f94772554c61ab262f3265426eacb))

## [0.3.1](https://github.com/mathematic-inc/if-changed/compare/v0.3.0...v0.3.1) (2024-04-21)


### Bug Fixes

* Allow missing `from-ref` ([#8](https://github.com/mathematic-inc/if-changed/issues/8)) ([e0eea9b](https://github.com/mathematic-inc/if-changed/commit/e0eea9b6a0f516b5ac77a60b7d6d9e52d208dffa))

## [0.3.0](https://github.com/mathematic-inc/if-changed/compare/v0.2.0...v0.3.0) (2024-04-20)


### ⚠ BREAKING CHANGES

* reverse pattern matching

### Features

* Reverse pattern matching ([7d09224](https://github.com/mathematic-inc/if-changed/commit/7d092248560313564a55ef58e6e73e54cb00afc2))

## [0.2.0](https://github.com/mathematic-inc/if-changed/compare/v0.1.0...v0.2.0) (2024-04-19)


### ⚠ BREAKING CHANGES

* allow pathspec for files

### Features

* Allow pathspec for files ([5df28a6](https://github.com/mathematic-inc/if-changed/commit/5df28a6ade078a827140fe76d33da3e4a4d65c1d))


### Bug Fixes

* Allow missing HEAD ([81d5c2e](https://github.com/mathematic-inc/if-changed/commit/81d5c2ea0f9497fc7e778059808f8dac37257795))
* Include untracked files ([40f2371](https://github.com/mathematic-inc/if-changed/commit/40f23711d45a01738b2089c5b2ba4325de5d2067))
* Read pre-commit refs from env ([0a1e5db](https://github.com/mathematic-inc/if-changed/commit/0a1e5db64749a04ff1f6bf9b19f5d7221a8d7e68))

## 0.1.0 (2024-04-19)


### Features

* Initial commit ([b3d8f62](https://github.com/mathematic-inc/if-changed/commit/b3d8f6247a70ef30a4423e89959b2639eb0a15e9))
