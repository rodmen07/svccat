window.BENCHMARK_DATA = {
  "lastUpdate": 1784567328046,
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
          "id": "257fea5403a35d2170eaffee49b01a64cf796644",
          "message": "chore(release): sync lockfile for 1.1.3",
          "timestamp": "2026-06-27T18:18:04-05:00",
          "tree_id": "f4bfe8769d85e5a0c576d1d9e8876b6e49c2a1b3",
          "url": "https://github.com/rodmen07/svccat/commit/257fea5403a35d2170eaffee49b01a64cf796644"
        },
        "date": 1782602455683,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 12427,
            "range": "± 326",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 23094,
            "range": "± 66",
            "unit": "ns/iter"
          },
          {
            "name": "validate_public_url",
            "value": 268,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "reject_private_ip",
            "value": 5200,
            "range": "± 59",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 4847,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 12448,
            "range": "± 146",
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
          "id": "a6fc049846063a2be5f6175ca2ec05e13e6a97b1",
          "message": "release: ship svccat 1.1.4",
          "timestamp": "2026-06-27T18:22:48-05:00",
          "tree_id": "e37d7e737d326d2a4500aa8fa1beaa050e1a8eae",
          "url": "https://github.com/rodmen07/svccat/commit/a6fc049846063a2be5f6175ca2ec05e13e6a97b1"
        },
        "date": 1782602750699,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 13021,
            "range": "± 176",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 23929,
            "range": "± 257",
            "unit": "ns/iter"
          },
          {
            "name": "validate_public_url",
            "value": 312,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "reject_private_ip",
            "value": 5508,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 5110,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 12522,
            "range": "± 108",
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
          "id": "fd4778bde1677c285054aff056960a5b9a3c2105",
          "message": "release: ship svccat 1.1.5",
          "timestamp": "2026-06-27T18:32:37-05:00",
          "tree_id": "1ca2b271187adf9f2cc29b7effa39e9ae4609168",
          "url": "https://github.com/rodmen07/svccat/commit/fd4778bde1677c285054aff056960a5b9a3c2105"
        },
        "date": 1782603340184,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 12689,
            "range": "± 209",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 24290,
            "range": "± 102",
            "unit": "ns/iter"
          },
          {
            "name": "validate_public_url",
            "value": 326,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "reject_private_ip",
            "value": 5630,
            "range": "± 62",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 5207,
            "range": "± 55",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 12381,
            "range": "± 117",
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
          "id": "6c9c2259300170f70ed002d81457e9b02f3b8878",
          "message": "release: ship svccat 1.1.7",
          "timestamp": "2026-06-27T18:45:17-05:00",
          "tree_id": "a828ce5129bf00d15342eb1f1c676cea72281c85",
          "url": "https://github.com/rodmen07/svccat/commit/6c9c2259300170f70ed002d81457e9b02f3b8878"
        },
        "date": 1782604070383,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 9636,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 18043,
            "range": "± 67",
            "unit": "ns/iter"
          },
          {
            "name": "validate_public_url",
            "value": 211,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "reject_private_ip",
            "value": 4075,
            "range": "± 55",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 3789,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 9498,
            "range": "± 45",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "117107251+rodmen07@users.noreply.github.com",
            "name": "Roderick Mendoza",
            "username": "rodmen07"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "742d84409660790eafa06716567fbb234698f789",
          "message": "Merge pull request #1 from rodmen07/release/1.2.0\n\nchore(release): v1.2.0 (metadata-only)",
          "timestamp": "2026-07-09T08:16:44-05:00",
          "tree_id": "63aea24c24d96805706da035f177480f765c8290",
          "url": "https://github.com/rodmen07/svccat/commit/742d84409660790eafa06716567fbb234698f789"
        },
        "date": 1783603348509,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 12551,
            "range": "± 125",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 23590,
            "range": "± 112",
            "unit": "ns/iter"
          },
          {
            "name": "validate_public_url",
            "value": 317,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "reject_private_ip",
            "value": 5395,
            "range": "± 55",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 4975,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 12308,
            "range": "± 66",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "117107251+rodmen07@users.noreply.github.com",
            "name": "Roderick Mendoza",
            "username": "rodmen07"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "da5a4243b8ba8656ba10fc4a482bda43bcbd3ced",
          "message": "Merge pull request #2 from rodmen07/release/1.3.0\n\nchore(release): v1.3.2",
          "timestamp": "2026-07-09T08:49:11-05:00",
          "tree_id": "850d4d110b77b253005dc41535e5cc5d213bd39b",
          "url": "https://github.com/rodmen07/svccat/commit/da5a4243b8ba8656ba10fc4a482bda43bcbd3ced"
        },
        "date": 1783605140444,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 12505,
            "range": "± 106",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 23669,
            "range": "± 235",
            "unit": "ns/iter"
          },
          {
            "name": "validate_public_url",
            "value": 300,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "reject_private_ip",
            "value": 5531,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 5270,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 12340,
            "range": "± 32",
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
          "id": "97d3a09db11182a1172a96ad82a5db03ccfb98fa",
          "message": "docs: add release plan for v1.4.0",
          "timestamp": "2026-07-09T09:06:34-05:00",
          "tree_id": "b119ef92815e6e2a148e7fb48c6b1333828c1188",
          "url": "https://github.com/rodmen07/svccat/commit/97d3a09db11182a1172a96ad82a5db03ccfb98fa"
        },
        "date": 1783606179934,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 12581,
            "range": "± 59",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 24010,
            "range": "± 129",
            "unit": "ns/iter"
          },
          {
            "name": "validate_public_url",
            "value": 300,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "reject_private_ip",
            "value": 5444,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 5161,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 12082,
            "range": "± 65",
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
          "id": "d1509df2aa06b4f5684a363e7486bf66db133fd5",
          "message": "chore(release): 1.4.0 — multi-threaded discovery & backstage export",
          "timestamp": "2026-07-09T09:15:18-05:00",
          "tree_id": "0b373e974ea89f3e5c8e5a0a6957f777ccb68933",
          "url": "https://github.com/rodmen07/svccat/commit/d1509df2aa06b4f5684a363e7486bf66db133fd5"
        },
        "date": 1783606701933,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 12659,
            "range": "± 1312",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 23713,
            "range": "± 561",
            "unit": "ns/iter"
          },
          {
            "name": "validate_public_url",
            "value": 288,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "reject_private_ip",
            "value": 5148,
            "range": "± 55",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 4775,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 12318,
            "range": "± 35",
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
          "id": "1bf3b646e58ed138251111b8e524cc5bffb63a25",
          "message": "chore(release): 1.4.1 — resolve security dependencies in Cargo.lock",
          "timestamp": "2026-07-09T09:22:23-05:00",
          "tree_id": "1f86fae834533f77b0f599a1a3178815ef2fdacd",
          "url": "https://github.com/rodmen07/svccat/commit/1bf3b646e58ed138251111b8e524cc5bffb63a25"
        },
        "date": 1783607131956,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 12620,
            "range": "± 57",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 23410,
            "range": "± 94",
            "unit": "ns/iter"
          },
          {
            "name": "validate_public_url",
            "value": 329,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "reject_private_ip",
            "value": 5396,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 5087,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 12353,
            "range": "± 46",
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
          "id": "2a4378dcbbe27ffc5070a13e6379f48cda5fa67d",
          "message": "chore(workflows): update GitHub Actions workflow files",
          "timestamp": "2026-07-13T06:08:20-05:00",
          "tree_id": "ee5a9c1b725a22dbcfe2df63dfb8e9551cf644f5",
          "url": "https://github.com/rodmen07/svccat/commit/2a4378dcbbe27ffc5070a13e6379f48cda5fa67d"
        },
        "date": 1783941081191,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 12524,
            "range": "± 124",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 23841,
            "range": "± 174",
            "unit": "ns/iter"
          },
          {
            "name": "validate_public_url",
            "value": 261,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "reject_private_ip",
            "value": 5271,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 4694,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 12403,
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "60c56b2fee1130f4e3b5e31aeda635f2051f9d77",
          "message": "Merge pull request #3 from rodmen07/feat/spdx-sbom\n\nv1.5.0: SPDX 2.3 JSON SBOM export + snapshot --sbom sidecar",
          "timestamp": "2026-07-18T17:49:46-05:00",
          "tree_id": "12b19323c472598d93bbc522e3f7766f00be4280",
          "url": "https://github.com/rodmen07/svccat/commit/60c56b2fee1130f4e3b5e31aeda635f2051f9d77"
        },
        "date": 1784415187693,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 12479,
            "range": "± 62",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 23628,
            "range": "± 165",
            "unit": "ns/iter"
          },
          {
            "name": "validate_public_url",
            "value": 313,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "reject_private_ip",
            "value": 5507,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 5143,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 12203,
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "bb907d8d7c46fa04fffc153db990c669eefa6059",
          "message": "feat(multi-repo): workspace config completion and repo filtering (multi-repo slice 1) (#4)\n\nPhases 1-4 of docs/FEATURE_DESIGN_MULTI_REPO.md shipped in v0.21.0, so the\ndesign is now sliced over the unshipped remainder, recorded as a checkbox\nlist at the top of the doc:\n\n  1. Workspace config completion and repo filtering (this commit)\n  2. [reporting] config section (format default, include_cross_repo_deps,\n     exclude_patterns merged into ignore globs)\n  3. workspace check --format html interactive visualization (already a\n     ROADMAP.md Later / candidates item)\n\nSlice 1 delivers:\n- Parse [workspace] name and description from svccat.toml into\n  WorkspaceConfig; both default to None.\n- Carry the workspace name into WorkspaceDriftReport and all three\n  renderers: terminal header line, workspace_name JSON field, markdown\n  Workspace line.\n- Wire the previously parsed-but-ignored workspace check --filter flag:\n  comma-separated repo names, whitespace trimmed, duplicates deduped,\n  unknown names rejected with the list of available repos (exit 2).\n  Filtering selects among configured repos; enabled = false still skips.\n\nTests: 7 new unit tests in src/workspace.rs and 5 new integration tests\nin tests/workspace_integration_tests.rs (202 to 214 passing).\n\nCo-authored-by: Claude Fable 5 <noreply@anthropic.com>",
          "timestamp": "2026-07-19T12:53:10-05:00",
          "tree_id": "47a7d9a8982bbc0595ab24d66339cd1e7f935506",
          "url": "https://github.com/rodmen07/svccat/commit/bb907d8d7c46fa04fffc153db990c669eefa6059"
        },
        "date": 1784483779997,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 11283,
            "range": "± 54",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 22231,
            "range": "± 165",
            "unit": "ns/iter"
          },
          {
            "name": "validate_public_url",
            "value": 282,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "reject_private_ip",
            "value": 4870,
            "range": "± 61",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 4497,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 10884,
            "range": "± 17",
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "c4fc702e93dadb26be2dd531e2c874ed3523a96e",
          "message": "feat(workspace): [reporting] config section with CLI-over-config precedence (multi-repo slice 2) (#5)\n\nAdd the `[reporting]` section to workspace `svccat.toml`, providing\nconfig-driven defaults for `workspace check`:\n\n- `format`: default output format. Precedence is `--format` (CLI) over\n  `[reporting].format` over the hard-coded terminal default. The config\n  value is validated against the same set clap accepts for `--format`, so\n  the flag and the config can never drift apart.\n- `include_cross_repo_deps` (default true): when false, the cross-repo\n  dependency graph is never built. The toggle removes the work rather than\n  hiding output, so it is a genuine cost knob.\n- `exclude_patterns`: merged additively into the existing discovery ignore\n  globs alongside `--ignore` and the manifest's own `discovery.ignore`; no\n  second glob engine.\n\nParsing, validation, precedence, and the glob merge live in a new focused\n`src/reporting.rs` module. Unknown keys inside `[reporting]` are ignored and\nmistyped known keys are rejected, matching how the rest of svccat.toml is\nparsed. `main.rs` gains only thin wiring (the resolver moved out of it).\n\nTests: 17 unit tests in the new module (parsing, per-key precedence,\nglob-merge semantics, value validation), 7 integration tests in a new\n`tests/reporting_config_tests.rs` (exclude flows through real discovery,\ntoggle skips the graph build with `dependency_summary` as the evidence,\nformat resolution end to end), plus loader-integration coverage in\nworkspace.rs.\n\nCo-authored-by: Claude Fable 5 <noreply@anthropic.com>",
          "timestamp": "2026-07-20T09:17:01-05:00",
          "tree_id": "53bf2df0bc65050bc220738309c91f4117f9ebbc",
          "url": "https://github.com/rodmen07/svccat/commit/c4fc702e93dadb26be2dd531e2c874ed3523a96e"
        },
        "date": 1784557207037,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 12641,
            "range": "± 458",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 23830,
            "range": "± 138",
            "unit": "ns/iter"
          },
          {
            "name": "validate_public_url",
            "value": 311,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "reject_private_ip",
            "value": 5497,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 5134,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 12585,
            "range": "± 123",
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "8f625fcdc6d97a6c042600bac4aa2edfaa147994",
          "message": "feat(workspace): add HTML output format to workspace check (multi-repo slice 3) (#6)\n\n* feat(workspace): add HTML output format to workspace check (multi-repo slice 3)\n\nAdd `workspace check --format html`: a self-contained HTML report covering\nevery repo's summary and drift table plus a cross-repo dependency graph,\ncompleting the FEATURE_DESIGN_MULTI_REPO.md implementation slices and the\nROADMAP.md \"Later / candidates\" item of the same name.\n\n- `Html` joins the shared `OutputFormat` enum used by both `check` and\n  `workspace check`, so it slots into the precedence machinery slice 2\n  established (`--format` over `[reporting].format` over the terminal\n  default) instead of a parallel path. Since the enum is shared, `check\n  --format html` also gets a renderer: it reuses the existing single-repo\n  `report::render_html` rather than standing up a second HTML renderer for\n  the same (Manifest, DriftReport) pair.\n- New `src/output/workspace_html.rs` renders the multi-repo report: per-repo\n  summary/drift tables, a dependency-analysis section, and (when cross-repo\n  dependency analysis is on) a D3.js v7 force-directed graph reusing the same\n  layout/interaction model as `svccat graph --format html`\n  (`output::mermaid::render_html_graph`), restyled into a bounded panel and\n  coloured by repo instead of platform.\n- `workspace::analyze_workspace` now retains the built dependency graph's\n  nodes (`WorkspaceDriftReport::dependency_graph_nodes`) alongside the\n  existing summary/circular/unresolvable fields, so the HTML renderer draws\n  the real topology without reloading every manifest and rebuilding the graph\n  a second time.\n- Two escaping mechanisms, matching the two trust boundaries repo-sourced\n  text crosses: plain HTML text/attributes (repo, service, team names, drift\n  messages) go through `report::esc` (now `pub(crate)`, shared with the\n  single-repo report via an extracted `REPORT_STYLE` constant); the graph's\n  node/link data is embedded inside a `<script>` element instead, where\n  HTML-escaping alone doesn't stop a value containing a literal `</script>`\n  from closing the element early. That data is routed through the new\n  `src/output/json_script.rs` helper, which JSON-encodes and then neutralizes\n  `<`, `>`, `&` to their `\\uXXXX` forms — safe in both a `JSON.parse` data\n  island and an inlined JS literal, and provably inert since those characters\n  never appear in JSON's own structural syntax.\n\nTests: 242 to 253 (11 new: 6 in workspace_html.rs incl. two proving a\n`<script>`-shaped repo/service name renders as inert text in both the plain\nHTML and the graph's `<script>` data island; 4 in json_script.rs proving the\nescape is reversible and neutralizes a script-breakout payload; 1 in main.rs\ncovering `check --format html`). Manually verified end to end against a\ncrafted workspace with a `</script><script>alert(...)</script>`-named\nservice and a cross-repo dependency: renders as inert `&lt;/script&gt;...`\ntext in drift tables and `</script>...` in the graph JSON, with the\ncross-repo edge correctly resolved.\n\nNo version bump: slices 2 and 3 accumulate into the next minor per the\nexisting convention (CHANGELOG.md, Cargo.toml untouched).\n\nCo-authored-by: Claude Fable 5 <noreply@anthropic.com>\n\n* fix(workspace-html): escape D3 tooltip innerHTML and dedupe graph renderer\n\nAdversarial review of the workspace check --format html PR found the D3\ndependency-graph tooltip writing untrusted repo/service names straight into\nElement.innerHTML via a template literal. json_script::embed and esc()\ncorrectly protect the JSON data island and the plain HTML tables, but\nJSON.parse reverses that encoding on the client before the tooltip handler\nruns, so a service named <img src=x onerror=alert(1)> executes on hover.\n\nThe same bug already existed in mermaid.rs::render_html_graph, which this\nreport's docs claimed to reuse but didn't: it was a second, independently\nmaintained ~70-line D3 script with its own copy of the same tooltip sink and\ndrifted layout constants.\n\n- Add src/output/d3_force_graph.rs: the single D3 force-graph script shared\n  by both renderers. An escHtml() JS helper is the one place tooltip fields\n  reach innerHTML, applied by render_script() itself so a call site can't\n  opt out. Per-call-site differences (panel size, colour field, tooltip\n  content, layout constants) are named, documented D3GraphConfig fields\n  instead of copy-pasted magic numbers.\n- workspace_html.rs::render_graph_panel and mermaid.rs::render_html_graph\n  now both build their <script> body via d3_force_graph::render_script.\n- New tests assert every configured tooltip field is escHtml-wrapped on the\n  tip.innerHTML assignment line, covering both the mechanism and a call site\n  adding fields it forgets to escape.\n\nVerified: cargo fmt --check, cargo clippy --all-targets --all-features\n-- -D warnings, cargo test --all-features all green (114 lib tests + full\nintegration suite, including the PR's existing malicious-name tests).\n\nCo-Authored-By: Claude Sonnet 5 <noreply@anthropic.com>\n\n---------\n\nCo-authored-by: Claude Fable 5 <noreply@anthropic.com>",
          "timestamp": "2026-07-20T10:36:42-05:00",
          "tree_id": "fc78eafdc959bdbfdef6750bef989e3f26f8f0a1",
          "url": "https://github.com/rodmen07/svccat/commit/8f625fcdc6d97a6c042600bac4aa2edfaa147994"
        },
        "date": 1784561987380,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 12663,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 23518,
            "range": "± 1440",
            "unit": "ns/iter"
          },
          {
            "name": "validate_public_url",
            "value": 267,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "reject_private_ip",
            "value": 5244,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 4814,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 12269,
            "range": "± 26",
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "e97a67be926b603010e2045f8abf02721e8ee26f",
          "message": "fix(security): close DOM-based XSS in mermaid.rs graph --format html (#7)\n\n`src/output/mermaid.rs::render_html_graph` (svccat graph --format html)\nbuilt its nodes_json/links_json by interpolating raw Rust `{:?}`\nDebug-format strings, which does not escape `<`, `>`, or `&`. A\nservice/team/platform/language name containing a literal `</script>`\nclosed the surrounding <script> element early and injected live markup\n- the same vulnerability class PR #6 (commit 07b0485) fixed in\nworkspace_html.rs's D3 data island, left unpatched here because that PR\nunified the two renderers' D3 *script* (drag/tick/tooltip, already\nrouted through the shared d3_force_graph.rs since that PR) but not\ntheir *data-embedding* path.\n\nFix: build the node/link data as a typed, Serialize-derived D3Graph and\nembed it via the existing json_script::embed (JSON-encode, then\n`\\uXXXX`-escape `<`/`>`/`&`), the same mechanism workspace_html.rs\nalready uses. The JSON now lives in a `<script type=\"application/json\">`\ndata island, parsed via JSON.parse client-side, matching\nworkspace_html.rs's own pattern exactly.\n\nRegression test mermaid.rs::tests::malicious_service_name_in_graph_data_cannot_close_the_script_tag\nmirrors json_script.rs's script_breakout_attempt_is_neutralized and\nworkspace_html.rs's malicious_service_name_in_graph_data_cannot_close_the_script_tag\nat the same rigor; verified it fails against the pre-fix code (raw\npayload survives unescaped) before applying the fix, and passes after.\ngraph_data_json_island_round_trips_through_json_parse proves the new\ndata path still carries real data, not just that it's unreachable.\n\nAlso bundles the two LOW findings filed alongside this one in the same\nreview pass, both trivial:\n- report.rs::esc() now escapes `'` to `&#39;` (defense-in-depth; no\n  call site currently writes single-quoted attributes, verified by\n  grep). Regression test in integration_test.rs proven to fail\n  pre-fix.\n- main.rs: extracted the inline `workspace check` format-dispatch match\n  arm into render_workspace_check_output_to_string, mirroring\n  render_check_output_to_string's existing shape, and added three unit\n  tests (html/json+markdown/terminal-skip) mirroring the existing\n  string_output_helper_supports_* tests. Previously this dispatch arm\n  was only exercised indirectly through workspace_html.rs's own unit\n  tests; a regression in the match arm itself (wrong format falling\n  through, Html routed to the wrong renderer) would have slipped\n  through undetected.\n\nNo version bump: continues the slices 2-3 accumulation convention\n(this DevSecOps fix targets a defect in already-released 1.5.0\nbehavior rather than gating a new feature, so it rides along rather\nthan forcing an off-cycle patch release).\n\nTests: 256 -> 262 (all --all-features suites, lib + bin + every\nintegration file + doctest), 0 failed.\n\nCode health: main.rs 1094 -> 1168 lines, tests/integration_test.rs\n2052 -> 2093 lines. Both were already over the 1000-line hard\nthreshold before this change (preflight C10, filed 2026-07-20).\nGrowth wasn't avoidable by extraction here: main.rs's format-dispatch\nhelpers and their tests are private to the `svccat` binary target, so\nonly an in-file unit test can reach them (tests/integration_test.rs\ncompiles as a separate crate that only sees the `svccat` *library*'s\npub API); the integration_test.rs addition is one black-box test\nfollowing the file's existing report_html_contains_html_structure\nprecedent exactly. No refactor attempted here - that's a separate,\ntrigger-based increment per the code-health bar, not bundled into a\nsecurity fix.\n\nLessons applied: L-001 (behavior-difference test proven at\nsrc/output/mermaid.rs::tests::malicious_service_name_in_graph_data_cannot_close_the_script_tag\nand src/report.rs's esc() fix via\ntests/integration_test.rs::report_html_escapes_single_quotes_in_service_fields\n- both verified failing pre-fix and passing post-fix by temporarily\nreverting each fix and re-running the test).\n\nCo-authored-by: Claude Sonnet 5 <noreply@anthropic.com>",
          "timestamp": "2026-07-20T11:33:16-05:00",
          "tree_id": "9015392c9ce7c0583b1956c4755ca2e358989f57",
          "url": "https://github.com/rodmen07/svccat/commit/e97a67be926b603010e2045f8abf02721e8ee26f"
        },
        "date": 1784565378181,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 12389,
            "range": "± 157",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 23794,
            "range": "± 106",
            "unit": "ns/iter"
          },
          {
            "name": "validate_public_url",
            "value": 315,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "reject_private_ip",
            "value": 5417,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 5036,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 12557,
            "range": "± 44",
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "23cccff6e1a9b260b526c2b0ccac524d5ba77f65",
          "message": "test(cli): add binary-level integration tests via assert_cmd (#8)\n\nTwo adversarial security reviews (PR #6, PR #7) both flagged the same\ngap: this codebase had zero tests that spawn the actual compiled\nsvccat binary. Every existing test for the CLI's format-dispatch\nmatch arms (workspace check --format json/markdown/html, graph\n--format) calls the extracted render functions directly in-process,\nwhich main.rs's own doc comments admit exist specifically to avoid\nspawning the binary. That leaves a real regression in the wiring\nitself (a match arm falling through to the wrong branch, a rendered\nstring never reaching stdout) able to compile clean and pass every\nexisting test.\n\nAdds assert_cmd + predicates as dev-dependencies (the conventional\npair for this in the Rust CLI ecosystem) and tests/cli_binary_tests.rs\nwith real binary-level coverage for:\n- workspace check --format html/json/markdown (the command both\n  reviews specifically worried about), reusing the existing\n  tests/fixtures/workspace fixture rather than a parallel scheme\n- workspace check's default terminal format, proving the None-arm\n  fallback actually prints instead of silently discarding\n- svccat graph --format html (the sibling command with today's XSS\n  fix), including a binary-level run of the exact script-breakout\n  payload mermaid.rs's unit test already covers in-process, now\n  proven through real CLI parsing + disk I/O + stdout\n- an unknown-subcommand exit-code sanity check\n\nVerified these tests catch what unit tests can't: temporarily\ndisabling the workspace-check stdout print in main.rs failed 3 of\nthe new binary tests while all 6 existing main.rs unit tests (which\ncall the render function directly) stayed green.\n\nTests 262 -> 270. fmt/clippy/test --all-features all clean.\n\nCo-authored-by: Claude Sonnet 5 <noreply@anthropic.com>",
          "timestamp": "2026-07-20T12:06:01-05:00",
          "tree_id": "ff089c9acd13420c7c1c6748e3c970113ac55406",
          "url": "https://github.com/rodmen07/svccat/commit/23cccff6e1a9b260b526c2b0ccac524d5ba77f65"
        },
        "date": 1784567327113,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 9781,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 18541,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "validate_public_url",
            "value": 205,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "reject_private_ip",
            "value": 4003,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 3735,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 9416,
            "range": "± 12",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}