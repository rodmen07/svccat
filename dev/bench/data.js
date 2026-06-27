window.BENCHMARK_DATA = {
  "lastUpdate": 1782602230526,
  "repoUrl": "https://github.com/rodmen07/svccat",
  "entries": {
    "Benchmark": [
      {
        "commit": {
          "author": {
            "email": "rodmendoza07@gmail.com",
            "name": "Roderick Mendoza",
            "username": "rodmen07"
          },
          "committer": {
            "email": "rodmendoza07@gmail.com",
            "name": "Roderick Mendoza",
            "username": "rodmen07"
          },
          "distinct": true,
          "id": "248e7f9dd5ab3f9a1e3a9c13dd23e27328c413cf",
          "message": "fix(ci): make Performance Benchmarks workflow pass\n\nThe benchmark job ran `cargo bench` but never wrote the file the tracking\naction reads, so github-action-benchmark failed with ENOENT on\ntarget/criterion/output.txt on every run since it was added.\n\nRun the criterion bench with `--output-format bencher` and tee stdout to\noutput.txt (the format `tool: cargo` parses), point the action at it, and\ngrant `contents: write` so auto-push to gh-pages can succeed.\n\nCo-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>",
          "timestamp": "2026-06-03T00:06:53-05:00",
          "tree_id": "7981eb4089871af67f62b9810a4372ea076d2f6c",
          "url": "https://github.com/rodmen07/svccat/commit/248e7f9dd5ab3f9a1e3a9c13dd23e27328c413cf"
        },
        "date": 1780463970166,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 11211,
            "range": "± 325",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 21622,
            "range": "± 145",
            "unit": "ns/iter"
          },
          {
            "name": "validate_public_url",
            "value": 289,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "reject_private_ip",
            "value": 4833,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 4433,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 11059,
            "range": "± 22",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "rodmendoza07@gmail.com",
            "name": "Roderick Mendoza",
            "username": "rodmen07"
          },
          "committer": {
            "email": "rodmendoza07@gmail.com",
            "name": "Roderick Mendoza",
            "username": "rodmen07"
          },
          "distinct": true,
          "id": "470e4a3a3af13b91fa945c880fafed6807b77535",
          "message": "chore(release): prepare v0.21.0\n\nBump version 0.20.0 -> 0.21.0. Reconcile the CHANGELOG: move the shipped\n`audit --cost-estimate` feature into a [0.20.0] section and add an\n[Unreleased] section covering multi-repo workspaces, cross-repo dependency\nanalysis, composable rules, and the watch/install-hooks changes. Document\n`svccat workspace` in the README.\n\nCo-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>",
          "timestamp": "2026-06-03T00:30:08-05:00",
          "tree_id": "a8f3cb66d7cd144323aadc3bd344ed573956a5d2",
          "url": "https://github.com/rodmen07/svccat/commit/470e4a3a3af13b91fa945c880fafed6807b77535"
        },
        "date": 1780464803350,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 12489,
            "range": "± 71",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 23984,
            "range": "± 85",
            "unit": "ns/iter"
          },
          {
            "name": "validate_public_url",
            "value": 335,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "reject_private_ip",
            "value": 5323,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 5043,
            "range": "± 83",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 12058,
            "range": "± 52",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "rodmendoza07@gmail.com",
            "name": "Roderick Mendoza",
            "username": "rodmen07"
          },
          "committer": {
            "email": "rodmendoza07@gmail.com",
            "name": "Roderick Mendoza",
            "username": "rodmen07"
          },
          "distinct": true,
          "id": "9b7c75b0d68fe6575c88971dfe3f1326f9a9cd82",
          "message": "docs(changelog): cut v0.21.0 (2026-06-03)\n\nRename the Unreleased section to [0.21.0] now that the version is bumped,\nso the changelog is publish-ready.\n\nCo-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>",
          "timestamp": "2026-06-03T00:34:18-05:00",
          "tree_id": "6c9a06d46480f095d229a0943f12d6ce50a764f0",
          "url": "https://github.com/rodmen07/svccat/commit/9b7c75b0d68fe6575c88971dfe3f1326f9a9cd82"
        },
        "date": 1780465046340,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 11178,
            "range": "± 70",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 21944,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "validate_public_url",
            "value": 289,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "reject_private_ip",
            "value": 4770,
            "range": "± 110",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 4473,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 10971,
            "range": "± 92",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "rodmendoza07@gmail.com",
            "name": "Roderick Mendoza",
            "username": "rodmen07"
          },
          "committer": {
            "email": "rodmendoza07@gmail.com",
            "name": "Roderick Mendoza",
            "username": "rodmen07"
          },
          "distinct": true,
          "id": "20e3b4f18a9cf5d2bcc4fda74955000fb51eb0f5",
          "message": "feat: svccat demo walkthrough + library example; fix Windows stack overflow\n\nAdd `svccat demo`, a zero-setup narrated walkthrough that builds a throwaway\nsample monorepo and runs check/graph/stats against it (`--keep` retains it), plus\n`examples/demo.rs` showing the same flow through the library API.\n\nFix a Windows-only stack overflow: clap's construction of the large `Commands`\nenum exceeded the default 1 MB main-thread stack, so the CLI now runs on a worker\nthread with a 16 MB stack (Linux's 8 MB default hid this in CI and tests).\n\nAlso condense the README by ~75% (per-command deep dives -> `--help`) and stop\ntracking a stray `targetLZiDL5/` cargo directory.\n\nCo-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>",
          "timestamp": "2026-06-03T07:13:03-05:00",
          "tree_id": "2d1153f352ad173db1d2980c741cf18c50c0dd0c",
          "url": "https://github.com/rodmen07/svccat/commit/20e3b4f18a9cf5d2bcc4fda74955000fb51eb0f5"
        },
        "date": 1780489063563,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 12382,
            "range": "± 1234",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 23470,
            "range": "± 168",
            "unit": "ns/iter"
          },
          {
            "name": "validate_public_url",
            "value": 267,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "reject_private_ip",
            "value": 5229,
            "range": "± 73",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 4815,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 12448,
            "range": "± 209",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "rodmendoza07@gmail.com",
            "name": "Roderick Mendoza",
            "username": "rodmen07"
          },
          "committer": {
            "email": "rodmendoza07@gmail.com",
            "name": "Roderick Mendoza",
            "username": "rodmen07"
          },
          "distinct": true,
          "id": "ae04d7eef30873fdf0e693e9387747ac996ce5be",
          "message": "docs: add Buy Me a Coffee funding (Sponsor button + README Support)\n\nAdd .github/FUNDING.yml to enable the repo Sponsor button, plus a Support\nsection in the README linking to buymeacoffee.com/rodmen07.\n\nCo-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>",
          "timestamp": "2026-06-03T07:43:05-05:00",
          "tree_id": "aab0015ed0c0a7acc3e6c5ffb9193a026bf8fe88",
          "url": "https://github.com/rodmen07/svccat/commit/ae04d7eef30873fdf0e693e9387747ac996ce5be"
        },
        "date": 1780490771699,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 12402,
            "range": "± 62",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 23029,
            "range": "± 86",
            "unit": "ns/iter"
          },
          {
            "name": "validate_public_url",
            "value": 268,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "reject_private_ip",
            "value": 5227,
            "range": "± 150",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 4766,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 11887,
            "range": "± 39",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "rodmendoza07@gmail.com",
            "name": "Roderick Mendoza",
            "username": "rodmen07"
          },
          "committer": {
            "email": "rodmendoza07@gmail.com",
            "name": "Roderick Mendoza",
            "username": "rodmen07"
          },
          "distinct": true,
          "id": "73ec0e20c5d0382ec1c4b4595dc53783bbc44949",
          "message": "docs: declutter root, move reference docs to docs/\n\nMove FEATURE_DESIGN_MULTI_REPO, FUZZING, PERFORMANCE_OPTIMIZATIONS_PHASE1, and\nSECURITY_BEST_PRACTICES into docs/. Delete superseded version-stamped artifacts\n(QUICK_REFERENCE, RELEASE_NOTES, RELEASE_SUMMARY, VALIDATION_CHECKLIST, PLANNING,\nand the v0.19.0 SECURITY_ANNOUNCEMENT); their content lives in the CHANGELOG,\nGitHub releases, and git history. Root keeps README, CHANGELOG, and SECURITY.\n\nCo-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>",
          "timestamp": "2026-06-03T16:06:39-05:00",
          "tree_id": "29e5b474b3a702e49065e18f228f91a0d77922e0",
          "url": "https://github.com/rodmen07/svccat/commit/73ec0e20c5d0382ec1c4b4595dc53783bbc44949"
        },
        "date": 1780520989098,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 12561,
            "range": "± 77",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 23362,
            "range": "± 479",
            "unit": "ns/iter"
          },
          {
            "name": "validate_public_url",
            "value": 255,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "reject_private_ip",
            "value": 5209,
            "range": "± 57",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 4790,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 12658,
            "range": "± 28",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "rodmendoza07@gmail.com",
            "name": "Roderick Mendoza",
            "username": "rodmen07"
          },
          "committer": {
            "email": "rodmendoza07@gmail.com",
            "name": "Roderick Mendoza",
            "username": "rodmen07"
          },
          "distinct": true,
          "id": "a36bd4ab5b22a1eb17dd5aa238d34d4f670b009e",
          "message": "release: v0.22.0 - svccat demo, library example, Windows stack fix\n\nCut v0.22.0: `svccat demo` walkthrough, `examples/demo.rs` library example,\nthe Windows main-thread stack-overflow fix, and the ~75% README condense.\n\nCo-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>",
          "timestamp": "2026-06-03T16:08:40-05:00",
          "tree_id": "c3b007668b3006ce2dc589c9a4c8b2bcdd8456ec",
          "url": "https://github.com/rodmen07/svccat/commit/a36bd4ab5b22a1eb17dd5aa238d34d4f670b009e"
        },
        "date": 1780521105155,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 12773,
            "range": "± 196",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 24090,
            "range": "± 104",
            "unit": "ns/iter"
          },
          {
            "name": "validate_public_url",
            "value": 333,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "reject_private_ip",
            "value": 5479,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 5140,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 11908,
            "range": "± 102",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "rodmendoza07@gmail.com",
            "name": "Roderick Mendoza",
            "username": "rodmen07"
          },
          "committer": {
            "email": "rodmendoza07@gmail.com",
            "name": "Roderick Mendoza",
            "username": "rodmen07"
          },
          "distinct": true,
          "id": "18a66fe4943df85fca6b5de7b5c9e51693b72862",
          "message": "ci: make publish workflow idempotent and drop deprecated --token\n\nTolerate an already-published version so a manual `cargo publish` or a re-run\nno longer fails the release workflow with \"already exists\". Also use the\nCARGO_REGISTRY_TOKEN env var instead of the deprecated `cargo publish --token`.\n\nCo-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>",
          "timestamp": "2026-06-03T16:15:37-05:00",
          "tree_id": "09c419ed3816cd3bea30b6b44bbe922a410c9cec",
          "url": "https://github.com/rodmen07/svccat/commit/18a66fe4943df85fca6b5de7b5c9e51693b72862"
        },
        "date": 1780521530160,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 12565,
            "range": "± 101",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 24014,
            "range": "± 253",
            "unit": "ns/iter"
          },
          {
            "name": "validate_public_url",
            "value": 327,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "reject_private_ip",
            "value": 5458,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 5222,
            "range": "± 61",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 12593,
            "range": "± 86",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "rodmendoza07@gmail.com",
            "name": "Roderick Mendoza",
            "username": "rodmen07"
          },
          "committer": {
            "email": "rodmendoza07@gmail.com",
            "name": "Roderick Mendoza",
            "username": "rodmen07"
          },
          "distinct": true,
          "id": "bc200bbec3a701c7058f1597edcebd9010b63b56",
          "message": "release: prepare v0.23.0 - API freeze prep for 1.0\n\nPrepares the public API for a 1.0.0 freeze (last window for breaking\nlibrary changes before 1.0):\n\n- Migrate serde_yaml -> serde_yaml_ng (maintained fork) via Cargo.toml\n  package rename; zero source changes\n- Curate public API: only manifest/discovery/drift/report/config are the\n  stable, documented surface; doc-hide the remaining CLI-plumbing modules\n- Mark core types #[non_exhaustive]; derive Default on Manifest/ServiceEntry\n- Declare MSRV rust-version = 1.85 (clap dependency floor)\n- Add crate-level docs + docs/API_STABILITY.md\n- Stop gitignoring Cargo.lock and commit it for reproducible binary/CI\n  builds; broaden temp-file ignore to *.tmp.*\n\nCo-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>",
          "timestamp": "2026-06-05T06:10:49-05:00",
          "tree_id": "fd4db6deae2fc27528791744e3016d0b2bc23369",
          "url": "https://github.com/rodmen07/svccat/commit/bc200bbec3a701c7058f1597edcebd9010b63b56"
        },
        "date": 1780658244253,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 12651,
            "range": "± 183",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 23687,
            "range": "± 267",
            "unit": "ns/iter"
          },
          {
            "name": "validate_public_url",
            "value": 256,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "reject_private_ip",
            "value": 5093,
            "range": "± 160",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 4818,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 12482,
            "range": "± 51",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "rodmendoza07@gmail.com",
            "name": "Roderick Mendoza",
            "username": "rodmen07"
          },
          "committer": {
            "email": "rodmendoza07@gmail.com",
            "name": "Roderick Mendoza",
            "username": "rodmen07"
          },
          "distinct": true,
          "id": "7d8d8b82189b51a3448743f3672c1258e41a31dc",
          "message": "release: v1.0.0 - first stable release / API freeze\n\nVersion-only bump. No functional or API changes since 0.23.0; promotes the\ncurated public API to a stable 1.x semver guarantee (see docs/API_STABILITY.md).\n\nCo-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>",
          "timestamp": "2026-06-05T07:06:40-05:00",
          "tree_id": "a5117b946565039b64d15bdf19d3c557a903bd93",
          "url": "https://github.com/rodmen07/svccat/commit/7d8d8b82189b51a3448743f3672c1258e41a31dc"
        },
        "date": 1780661385519,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 12925,
            "range": "± 715",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 23976,
            "range": "± 124",
            "unit": "ns/iter"
          },
          {
            "name": "validate_public_url",
            "value": 320,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "reject_private_ip",
            "value": 5528,
            "range": "± 109",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 5190,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 12539,
            "range": "± 45",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "rodmendoza07@gmail.com",
            "name": "Roderick Mendoza",
            "username": "rodmen07"
          },
          "committer": {
            "email": "rodmendoza07@gmail.com",
            "name": "Roderick Mendoza",
            "username": "rodmen07"
          },
          "distinct": true,
          "id": "7b15d4fe282dfc6046b5d928333e55f541c23118",
          "message": "release: v1.0.1 - add homepage & documentation metadata\n\nMetadata-only patch: declares homepage and documentation in Cargo.toml so\ncrates.io renders the Homepage and Documentation links. No code or API changes.\n\nCo-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>",
          "timestamp": "2026-06-05T07:30:58-05:00",
          "tree_id": "683913c7a5c57a5d3e66953ce6fc4949d605b9dc",
          "url": "https://github.com/rodmen07/svccat/commit/7b15d4fe282dfc6046b5d928333e55f541c23118"
        },
        "date": 1780662843793,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 12861,
            "range": "± 55",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 24453,
            "range": "± 280",
            "unit": "ns/iter"
          },
          {
            "name": "validate_public_url",
            "value": 325,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "reject_private_ip",
            "value": 5559,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 5155,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 12665,
            "range": "± 30",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "rodmendoza07@gmail.com",
            "name": "Roderick Mendoza",
            "username": "rodmen07"
          },
          "committer": {
            "email": "rodmendoza07@gmail.com",
            "name": "Roderick Mendoza",
            "username": "rodmen07"
          },
          "distinct": true,
          "id": "0d936c4300faca7e7199cd767c1a221f95ba04ec",
          "message": "release: v1.1.0 - infer language & platform in init/fix",
          "timestamp": "2026-06-07T12:57:04-05:00",
          "tree_id": "916fe69dee8e01edb51d3f461f99aa10d6e22895",
          "url": "https://github.com/rodmen07/svccat/commit/0d936c4300faca7e7199cd767c1a221f95ba04ec"
        },
        "date": 1780855208894,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 12574,
            "range": "± 79",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 23842,
            "range": "± 1113",
            "unit": "ns/iter"
          },
          {
            "name": "validate_public_url",
            "value": 331,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "reject_private_ip",
            "value": 5417,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 5069,
            "range": "± 62",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 12302,
            "range": "± 71",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "rodmendoza07@gmail.com",
            "name": "Roderick Mendoza",
            "username": "rodmen07"
          },
          "committer": {
            "email": "rodmendoza07@gmail.com",
            "name": "Roderick Mendoza",
            "username": "rodmen07"
          },
          "distinct": true,
          "id": "376e446d39105b3d985dd8137320245b4d9909bc",
          "message": "test(output): add formatter payload and helper regression coverage",
          "timestamp": "2026-06-27T11:59:23-05:00",
          "tree_id": "95a2df2980e1896a28dfb7bf474bcc23717b9083",
          "url": "https://github.com/rodmen07/svccat/commit/376e446d39105b3d985dd8137320245b4d9909bc"
        },
        "date": 1782579735764,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 12714,
            "range": "± 364",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 23999,
            "range": "± 131",
            "unit": "ns/iter"
          },
          {
            "name": "validate_public_url",
            "value": 338,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "reject_private_ip",
            "value": 5372,
            "range": "± 46",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 5134,
            "range": "± 119",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 12193,
            "range": "± 188",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "rodmendoza07@gmail.com",
            "name": "Roderick Mendoza",
            "username": "rodmen07"
          },
          "committer": {
            "email": "rodmendoza07@gmail.com",
            "name": "Roderick Mendoza",
            "username": "rodmen07"
          },
          "distinct": true,
          "id": "82584b66cead62291648b7048847cb9d70ac7ed5",
          "message": "chore(release): update lockfile for v1.1.1",
          "timestamp": "2026-06-27T12:22:35-05:00",
          "tree_id": "93ebdbb3f819e6a37f42646c531aaeef5a3b2621",
          "url": "https://github.com/rodmen07/svccat/commit/82584b66cead62291648b7048847cb9d70ac7ed5"
        },
        "date": 1782581135536,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 12716,
            "range": "± 111",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 23784,
            "range": "± 92",
            "unit": "ns/iter"
          },
          {
            "name": "validate_public_url",
            "value": 327,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "reject_private_ip",
            "value": 5562,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 5217,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 12122,
            "range": "± 43",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "rodmendoza07@gmail.com",
            "name": "Roderick Mendoza",
            "username": "rodmen07"
          },
          "committer": {
            "email": "rodmendoza07@gmail.com",
            "name": "Roderick Mendoza",
            "username": "rodmen07"
          },
          "distinct": true,
          "id": "9d246c80d237be6fa5d08d0c41f5eeddc566414f",
          "message": "fix(ci): satisfy clippy items_after_test_module",
          "timestamp": "2026-06-27T12:27:24-05:00",
          "tree_id": "443810fe936b98bea90131f7ec8d417532241e32",
          "url": "https://github.com/rodmen07/svccat/commit/9d246c80d237be6fa5d08d0c41f5eeddc566414f"
        },
        "date": 1782581424668,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 12665,
            "range": "± 54",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 23565,
            "range": "± 94",
            "unit": "ns/iter"
          },
          {
            "name": "validate_public_url",
            "value": 327,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "reject_private_ip",
            "value": 5978,
            "range": "± 78",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 5733,
            "range": "± 209",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 12367,
            "range": "± 39",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "rodmendoza07@gmail.com",
            "name": "Roderick Mendoza",
            "username": "rodmen07"
          },
          "committer": {
            "email": "rodmendoza07@gmail.com",
            "name": "Roderick Mendoza",
            "username": "rodmen07"
          },
          "distinct": true,
          "id": "a406ede38be5bd4fcd0837b1004b8cb4f30dc519",
          "message": "test(output): add shared drift output matrix coverage",
          "timestamp": "2026-06-27T17:56:07-05:00",
          "tree_id": "39cc5332827657a5077076cb6c32aa7ee997f742",
          "url": "https://github.com/rodmen07/svccat/commit/a406ede38be5bd4fcd0837b1004b8cb4f30dc519"
        },
        "date": 1782601153810,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 12923,
            "range": "± 281",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 23555,
            "range": "± 346",
            "unit": "ns/iter"
          },
          {
            "name": "validate_public_url",
            "value": 331,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "reject_private_ip",
            "value": 5495,
            "range": "± 46",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 5132,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 12370,
            "range": "± 453",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "rodmendoza07@gmail.com",
            "name": "Roderick Mendoza",
            "username": "rodmen07"
          },
          "committer": {
            "email": "rodmendoza07@gmail.com",
            "name": "Roderick Mendoza",
            "username": "rodmen07"
          },
          "distinct": true,
          "id": "820788b1359baa867295197fa38ded6083330e16",
          "message": "feat(check): extend --output support for chat formats",
          "timestamp": "2026-06-27T18:14:11-05:00",
          "tree_id": "4cf60069b37d26256d7cf084b8254266f3c96ffc",
          "url": "https://github.com/rodmen07/svccat/commit/820788b1359baa867295197fa38ded6083330e16"
        },
        "date": 1782602230046,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 12914,
            "range": "± 188",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 23968,
            "range": "± 160",
            "unit": "ns/iter"
          },
          {
            "name": "validate_public_url",
            "value": 264,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "reject_private_ip",
            "value": 5136,
            "range": "± 61",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 4846,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 12383,
            "range": "± 60",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}