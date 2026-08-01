window.BENCHMARK_DATA = {
  "lastUpdate": 1785614031638,
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
          "id": "8c6dc20c71934732bc1753907e32284a4d428b83",
          "message": "ci(gap): build and test this checkout, not just the published crate (#10)\n\n* ci(gap): build and test THIS checkout, not just the published crate\n\nVerify Registry Deployment (9-way OS x toolchain) and the drift-check\njob in svccat-pr.yml both `cargo install svccat` from crates.io, so\nneither ever compiles or tests a PR's own diff. Only coverage.yml\n(ubuntu/nightly) touched a PR's real code, so 12 of the 13 required\nchecks would pass identically even if new code were broken.\n\nAdd a new \"Build & Test (This Checkout)\" job to ci.yml: one runner per\nOS (ubuntu/windows/macos) on stable, running `cargo build --all-features`\nthen `cargo test --all-features` against the actual checkout, cached\nwith Swatinem/rust-cache (this repo's existing convention). Scoped to\nstable-per-OS rather than a second 9-way matrix: coverage.yml already\ncovers ubuntu/nightly against real code, so this closes the two OSes\n(windows, macos) that had zero real-code coverage, without doubling\nthis workflow's billed runner-minutes.\n\nThe existing \"Verify Registry Deployment\" job is untouched: it answers\na real, different question (does the published release still install\nand run on this OS/toolchain) and keeps doing exactly that.\n\nCo-Authored-By: Claude Sonnet 5 <noreply@anthropic.com>\n\n* fix(ci): build svccat-pr.yml's drift check from this checkout, not crates.io\n\nAdversarial review on PR #10 found that svccat-pr.yml still ran\n`cargo install svccat --locked`, which always fetches the last-published\ncrates.io binary and never this checkout's own code. So any PR that\nbreaks compilation or regresses check/drift-detection logic would still\nshow a green required \"Service catalog drift check\" status (demonstrated\nconcretely in scratch PR #9, run 29763368089, with a deliberately broken\nsrc/main.rs).\n\nFix: `cargo install --path . --locked --force` builds and installs the\nPR's own binary instead.\n\n---------\n\nCo-authored-by: Claude Sonnet 5 <noreply@anthropic.com>",
          "timestamp": "2026-07-20T13:09:27-05:00",
          "tree_id": "8a8961844acd2084959919838b5552d78bc27ddc",
          "url": "https://github.com/rodmen07/svccat/commit/8c6dc20c71934732bc1753907e32284a4d428b83"
        },
        "date": 1784571168294,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 12553,
            "range": "± 83",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 23542,
            "range": "± 84",
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
            "value": 5462,
            "range": "± 154",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 5169,
            "range": "± 216",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 12268,
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
          "id": "4202db6825a6c18c66be7ecdcd70f45036e70dcc",
          "message": "feat(export): CycloneDX 1.7 JSON SBOM export as a sibling of spdx-json (#11)\n\n* feat(export): add CycloneDX 1.7 JSON SBOM export as a sibling of spdx-json\n\n`svccat export --format cyclonedx-json` renders the service catalog as a\nCycloneDX 1.7 JSON software bill of materials, following the exact same\narchitectural pattern as the existing SPDX 2.3 exporter:\n\n- Same trigger mechanism: a new `ExportFormat::CyclonedxJson` value on the\n  existing `export --format` flag, not a new subcommand or flag shape.\n- Same data source: `Manifest`/`ServiceEntry`, the identical in-memory model\n  every other svccat renderer reads. No parallel manifest-loading path.\n- Same determinism seam: a `render_at(manifest, secs, subsec_nanos, pid)`\n  inner function with a `render_export` wall-clock wrapper, mirroring\n  `spdx::render_at`/`render_export` so tests can pin output exactly.\n\nCycloneDX-specific shape: `bomFormat`/`specVersion`/`serialNumber` (a\n`urn:uuid:` v4 UUID synthesized deterministically from the same\nversion/time/pid seed SPDX's `documentNamespace` uses, no new `uuid` or\n`rand` dependency), `metadata.timestamp`/`metadata.tools.components`, one\n`application` component per service with `purl`, and a `dependencies` graph\nentry for every component (including dependency-free ones, per the spec's\nown recommendation) built from `depends_on` edges. `platform` has no\nfirst-class CycloneDX field, so it goes into `properties` (CycloneDX's own\nextension slot) instead of SPDX's `OTHER` external-ref stretch.\n\nSchema version: 1.7 is the newest full CycloneDX schema (released\n2026-02-25; 1.7.1 is an errata-only patch of the same schema). Verified\nindependently, not just eyeballed: fetched the real\n`CycloneDX/specification` bom-1.7.schema.json and validated three generated\nsamples (a multi-service catalog with dependencies, an empty catalog, and a\nservice name with Unicode/emoji characters) against it with a standalone\n`jsonschema`-crate validator, including resolving the schema's external\n`jsf-0.82.schema.json` vocabulary reference over HTTP — all three came back\nschema-valid.\n\nTests mirror `output::spdx`'s rigor (11 unit tests, up from SPDX's 8):\ndocument shape, camelCase key casing, bom-ref sanitization and collision\nhandling, serial-number determinism/uniqueness/urn-validity, dependency\ngraph completeness, empty-catalog arrays-present, unresolved depends_on\nskipping, component field mapping, purl percent-encoding (including\nmulti-byte UTF-8), and a dedicated Unicode/emoji service-name edge case.\nPlus a `tests/cyclonedx_export_tests.rs` integration file mirroring\n`tests/spdx_export_tests.rs`'s discovered-manifest and CLI-surface coverage.\nFixed a real bug found while adding these: a carried-over test asserted the\nUUID version/variant nibbles at hardcoded string indices that were off by 4;\ncorrected with a documented layout derivation instead of magic numbers.\n\nNew code lives in its own module (`src/output/cyclonedx.rs`) and its own\ntest file rather than growing `src/main.rs` or `tests/integration_test.rs`,\nboth already-flagged code-health hotspots. No new runtime dependency:\nserde_json (already a dependency) is sufficient.\n\nTests: 270 -> 284.\n\nCo-Authored-By: Claude Sonnet 5 <noreply@anthropic.com>\n\n* fix(export): stop duplicate service names from corrupting CycloneDX dependencies\n\nbom_ref_by_name was keyed by svc.name and overwritten on every insert, so a\nmanifest with two services sharing a name collapsed both of their\ndependencies-array entries onto whichever bom-ref was assigned last. That\nproduced two byte-identical {\"ref\": ...} objects (violating the schema's\nuniqueItems constraint on `dependencies`) while the first duplicate's\ncomponent was left with no dependency-graph entry at all, contradicting the\nmodule's own \"entry for every component\" invariant.\n\nEach component's own dependencies entry is now taken from a positional\nbom_refs_by_index vector built alongside the components loop, so it is\nalways the bom-ref actually assigned to that specific service, never a\nname-keyed lookup. depends_on edges still resolve dependency names via the\nname-keyed map, now first-occurrence-wins instead of last-write-wins, which\nis the best achievable resolution for a name that identifies more than one\nservice without rejecting the manifest outright (Manifest::load does not\nrequire unique names; only the opt-in `svccat lint` flags that).\n\nAdds a regression test with two same-named services asserting the\ndependencies array has one entry per component with pairwise-distinct refs.\n\n---------\n\nCo-authored-by: Claude Sonnet 5 <noreply@anthropic.com>",
          "timestamp": "2026-07-20T15:00:10-05:00",
          "tree_id": "999cade24f3c108ecfe6419a3d64286269e89605",
          "url": "https://github.com/rodmen07/svccat/commit/4202db6825a6c18c66be7ecdcd70f45036e70dcc"
        },
        "date": 1784577802028,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 12338,
            "range": "± 618",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 22894,
            "range": "± 219",
            "unit": "ns/iter"
          },
          {
            "name": "validate_public_url",
            "value": 266,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "reject_private_ip",
            "value": 5128,
            "range": "± 89",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 4714,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 11931,
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
          "id": "da3d537b7b7dd2d00a1e46b0314f57291132f928",
          "message": "feat(lint): validate inline policy rule schema before it reaches the compiler (#12)\n\nsvccat lint never looked at manifest.policy.rules at all, so a malformed\npolicy rule (duplicate id, dangling `base`, bad severity, unparsable\nexpression) was silently accepted by lint and only surfaced later as a\nswallowed eprintln! warning inside `svccat check`/`workspace check` -\nthe command still exits 0 and every policy rule is disabled for that run.\n\nWorse: a `base` chain that forms a cycle (a rule naming itself, or two\nrules naming each other) isn't merely unvalidated, it crashes the whole\nprocess. RuleEngine::compile's inheritance resolver recurses through the\nbase chain with no cycle guard; verified directly with a throwaway repro\nthat a single self-referencing rule passed to RuleEngine::compile\nterminates the process with STATUS_STACK_OVERFLOW (0xc00000fd on\nWindows) instead of returning an Err.\n\nNew focused module src/rule_schema.rs runs cheap structural checks first\n(blank/duplicate rule ids, dangling base references, and - the check with\nno prior coverage anywhere - base-chain cycle detection via an iterative\nwalk, since the existing resolver's recursion is exactly what a cycle\ninput must never reach) and only delegates to the existing\nRuleEngine::compile for its semantic checks (severity enum, expression\nsyntax) once the structure is confirmed safe to resolve. rules.rs's own\nerror messages are also tightened to name the offending rule id, since\nneither the severity nor the expression-parse error case did before.\n\nTests 285 to 301 (all green before and after, verified by stashing this\nchange and re-running the full suite): 9 unit tests in\nsrc/rule_schema.rs covering each check in isolation, plus 7 binary-level\ntests in the new tests/policy_rule_schema_tests.rs spawning the real\nsvccat lint binary end to end (valid rules pass, duplicate ids/dangling\nbase/self-cycle/mutual-cycle/bad severity all fail with a specific\nmessage naming the offending rule, no-policy-block stays clean).\n\nCo-authored-by: Claude Sonnet 5 <noreply@anthropic.com>",
          "timestamp": "2026-07-20T15:57:29-05:00",
          "tree_id": "d2f37995b4afe09ed61ca1dfd1204b8e02f47d1c",
          "url": "https://github.com/rodmen07/svccat/commit/da3d537b7b7dd2d00a1e46b0314f57291132f928"
        },
        "date": 1784581240904,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 12634,
            "range": "± 143",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 23966,
            "range": "± 120",
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
            "value": 5605,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 5284,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 11915,
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "36a58aaaa95f6c81ff1d49cd9c724b8760bec765",
          "message": "docs(roadmap): move three shipped items out of Later/candidates (#13)\n\nVerified against real git/gh state (gh pr view, git log) before editing:\n\n- Policy rule schema validation folded into `svccat lint` shipped via\n  PR #12 (squash da3d537).\n- `workspace check --format html` interactive visualization shipped via\n  PR #6 (squash 8f625fc), hardened by PR #7 (XSS fix, e97a67b), PR #8\n  (CLI integration tests, 23cccff), and PR #10 (CI now builds/tests this\n  checkout, 8c6dc20).\n- CycloneDX JSON export as a sibling to spdx-json shipped via PR #11\n  (squash 4202db6).\n\nRecorded in History and supersession with their real PR numbers and\nmerge commits, per the document's existing convention for retiring\ncarried-forward items.\n\nThe fourth candidate, SSRF redirect-hardening for --ping, was checked\nagainst src/ping.rs and src/webhook.rs and is genuinely still unshipped:\nboth validate the URL once before the request, but ureq's default\nconfig follows redirects without re-validating the target host per hop.\nLeft in Later/candidates with that finding recorded inline.\n\nDocs-only change; no code, tests, or CI behavior affected.",
          "timestamp": "2026-07-20T16:13:28-05:00",
          "tree_id": "0546fe424075635bd110c66f31cb9997fe52002d",
          "url": "https://github.com/rodmen07/svccat/commit/36a58aaaa95f6c81ff1d49cd9c724b8760bec765"
        },
        "date": 1784582195760,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 12588,
            "range": "± 171",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 23703,
            "range": "± 63",
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
            "value": 5456,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 5103,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 12323,
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "f84016164bb0fc7a8afcc84ae47e905bf806af15",
          "message": "fuzz: make the fuzzing harness real (it could never have built) (#15)\n\nThe previous setup could not have worked:\n\n- `Cargo.fuzz.toml` at the repo root is not a layout cargo-fuzz uses; it\n  expects `fuzz/Cargo.toml`. Nothing consumed the file.\n- It declared `svccat = { path = \".\", features = [\"__fuzz_target\"] }`, but\n  svccat's Cargo.toml has no `[features]` section at all and\n  `__fuzz_target` appears nowhere in the source, so resolution would have\n  failed on an unknown feature regardless.\n- The workflow matrixed over `[libfuzzer, afl, honggfuzz]` engines rather\n  than over actual fuzz targets.\n\nReplaced with the standard cargo-fuzz layout: `fuzz/Cargo.toml` (with the\n`[workspace]` stanza that keeps it out of the parent workspace), a\ncommitted `fuzz/Cargo.lock`, `fuzz/.gitignore` for target/corpus/\nartifacts/coverage, and a workflow matrixing over the three real targets\n(fuzz_manifest, fuzz_url, fuzz_glob). Fuzzing stays on push/schedule/\ndispatch and is deliberately not a per-PR gate.\n\nfuzz_manifest now drives parse-then-compile, mirroring how `svccat check`\nactually uses the pipeline via src/drift.rs, instead of only fuzzing YAML\nshape. That widening is what makes the target able to reach\nRuleEngine::compile's inheritance resolver.\n\nCo-authored-by: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-07-21T13:41:42-05:00",
          "tree_id": "aa512c60c6353c7bf05b8d6060e900eac7b08cfe",
          "url": "https://github.com/rodmen07/svccat/commit/f84016164bb0fc7a8afcc84ae47e905bf806af15"
        },
        "date": 1784659489216,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 12574,
            "range": "± 277",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 23411,
            "range": "± 82",
            "unit": "ns/iter"
          },
          {
            "name": "validate_public_url",
            "value": 288,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "reject_private_ip",
            "value": 5137,
            "range": "± 60",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 4793,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 12278,
            "range": "± 118",
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
          "id": "32f2bca9191c10effffdaccafe4854bd4e123efd",
          "message": "fix(rules): base-chain cycle crashed `svccat check` with a stack overflow (#16)\n\n`RuleEngine::resolve_rule` recursed over each rule's `base` chain with no\ncycle guard and no depth limit. A manifest whose rule names itself as its\nown base, or two rules naming each other, recursed until the stack was\nexhausted — a process abort (STATUS_STACK_OVERFLOW / SIGSEGV), not a\ncatchable `Err`.\n\nIt was reachable from untrusted input. `src/drift.rs` calls\n`RuleEngine::compile` directly, which is the `svccat check` and\n`svccat workspace check` path, while the cycle guard that already existed\n(`validate_no_base_cycles` in `src/rule_schema.rs`) is only invoked from\n`src/lint.rs`. So `lint` was protected and `check` was not, on a\npublished crate, against a manifest the user did not author.\n\n`resolve_rule` now threads the chain of ids currently being resolved and\nreturns a normal error naming the cycle. Behavior for acyclic input is\nunchanged.\n\nFive regression tests, all of which abort the test process with\nSTATUS_STACK_OVERFLOW when the guard is disabled: self-referential base,\nmutual pair, three-hop cycle, plus two that stop the fix from\nover-correcting — an acyclic a->b->c chain must still compile, and a\ndangling base must still report \"not found\" rather than being\nmisreported as a cycle.\n\nFound by the fuzz-harness rework (#15), which widened fuzz_manifest from\nparsing YAML to parse-then-compile, matching how `check` actually uses\nthe pipeline.\n\nCo-authored-by: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-07-21T14:03:18-05:00",
          "tree_id": "3ca1e320a5440153c56ba2bb20009f8353140b13",
          "url": "https://github.com/rodmen07/svccat/commit/32f2bca9191c10effffdaccafe4854bd4e123efd"
        },
        "date": 1784660785841,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 12723,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 23139,
            "range": "± 543",
            "unit": "ns/iter"
          },
          {
            "name": "validate_public_url",
            "value": 266,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "reject_private_ip",
            "value": 5173,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 4826,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 12152,
            "range": "± 27",
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
          "id": "c9250004c419b0a4d5b1438dcdad1acffc62c386",
          "message": "fix(security): close SSRF-via-redirect gap in --ping and webhooks (#14)\n\n* fix(security): close SSRF-via-redirect gap in --ping and webhooks\n\nureq (used for both --ping and webhook POSTs) follows HTTP redirects\nautomatically by default (AgentBuilder::redirects defaults to 5), but\nurlvalidation::validate_url was only ever called once, against the\ninitial destination URL, before the first request went out. A server\nthat responded to an initially-valid, public-looking URL with a 3xx\nredirect to a private/internal address (the cloud metadata endpoint\n169.254.169.254, an internal service on 127.0.0.1, or any RFC 1918\nrange) would have that redirect followed with no re-validation of the\nnew target.\n\nFix: new src/safe_http.rs disables ureq's automatic redirect-following\nentirely (AgentBuilder::redirects(0), confirmed from ureq 2.12.1's\nsource: with redirects(0) a 3xx response is returned to the caller\ninstead of being followed) and instead follows redirects manually,\nre-running validate_url against every Location header target before\nit is ever requested, bounded to 5 hops (matching ureq's own default\nredirect cap, so legitimate chains behave unchanged). Both call sites\nthat validate-then-fetch over HTTP (ping.rs, webhook.rs) now go\nthrough this shared module; grepped all of src/ to confirm there is\nno third call site.\n\nProof: tests/redirect_ssrf_tests.rs spins up real local HTTP servers\n(same hand-rolled TcpListener style as src/serve.rs) proving a\nredirect to a private IP literal is refused and the forbidden target\nnever receives a connection at all, plus a companion test proving a\nlegitimate localhost-to-localhost redirect chain still succeeds.\nsrc/safe_http.rs also carries fast unit tests on the pure per-hop\nvalidation logic. Tests 300 to 306 (verified by stashing this diff\nand rerunning the full suite on the unmodified tree: 300 passed, then\npopping and rerunning: 306 passed).\n\nCo-Authored-By: Claude Sonnet 5 <noreply@anthropic.com>\n\n* docs(roadmap): record SSRF redirect-hardening fix as PR #14\n\nFollow-up to the fix commit: cites the actual PR number now that gh pr\ncreate has returned it. Will be updated with the merge commit once the\nPR lands, matching this file's existing History and supersession\nconvention for other shipped items.\n\nCo-Authored-By: Claude Sonnet 5 <noreply@anthropic.com>\n\n---------\n\nCo-authored-by: Claude Sonnet 5 <noreply@anthropic.com>",
          "timestamp": "2026-07-21T14:11:12-05:00",
          "tree_id": "54cbd9ea64de0af7140a45c82b4a0a232ee33733",
          "url": "https://github.com/rodmen07/svccat/commit/c9250004c419b0a4d5b1438dcdad1acffc62c386"
        },
        "date": 1784661273149,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 12665,
            "range": "± 63",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 23783,
            "range": "± 102",
            "unit": "ns/iter"
          },
          {
            "name": "validate_public_url",
            "value": 316,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "reject_private_ip",
            "value": 5467,
            "range": "± 107",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 5155,
            "range": "± 78",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 12462,
            "range": "± 226",
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
          "id": "1a7d9a3955594a00b652bcc2bdab88a4c68fc805",
          "message": "ci(security): add cargo-audit gate (#17)\n\nsvccat (published to crates.io) had no dependency-vulnerability scanning in CI.\nNew `Security audit` job runs `cargo audit --deny warnings` against the\ncommitted Cargo.lock (the exact pinned versions that ship) on every PR, via\ntaiki-e/install-action@cargo-audit (already this org's install action in\nslokit/axum-api-kit); contents:read least-privilege.\n\nClean at time of adding (213 deps, zero findings), so no lockfile change is\nneeded — the gate exists so a future advisory fails CI instead of riding\nalong invisibly. Third and final PR of the cross-crate audit-gate sweep\n(slokit, axum-api-kit already landed).\n\nGate proven live (L-001): `cargo audit --deny warnings` on the real lockfile\nexits 0; on a copy with idna hand-forced to a vulnerable 0.5.0 it exits 1\nflagging RUSTSEC-2024-0421.\n\nCo-authored-by: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-07-21T20:01:47-05:00",
          "tree_id": "f2a82f856992e07fba25644c85390edeaffe4fb2",
          "url": "https://github.com/rodmen07/svccat/commit/1a7d9a3955594a00b652bcc2bdab88a4c68fc805"
        },
        "date": 1784682299289,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 12789,
            "range": "± 230",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 24457,
            "range": "± 443",
            "unit": "ns/iter"
          },
          {
            "name": "validate_public_url",
            "value": 322,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "reject_private_ip",
            "value": 5478,
            "range": "± 76",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 5163,
            "range": "± 60",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 12196,
            "range": "± 157",
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
          "id": "6a7accca2dbdfcf544d2ff4c0a612be4f785331f",
          "message": "docs(roadmap): reconcile ROADMAP with real state (v1.5.0 shipped, fuzzing real) (#18)\n\nProduct truth-audit (role=Product). Preflight C2 flagged svccat's ROADMAP as\ncarrying stale BLOCKED markers premised on v1.5.0 being unreleased; every claim\nbelow was re-verified against real git/gh/crates.io state before editing.\n\nCorrections:\n- Current state: latest published is v1.5.0 (was: v1.4.1). crates.io\n  newest_version = 1.5.0 (verified via API); tag v1.5.0 = merge commit 60c56b2.\n  Removed the \"in flight, unreleased feat/spdx-sbom\" paragraph (that branch\n  merged as PR #3 and was deleted). Added the features accumulated unreleased on\n  main since v1.5.0 (PRs #6/#11/#12/#14/#15/#16/#17).\n- Fuzzing is no longer a stub (was the 2026-07-18 caveat): PR #15 created\n  fuzz/Cargo.toml, made the 3 targets build, and made the run step real\n  (Continuous Fuzzing). fuzz_policy target + corpora remain (v1.6.0 PR2).\n- Lifted the file-edit gate on CHANGELOG.md/Cargo.toml/Cargo.lock (premised on\n  the unpushed v1.5.0 release-prep commit, which is now published).\n- Unblocked v1.5.1, v1.7.0, v1.8.0 (all cited the same resolved v1.5.0 premise).\n- Reconciled the release-authority line to the standing Merges-and-releases\n  delegation; secret/token writes stay USER-ONLY.\n- History: recorded PR #14 (SSRF) merge commit c925000; updated the\n  fuzz_manifest circular-base line (shipped PR #15, crash fixed PR #16).\n\nDocs-only, single file.\n\nCo-authored-by: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-07-24T01:59:05-05:00",
          "tree_id": "828e0453e24535d40a2674a7b9402a570f5e63a7",
          "url": "https://github.com/rodmen07/svccat/commit/6a7accca2dbdfcf544d2ff4c0a612be4f785331f"
        },
        "date": 1784876531658,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 12602,
            "range": "± 317",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 24120,
            "range": "± 137",
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
            "value": 5501,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 5118,
            "range": "± 70",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 12909,
            "range": "± 87",
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
          "id": "f1a3e084870b32d4a895f8786b9019e442890e1a",
          "message": "test(fuzz): committed seed/regression corpus + CI-gated replay for the 3 fuzz targets (#19)\n\nThe Continuous Fuzzing workflow (fuzzing.yml) only runs on push/schedule/\ndispatch, never on pull_request, so nothing on a PR proves that inputs a past\nfuzz run already found interesting -- crash reproducers in particular -- stay\nhandled gracefully by the current code between the daily runs.\n\nCommit a seed corpus under fuzz/corpus_seeds/<target>/ for fuzz_manifest,\nfuzz_url and fuzz_glob (valid inputs, adversarial inputs, and the PR #16\nbase-cycle crash reproducers), and add tests/fuzz_corpus_replay.rs which\nreplays every seed through the exact library entry point its target calls:\nasserts no process abort, pins the base-cycle seeds to a graceful Err (the\nregression fixed in PR #16), and pins private/loopback URL rejection. These\nfiles double as libFuzzer seed inputs for a `cargo fuzz run`.\n\nAdvances svccat v1.6.0 PR2 (seed-corpora slice). glob + serde_yaml_ng added to\ndev-dependencies (already normal deps, pinned identically) so the replay\nhelpers mirror the fuzz targets verbatim.\n\nCo-authored-by: Claude Opus 4.8 <noreply@anthropic.com>",
          "timestamp": "2026-07-25T03:07:05-05:00",
          "tree_id": "076377da0bc853fbb21acb9d6926f7be82c8c77c",
          "url": "https://github.com/rodmen07/svccat/commit/f1a3e084870b32d4a895f8786b9019e442890e1a"
        },
        "date": 1784967000649,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 9594,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 18206,
            "range": "± 179",
            "unit": "ns/iter"
          },
          {
            "name": "validate_public_url",
            "value": 201,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "reject_private_ip",
            "value": 4015,
            "range": "± 116",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 3810,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 9324,
            "range": "± 16",
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
          "id": "37cdcb3616d8acc8ca3005c0ab93d5eb2e642159",
          "message": "feat(fuzz): fuzz_policy target for the file-based .svccat/policy.yaml config (#20)\n\nThe file-based policy config (`src/policy.rs`, loaded from\n`.svccat/policy.yaml` by `svccat policy`, `svccat ci` and `svccat scorecard`)\nhad no fuzz coverage at all. `fuzz_manifest` only drives the INLINE\n`manifest.policy.rules` list, a different type that happens to share the name\n`PolicyConfig`.\n\n- `fuzz/fuzz_targets/fuzz_policy.rs` parses arbitrary bytes as the file-based\n  `PolicyConfig`, then runs `policy::check` against a fixed catalog so\n  arbitrary field names flow through `has_field` and arbitrary strings flow\n  through the violation `format!`s. It fuzzes the pipeline `PolicyConfig::load`\n  delegates to rather than `load` itself, which takes a `&Path` and would fuzz\n  the filesystem instead of the parser.\n- `fuzz/corpus_seeds/fuzz_policy/` (11 seeds): valid, lenient (unknown keys,\n  duplicates, YAML aliases) and rejected (malformed, wrong type, null, empty).\n- `tests/fuzz_corpus_replay.rs` gains the `drive_policy` mirror plus 5 tests,\n  including exact violation counts that pin `has_field`'s semantics, and\n  `fuzz_targets_agree_across_sources`: a drift guard reading all four sources\n  that must agree on the target set (`fuzz/fuzz_targets/*.rs`,\n  `fuzz/Cargo.toml` `[[bin]]` entries, the Continuous Fuzzing matrix, and\n  `fuzz/corpus_seeds/*`). A target missing from the workflow matrix builds,\n  tests and ships while never being fuzzed by anything, which is the exact\n  inert-surface failure this workflow itself had before PR #15.\n- `docs/FUZZING.md`: target 4 documented, coverage checklist corrected (the\n  old \"dedicated fuzz_policy target\" line conflated the file-based config with\n  structured `arbitrary`-derived generation for the inline rules; the latter\n  stays open and is now named for what it is).\n\nTests 318 -> 323. fmt/clippy/test --all-features green.",
          "timestamp": "2026-07-25T18:28:15-05:00",
          "tree_id": "7de9af17e7bb0c9e9422564c9d0bb5f3d1288d09",
          "url": "https://github.com/rodmen07/svccat/commit/37cdcb3616d8acc8ca3005c0ab93d5eb2e642159"
        },
        "date": 1785022284640,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 12726,
            "range": "± 89",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 23501,
            "range": "± 118",
            "unit": "ns/iter"
          },
          {
            "name": "validate_public_url",
            "value": 316,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "reject_private_ip",
            "value": 5533,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 5223,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 12863,
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "d6225554fc5f2868ee85b5297d51835aa900aa04",
          "message": "chore(deps): notify 6.1.1 -> 8.2.0, with the watch backend's first real coverage (#21)\n\n* chore(deps): notify 6.1.1 -> 8.2.0 with first real coverage for the watch backend\n\nv1.7.0 PR 1 (dependency currency, part 1). notify 8.2.0 is the current stable\nmajor; 9.0.0 is release-candidate only and raises the MSRV to 1.88, so 8 is the\nright target under the declared rust-version = \"1.85\".\n\nZero call-site migration was required: the `Config` / `RecommendedWatcher` /\n`RecursiveMode` / `EventKind` surface that `src/watch.rs` and `src/ci.rs` use is\nunchanged across notify 6, 7 and 8, and the crate built with no source edits at\nall. That is exactly why this PR does not stop there -- \"it compiled\" is the\nonly evidence a filesystem-watching backend swap normally leaves behind, and\n`src/watch.rs` had no tests whatsoever.\n\nSo the substance here is the coverage:\n\n- `is_relevant_accepts_create_modify_remove_and_rejects_everything_else` pins\n  the one place svccat interprets notify's event vocabulary, in both\n  directions. The negative half matters specifically because notify 7.0 started\n  reporting inotify open/access events on Linux; svccat must keep ignoring them\n  rather than re-running the whole drift analysis on every file read.\n- `recommended_watcher_delivers_a_relevant_event_for_a_real_write` wires a real\n  `RecommendedWatcher` exactly the way `run()` does and asserts a relevant event\n  actually arrives for a real file write. The three `Build & Test (This\n  Checkout)` legs run it on inotify, ReadDirectoryChangesW and FSEvents -- three\n  different implementations behind one type alias.\n\nSupply chain: 213 -> 212 crates. `crossbeam-channel`, `filetime` and\n`bitflags` 1.x drop out (notify 7 removed its internal crossbeam use and\ndisabled that feature by default); `notify-types` is added (notify 7 moved the\nevent types into it); the windows-sys family moves 0.48 -> 0.60.\n`cargo audit --deny warnings` clean at 212 deps.\n\n* docs(roadmap): cite PR #21 on the shipped v1.7.0 PR 1 line",
          "timestamp": "2026-07-25T19:15:53-05:00",
          "tree_id": "4e4a0a0df89bc2f581d5354221c0e58bba9af092",
          "url": "https://github.com/rodmen07/svccat/commit/d6225554fc5f2868ee85b5297d51835aa900aa04"
        },
        "date": 1785025137151,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 12725,
            "range": "± 376",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 24251,
            "range": "± 217",
            "unit": "ns/iter"
          },
          {
            "name": "validate_public_url",
            "value": 299,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "reject_private_ip",
            "value": 5057,
            "range": "± 69",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 4836,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 12366,
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "ff7e9aefbe436b7b057cd116b453a72853a76536",
          "message": "ci(fuzz): seed the Continuous Fuzzing campaign from the committed corpora (#22)\n\n* ci(fuzz): seed the Continuous Fuzzing campaign from the committed corpora\n\nPR #19 committed 37 seed/regression inputs under fuzz/corpus_seeds/<target>/\nand PR #20 added an 11-seed fourth target, but nothing ever handed them to\nlibFuzzer: the workflow ran `cargo fuzz run <target> -- -max_total_time=120`\nwith no corpus argument, so every job started from an empty corpus. The last\nrun on main (30180866540) shows it plainly -- all four targets report\n`INITED ... corp: 1/1b`, one synthetic empty input.\n\nPass the corpora explicitly, ordered. libFuzzer writes newly discovered\ninputs into the FIRST corpus directory and treats later ones as read-only\nseeds, so the gitignored working corpus fuzz/corpus/<target> goes first and\nthe committed fuzz/corpus_seeds/<target> second: a CI run now starts from the\ncommitted seeds without ever mutating them.\n\nfuzzing.yml does not run on pull_request, so this wiring needs a PR-time\nguard of its own or it can be silently undone the same way it was silently\nmissing. fuzzing_workflow_runs_from_the_committed_seed_corpus reads the real\nrun step out of the workflow and asserts the seeds are passed, that the\nwritable corpus precedes them, and that they land before the `--` separator\n(after it they would be a libFuzzer flag, not a corpus directory).\n\ndocs/FUZZING.md's CI Integration paragraph quoted the old command; updated to\nthe real one with the ordering rationale, so the doc and the workflow cannot\ndisagree.\n\n* test(fuzz): guard docs/FUZZING.md against quoting a command CI does not run\n\nThe doc quoted the pre-seed-corpus one-liner for exactly as long as the\ncommitted seeds went unused, so a reader following it would have reproduced\nthe empty-corpus run and seen nothing wrong. Reconciling it once is not\nenough: docs_quote_the_workflows_actual_fuzz_command reads the fenced command\nout of the CI Integration section AND the real run step out of fuzzing.yml,\nnormalises ${{ matrix.target }} to <target>, and asserts they are the same\ncommand.",
          "timestamp": "2026-07-25T19:57:29-05:00",
          "tree_id": "5f709ad01bed48edada7351213ec6dc769a2c519",
          "url": "https://github.com/rodmen07/svccat/commit/ff7e9aefbe436b7b057cd116b453a72853a76536"
        },
        "date": 1785027642568,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 12521,
            "range": "± 419",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 23866,
            "range": "± 137",
            "unit": "ns/iter"
          },
          {
            "name": "validate_public_url",
            "value": 311,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "reject_private_ip",
            "value": 5488,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 5115,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 12508,
            "range": "± 298",
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
          "id": "e08852b440724129026488d4f8fba63a260ec6e7",
          "message": "docs(fuzzing): rewrite FUZZING.md and pin it to the real fuzz layout (v1.6.0 PR2d) (#23)\n\n* docs(fuzzing): rewrite FUZZING.md and pin it to the real fuzz layout\n\nThe `Best Practices` section told readers to put interesting inputs in\n`fuzz/seeds/`. That directory has never existed in this repo -- the seed\ncorpus is `fuzz/corpus_seeds/<target>/` -- so anyone following the advice\nadded files that cargo-fuzz, the workflow and the replay suite all ignore.\nThe document also had no account of the committed regression corpus as a\n*workflow*: what to actually do when a run finds a crash.\n\nRewrite:\n\n- crash workflow documented end to end (get the input, reproduce, fix,\n  commit the seed under its real path, pin the contract by name), plus a\n  section on the committed corpus itself (the `drive_*` mirror rule and\n  the 0-byte `empty` seed that makes `seed corpus: files: N` read one low)\n- corpus/artifact arguments moved before the `--` separator, matching\n  `cargo fuzz run [OPTIONS] <TARGET> [CORPUS]... [-- <ARGS>...]`; the\n  run-all loop now passes the same corpora, in the same order, as CI\n- coverage section corrected: `cargo fuzz coverage` has no report-format\n  flag, and args after `--` go to libFuzzer rather than to a renderer\n- new subsection on verifying a fuzzing change by `workflow_dispatch` from\n  a branch, which is how this repo verifies such changes on a dev box with\n  no clang\n\nTwo new PR-time guards in tests/fuzz_corpus_replay.rs, because a wrong path\nin a doc is the quietest kind of inert surface:\n\n- `docs_only_reference_fuzz_paths_that_exist`: every `fuzz/...` path the\n  prose names must be committed under `fuzz/` or listed in `fuzz/.gitignore`\n  as generated. Reads the ignore file rather than hardcoding a list.\n- `docs_describe_every_fuzz_target`: the `## Fuzz Targets` section must cite\n  `fuzz/fuzz_targets/<name>.rs` for exactly the targets that exist, making\n  the prose a fifth source in the drift set `fuzz_targets_agree_across_sources`\n  already pins.\n\nThe `## CI Integration` fenced command is untouched, so\n`docs_quote_the_workflows_actual_fuzz_command` still holds it equal to the\nworkflow run step.\n\nTests 327 -> 329.\n\n* fix(test): make the docs path guard platform-independent\n\n`Build & Test (This Checkout)` failed on ubuntu and macos while passing on\nwindows. The cause was in the new guard, not in the doc: the extractor\ntreated the prose ellipsis in \"every `fuzz/...` path named below\" as a path\nsegment, and the guard then probed it with `Path::exists()`. Windows\nnormalizes trailing dots away, so `fuzz/...` resolved and reported true\nthere; Linux and macOS answered false and failed the assertion.\n\nTwo changes, either of which alone would have caught it:\n\n- compare against a `read_dir` listing of `fuzz/` instead of an\n  `exists()` probe, so no platform path normalizer is in the loop\n- drop segments that carry no alphanumeric character (and trailing\n  sentence dots), because those are prose, not paths\n\nVerified by reproducing the CI failure locally on Windows: with the\nread_dir change in place and only the prose filter removed, the Windows\nrun fails with the same `fuzz/...` message the ubuntu leg produced. Full\ngate green: fmt, clippy -D warnings, 329 tests.",
          "timestamp": "2026-07-25T20:37:56-05:00",
          "tree_id": "2c3b19b74d3e21876f69a3b0e6f0b43e5c9a112d",
          "url": "https://github.com/rodmen07/svccat/commit/e08852b440724129026488d4f8fba63a260ec6e7"
        },
        "date": 1785030065674,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 12490,
            "range": "± 129",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 23767,
            "range": "± 90",
            "unit": "ns/iter"
          },
          {
            "name": "validate_public_url",
            "value": 305,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "reject_private_ip",
            "value": 5614,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 5238,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 12183,
            "range": "± 63",
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
          "id": "805d7e0f41986403da03b84c208f97d938b07619",
          "message": "test(watch): real coverage for change detection, fixing the two fields it ignored (#24)\n\n`tests/watch_enhancement_tests.rs` was named for the watch surface but\nnever touched it. All five of its tests re-implemented the added/removed\nset computation inline (\"Detect changes manually (mimicking the\ndetect_changes logic)\"), so they passed no matter what `src/watch.rs`\ndid: gut `detect_changes` to return three empty vectors and all five\nstill pass.\n\nPointing the tests at the real function immediately surfaced a live\ndefect. `services_equal` hand-listed 11 of `ServiceEntry`'s 13 fields\nand omitted `path` and `submodule` -- the two that decide where a\nservice lives on disk (`ServiceEntry::declared_path`) -- so re-pointing\na service in `services.yaml` was never reported as a modification by\n`svccat watch`, and printed nothing at all when the re-point did not\nalso change the drift count.\n\n- `services_equal` now delegates to the derived `PartialEq`, so it reads\n  the struct definition instead of a copy of its field list.\n- Five unit tests in `src/watch.rs` that call `detect_changes`: a\n  per-field mutation sweep, rename as add-plus-remove, add + remove +\n  modify together, no-op including reordering, and both empty-list\n  directions.\n- `populated_service` is an exhaustive struct literal with no\n  `..Default::default()`, so a 14th `ServiceEntry` field stops this file\n  compiling until it is given a value and a mutator.\n- `tests/watch_enhancement_tests.rs` deleted; its five cases are covered\n  by tests that can actually fail.\n\nCo-authored-by: Claude Opus 5 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-07-25T21:12:19-05:00",
          "tree_id": "29cdf3bc32d54fe796e5c2f08b26c8c1924c17e5",
          "url": "https://github.com/rodmen07/svccat/commit/805d7e0f41986403da03b84c208f97d938b07619"
        },
        "date": 1785032123590,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 12459,
            "range": "± 178",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 23945,
            "range": "± 131",
            "unit": "ns/iter"
          },
          {
            "name": "validate_public_url",
            "value": 265,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "reject_private_ip",
            "value": 5030,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 4752,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 12104,
            "range": "± 36",
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
          "id": "bb68350abbc496e0f84e5611c6c54660240cfdd2",
          "message": "fix(policy): report a broken policy file instead of pretending it is absent (#25)\n\n`PolicyConfig::load` swallowed both the read error and the parse error and\nreturned `None`, so a `.svccat/policy.yaml` with a typo in it was\nindistinguishable from having no policy file at all. All three call sites\nreported that silence differently, and all three were wrong: `svccat policy`\nprinted \"No policy file found. Create .svccat/policy.yaml ...\" about a file\nthat exists and exited 0; `svccat ci` dropped the `policy` step and reported\n\"all checks passed\", silently disabling the policy gate in a pipeline; and\n`svccat scorecard` scored the repo with no policy contribution, in silence.\n\nAdds `PolicyConfig::load_checked` -> `Result<Option<Self>, PolicyLoadError>`\n(additive) and makes `load` delegate to it, dropping the error, so the two\ncan never disagree about which candidate file wins. Candidate order and the\n\"first one that loads wins\" rule are unchanged, which is what lets every call\nsite migrate without changing which configuration is in force.\n\nCo-authored-by: Claude Opus 5 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-07-25T22:34:22-05:00",
          "tree_id": "d3993dd0d676ede2a14cfea8f77bc447ba12b99f",
          "url": "https://github.com/rodmen07/svccat/commit/bb68350abbc496e0f84e5611c6c54660240cfdd2"
        },
        "date": 1785037052010,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 12504,
            "range": "± 342",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 23464,
            "range": "± 141",
            "unit": "ns/iter"
          },
          {
            "name": "validate_public_url",
            "value": 263,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "reject_private_ip",
            "value": 5158,
            "range": "± 87",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 4825,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 12386,
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "5767793f227bb062ba1dcc342ce8cceca6279a96",
          "message": "ci: add the missing lint gate (fmt + clippy) (#26)\n\n* ci: add the missing lint gate (fmt + clippy) to ci.yml\n\nsvccat was the only one of the three published crates with no lint gate at\nall: none of the six workflow files ran `cargo fmt` or `cargo clippy`, and\nthere is no rust-toolchain, clippy.toml, deny.toml, Makefile, justfile or\npre-commit config either, so formatting drift and clippy `-D warnings`\nerrors could merge silently. PR #24's first draft carried two\n(`field_reassign_with_default`, `type_complexity`) that only a local run\ncaught. slokit already requires a `fmt, clippy, test` context.\n\nNew `lint` job, \"Lint (fmt + clippy)\", ubuntu-latest only (the three-OS\nBuild & Test matrix already covers compile/test; rustfmt output and the\nclippy lint set are toolchain-versioned, not OS-versioned).\n\n`tests/ci_lint_gate_tests.rs` pins the job from the test suite, which every\nPR runs: one test asserts the job's `name:` (the branch-protection required\ncontext string) and one asserts both commands verbatim, so deleting the\njob, renaming it out from under its protection binding, or dropping\n`-D warnings` fails a check instead of passing silently.\n\nCo-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>\n\n* fix(rule_schema): clear the clippy question_mark the new gate caught\n\nThe `Lint (fmt + clippy)` job went red on its own first run: clippy 1.97 on\nthe ubuntu runner flagged `clippy::question_mark` at src/rule_schema.rs:179\n(`find_base_cycle`'s `match by_id.get(base_id)` with a `None => return None`\narm) while the local clippy 1.96 was silent about it. Pre-existing code,\nfound by the gate rather than by a diff, which is the point of adding it.\n\nApplied clippy's own suggestion (`let base_rule = by_id.get(base_id)?;`);\nsemantics are identical, a dangling base still terminates the walk as \"not a\ncycle\". `cargo test --all-features` 343 passed, including the 9 rule_schema\nunit tests that cover self-reference, mutual and three-hop cycles plus the\ndangling-base case. Verified clean under stable clippy 1.96 AND nightly\nclippy 0.1.99 (a superset of the runner's 1.97 lint set).\n\nAlso documents the floating-@stable toolchain-drift caveat in the job's\ncomment block: the response to a new lint is to clear it, never to weaken\nthe gate.\n\nCo-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>\n\n---------\n\nCo-authored-by: Claude Opus 5 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-07-25T23:26:45-05:00",
          "tree_id": "546742405fe5024af8bb9e2c1c5d281f868495e2",
          "url": "https://github.com/rodmen07/svccat/commit/5767793f227bb062ba1dcc342ce8cceca6279a96"
        },
        "date": 1785040208426,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 12631,
            "range": "± 108",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 23280,
            "range": "± 168",
            "unit": "ns/iter"
          },
          {
            "name": "validate_public_url",
            "value": 307,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "reject_private_ip",
            "value": 5437,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 5181,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 12052,
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
          "id": "ce830381658a4e128eea2f59dd5f8640589d6036",
          "message": "fix(watch): report manifest changes in manifest order, not hash order (#27)\n\ndetect_changes built its added/removed lists from HashSet::difference,\nwhose iteration order is unspecified and is re-randomised per set by the\ndefault hasher, so watch mode printed the same change set in a different\norder on every reload. modified was never affected: it walks new_services,\ni.e. manifest order. All three lists now follow the manifest they were read\nfrom, via one names_in_manifest_order helper that also deduplicates a name\nthe loader accepts twice.\n\nThree new tests (343 -> 346) and the existing combined test no longer sorts\nits result before asserting.\n\nCo-authored-by: Claude Opus 5 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-07-26T00:02:28-05:00",
          "tree_id": "929b5c4f0fecadda8c21857b359fffb4e76fda8f",
          "url": "https://github.com/rodmen07/svccat/commit/ce830381658a4e128eea2f59dd5f8640589d6036"
        },
        "date": 1785042336890,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 12505,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 23813,
            "range": "± 71",
            "unit": "ns/iter"
          },
          {
            "name": "validate_public_url",
            "value": 309,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "reject_private_ip",
            "value": 5497,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 5145,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 12276,
            "range": "± 36",
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
          "id": "eb9b9c1adbe2990acd2f68aaa6f40754b7e8dcd7",
          "message": "fix(sarif): stop dropping --ping results, and give the renderer its first tests (#28)\n\n`src/output/sarif.rs` had never had a test of any shape: no `mod tests`, and\nno file under `tests/` referenced it. Writing its first coverage surfaced a\nlive defect on the published CLI.\n\n`render_check(report, _ping_results)` took the ping results and discarded\nthem, so `svccat check --ping --format sarif` reported nothing at all about\nan unreachable or SSRF-blocked service URL. It is invisible twice over,\nbecause ping never affects `check`'s exit code either. Every sibling renderer\n(json, junit, markdown, terminal) reports them.\n\nPing failures are now results under two new rules, `unreachable_service` and\n`invalid_service_url`. A *reachable* service emits nothing: SARIF results are\nfindings, not a per-URL health log, which mirrors junit where a reachable\nservice is a passing testcase and only the other two states are failures.\nBoth result emitters now go through one `sarif_finding` helper so a new\nfinding kind cannot invent a different location layout.\n\nTests 346 -> 361: 11 unit tests in `src/output/sarif.rs` and 4 binary-level\ntests in the new `tests/sarif_output_tests.rs` that spawn the real binary.\n\nCo-authored-by: Claude Opus 5 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-07-26T01:02:08-05:00",
          "tree_id": "9c820f7742df9059ec1907eeca46b25862b0500d",
          "url": "https://github.com/rodmen07/svccat/commit/eb9b9c1adbe2990acd2f68aaa6f40754b7e8dcd7"
        },
        "date": 1785045920358,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 12789,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 24119,
            "range": "± 152",
            "unit": "ns/iter"
          },
          {
            "name": "validate_public_url",
            "value": 292,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "reject_private_ip",
            "value": 5546,
            "range": "± 65",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 5272,
            "range": "± 255",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 12456,
            "range": "± 89",
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
          "id": "010e0654289ffcfd2c1e4b0a9c9f94a63fa74c4d",
          "message": "docs(roadmap): reconcile ROADMAP with real state, and guard it with a test (#29)\n\nThe 2026-07-24 reconciliation (PR #18) was stale within two days. Facts\nchecked against git, the crates.io API and the repo itself:\n\n- `### v1.6.0: make fuzzing real` still read \"PR 1 SHIPPED, PR 2 remaining\"\n  and listed both remaining bullets as open. Both shipped: `fuzz_policy`\n  in PR #20, committed seed corpora in PR #19 (4 target dirs, 7/9/11/10\n  seeds), wired into the campaign in PR #22, docs pinned in PR #23. Moved\n  to History with the evidence.\n- `## Current state`'s unreleased list stopped at PR #17 while eleven more\n  PRs had merged, and it never named PR #4 or #5 at all. Replaced by a\n  `## Unreleased on main` section enumerating all 24 commits since the\n  `v1.5.0` tag, grouped, each with its PR and squash commit.\n- `### v1.5.0` sat under `## Milestones` a week after it shipped.\n- v1.7.0/v1.8.0 named no target versions. Now pinned to the crates.io\n  `max_stable_version` read today: criterion 0.8.2, ureq 3.3.0,\n  colored 3.1.1, notify 8.2.0 (already current).\n\nFound while reconciling, and the reason this is a milestone rather than a\ntidy-up: **CHANGELOG.md's `[Unreleased]` is nine user-facing changes\nshort.** It records PRs #21/#24/#25/#27 only; the CycloneDX exporter\n(#11), three multi-repo features (#4/#5/#6), policy rule schema\nvalidation (#12), the DOM-XSS fix (#7), the SSRF redirect fix (#14), the\nHIGH cyclic-base stack overflow fix (#16) and the SARIF ping fix (#28)\nare all absent. Verified by grepping the whole file per change, not by\nreading the top of it. So `### v1.6.0` is redefined as the release cut\nthat publishes what is already on main, with a checkable done-when and a\nsemver classification (MINOR) flagged as an overridable default.\n\n`### v1.5.1` is retired and folded into it: its two live tasks are\ncarried verbatim, and cutting a 1.5.1 patch would publish a version\nnumber strictly older than nine unreleased user-facing changes.\n\nNew `tests/roadmap_truth.rs` (5 tests) makes this the last hand-run\nreconciliation. It reads ROADMAP.md against Cargo.toml and CHANGELOG.md\nand fails the build when they disagree: the crate-version bullet vs\n`[package] version`; no released version still listed as an upcoming\n`### vX.Y.Z` milestone; no released version in a BLOCKED row; and\n`## Unreleased on main` present exactly when the changelog has\n`[Unreleased]` entries. Every extractor is exercised on synthetic input\nso a parser that stops matching fails loudly instead of passing\neverything.\n\nTests 361 -> 366. No src/ change; no API or behaviour change.\n\nCo-authored-by: Claude Opus 5 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-07-26T01:41:32-05:00",
          "tree_id": "13e0e9da80eb5b2400d5400510fbec9d02014b25",
          "url": "https://github.com/rodmen07/svccat/commit/010e0654289ffcfd2c1e4b0a9c9f94a63fa74c4d"
        },
        "date": 1785048287578,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 12670,
            "range": "± 182",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 24075,
            "range": "± 205",
            "unit": "ns/iter"
          },
          {
            "name": "validate_public_url",
            "value": 298,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "reject_private_ip",
            "value": 5439,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 5154,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 12201,
            "range": "± 37",
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
          "id": "fe59cd59bb35894a122ac531f6ba972d141e03d5",
          "message": "release: prepare v1.6.0, the nine changes main never wrote down (#30)\n\n24 commits have sat on main since the v1.5.0 tag and `cargo install svccat`\nstill delivers 1.5.0, so three security fixes reach nobody: the DOM-based XSS\nin the HTML graph (PR #7), the SSRF-via-redirect in --ping and webhooks\n(PR #14), and the HIGH stack overflow on a cyclic policy `base` chain (PR #16).\n\nCHANGELOG.md's [Unreleased] section recorded four of the thirteen user-facing\nchanges. The other nine were absent, re-checked here by grepping the whole file\nper change rather than reading its top: CycloneDX 0 hits, \"rule schema\" 0,\nXSS 0, redirect 0, \"repo filter\" 0, \"[reporting]\" 0, and \"workspace check\",\nSSRF, \"stack overflow\" and sarif hitting only pre-1.5.0 sections.\n\n  - [Unreleased] becomes [1.6.0] - 2026-07-26 with all thirteen entries under\n    Added / Changed / Fixed / Security, each citing its PR.\n  - CHANGELOG.md is reordered into strictly descending version order. It was\n    not: 1.2.0 sat between 1.4.0 and 1.3.0, and 1.3.2 above 1.3.1, since\n    2026-07-09. That reorder was a hand task carried through two hygiene\n    milestones (v1.5.1, retired; then v1.6.0), so this third reconciliation\n    ships as a guard instead of a promise:\n    changelog_versions_are_in_strictly_descending_order in tests/roadmap_truth.rs.\n  - SECURITY.md's \"v0.19.0 (Planned)\" boxes had been unchecked since\n    2026-05-28 although v0.19.0 shipped all five that day. Resolved against the\n    0.19.0 changelog entry, with the honest note that the --follow-symlinks\n    opt-out was never built because discovery rejects symlinks unconditionally\n    (src/discovery.rs:140). \"v0.18.1 (Current)\" was seven versions stale. This\n    release's own three security fixes were added to that file.\n  - Cargo.toml and Cargo.lock go to 1.6.0; ROADMAP.md moves the v1.6.0\n    milestone to History and drops `## Unreleased on main`, both of which the\n    existing roadmap guards now require.\n\nNOT done: the v1.6.0 tag. The MINOR-versus-MAJOR classification is an\noverridable default ROADMAP.md routes to the user, and a crates.io publish\ncannot be undone. The full argument is carried into ROADMAP.md's History entry\nso it does not vanish with the milestone, and the pending step is a dated row\nin the blocked/user-only table.\n\nCo-authored-by: Claude Opus 5 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-07-26T02:25:50-05:00",
          "tree_id": "eec636d2bd9e4f56c6da3b2aa39ca9a618ee9ad2",
          "url": "https://github.com/rodmen07/svccat/commit/fe59cd59bb35894a122ac531f6ba972d141e03d5"
        },
        "date": 1785050942179,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 12513,
            "range": "± 66",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 23925,
            "range": "± 91",
            "unit": "ns/iter"
          },
          {
            "name": "validate_public_url",
            "value": 300,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "reject_private_ip",
            "value": 5436,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 5200,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 12174,
            "range": "± 689",
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
          "id": "eda3bda271d5da5341805455f056662cc6035b5e",
          "message": "fix(diff): report snapshot-diff drift in snapshot order, not hash order (#31)\n\n`svccat snapshot diff` and `svccat diff` fill the same public\n`DiffReport::new_drift` / `resolved_drift` fields through two separate\nbuilders, and only one of them was correct.\n\n`build_diff` (the `snapshot diff` path) computed both lists with\n`HashSet::difference`. `RandomState` seeds each set independently, so the\nlists shuffled between two runs over byte-identical snapshots, and in fact\nbetween two calls inside one process. It also emitted the bare\n`service:message` dedup key, while `diff_snapshots` (the `svccat diff` path)\nemitted the severity-prefixed line, so one public field carried two different\nformats depending on which command filled it.\n\nBoth paths now share one `drift_changes` helper that walks the source\nsnapshot's drift vector in order and reports each `service:message` once,\nkeeping its first occurrence. A drift item whose severity was re-classified\nbetween snapshots is still not reported as one resolved plus one new, since\nthe comparison key deliberately excludes severity.\n\n`tests/diff_drift_order_tests.rs` reads BOTH entry points and asserts they\nproduce identical lists, so re-splitting the implementation fails the build\ninstead of drifting quietly the way the two builders already had.\n\nCo-authored-by: Claude Opus 5 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-07-26T04:00:12-05:00",
          "tree_id": "5a7b4ef74779794ece91a468f9a93ba4712951b9",
          "url": "https://github.com/rodmen07/svccat/commit/eda3bda271d5da5341805455f056662cc6035b5e"
        },
        "date": 1785056604679,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 12575,
            "range": "± 104",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 24110,
            "range": "± 243",
            "unit": "ns/iter"
          },
          {
            "name": "validate_public_url",
            "value": 296,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "reject_private_ip",
            "value": 5568,
            "range": "± 46",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 5214,
            "range": "± 54",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 12169,
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "175f6903e7f698a47242237b7261c1282b57e900",
          "message": "docs(roadmap): record the v1.6.0 cut, and make the guard see HELD rows (#32)\n\nv1.6.0 is published (tag `v1.6.0` at `fe59cd5`, publish run 30196265195 green,\ncrates.io `\"newest_version\":\"1.6.0\"`), so every claim in ROADMAP.md that said\notherwise is now false and is corrected here.\n\nThe interesting half is why the drift guard stayed green while the roadmap was\nwrong. `blocked_row_versions` required the status cell to equal `BLOCKED`\nexactly, so it never saw\n\n  | The **v1.6.0** tag push specifically | HELD ON THE LOCAL HARNESS (2026-07-26) | ... |\n\neven though CHANGELOG.md had said `## [1.6.0] - 2026-07-26` since PR #30. That\nis exactly the disagreement guard 3 exists to shout about. And it is structural\nrather than a one-off: the marker convention requires a status cell to carry its\ndate and clearing condition inline, so a real gating row can never equal a bare\nkeyword. The extractor now matches the status cell's FIRST WORD against\n`BLOCKED` or `HELD`; `USER-ONLY`, `Delegated` and `Avoid` rows stay ignored,\nsince those are standing policy about an action rather than a gate on a version.",
          "timestamp": "2026-07-26T08:00:00-05:00",
          "tree_id": "09f069f0e093a16c22c3acc96b44519d22f51e26",
          "url": "https://github.com/rodmen07/svccat/commit/175f6903e7f698a47242237b7261c1282b57e900"
        },
        "date": 1785070987575,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 12452,
            "range": "± 55",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 23288,
            "range": "± 66",
            "unit": "ns/iter"
          },
          {
            "name": "validate_public_url",
            "value": 299,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "reject_private_ip",
            "value": 5408,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 5140,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 12019,
            "range": "± 61",
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
          "id": "203281930aa8afec2ab559217cfff1a925874cab",
          "message": "fix(policy): load .svccat/policy.yaml under the manifest's resource-limit posture (#33)\n\n* wip: interrupted policy size-limits increment\n\nAutodev wave 17 was killed mid-increment when the Claude Code process\nexited (2026-07-26 10:20). Preserves the working tree as the worker left\nit. NOT verified, NOT gate-checked, NOT PR-ready: audit before trusting\n(L-007), then finish or discard this branch on its merits.\n\nCo-Authored-By: Claude Fable 5 <noreply@anthropic.com>\n\n* style: rustfmt pass over the interrupted WIP\n\nCo-Authored-By: Claude Fable 5 <noreply@anthropic.com>\n\n* docs: changelog and roadmap entries for the policy resource limits\n\nCo-Authored-By: Claude Fable 5 <noreply@anthropic.com>\n\n---------\n\nCo-authored-by: Claude Fable 5 <noreply@anthropic.com>",
          "timestamp": "2026-07-26T12:33:10-05:00",
          "tree_id": "5aa6c08c2525e2ed4b254d501b9647524d057f3a",
          "url": "https://github.com/rodmen07/svccat/commit/203281930aa8afec2ab559217cfff1a925874cab"
        },
        "date": 1785087348563,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 9729,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 18250,
            "range": "± 62",
            "unit": "ns/iter"
          },
          {
            "name": "validate_public_url",
            "value": 207,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "reject_private_ip",
            "value": 3931,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 3565,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 9529,
            "range": "± 19",
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
          "id": "c847d15096879a27e8970e3f490b20ef5f5e3c53",
          "message": "fix(sarif): emit legal URIs for absolute manifest paths in artifactLocation.uri (#34)\n\nSARIF 2.1.0 types artifactLocation.uri as a URI reference, so the old\npass-through of absolute paths shipped C:/repo/services.yaml as a URI\nwith scheme 'c' and /srv/repo/services.yaml as a root-relative\nreference. Absolute paths are now relativised against the run root when\nunder it, and become percent-encoded file:// URIs otherwise; relative\npaths are byte-identical to before. The artifact URI is computed once\nin build_sarif and threaded into every result.\n\nCo-authored-by: Claude Fable 5 <noreply@anthropic.com>",
          "timestamp": "2026-07-26T13:10:10-05:00",
          "tree_id": "82e16c46c58a91969a3b54a4449b45f1bd3bca58",
          "url": "https://github.com/rodmen07/svccat/commit/c847d15096879a27e8970e3f490b20ef5f5e3c53"
        },
        "date": 1785089608335,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 10391,
            "range": "± 349",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 21231,
            "range": "± 450",
            "unit": "ns/iter"
          },
          {
            "name": "validate_public_url",
            "value": 241,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "reject_private_ip",
            "value": 4326,
            "range": "± 112",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 4012,
            "range": "± 105",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 10039,
            "range": "± 385",
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
          "id": "529d66dece86ecb06206bf6d134fc3bd5bbb782e",
          "message": "qa(github-annotation): first coverage — wire --ping results in, escape workflow-command output (#35)\n\n* wip: interrupted github-annotation QA increment\n\nAutodev wave 20 was killed mid-increment when the Claude Code process\nexited (2026-07-26 13:28). Preserves the working tree as the worker left\nit. NOT verified, NOT gate-checked, NOT PR-ready: audit before trusting\n(L-007), then finish or discard this branch on its merits.\n\nCo-Authored-By: Claude Fable 5 <noreply@anthropic.com>\n\n* qa(github-annotation): first coverage; wire --ping results in; escape workflow-command output\n\nCompletes the interrupted wave-20 increment (wip commit audited per\nL-007, gap found: render_check's new two-arg signature was never wired\ninto main.rs, so the branch did not compile). Adds the binary-level test\nhalf, the CHANGELOG/ROADMAP entries, and corrects a false comment (\nPingStatus is not #[non_exhaustive]).\n\nCo-Authored-By: Claude Fable 5 <noreply@anthropic.com>\n\n---------\n\nCo-authored-by: Claude Fable 5 <noreply@anthropic.com>",
          "timestamp": "2026-07-26T20:19:36-05:00",
          "tree_id": "04de6b90eae40ed591a5f7276c3312067d30f150",
          "url": "https://github.com/rodmen07/svccat/commit/529d66dece86ecb06206bf6d134fc3bd5bbb782e"
        },
        "date": 1785115369464,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 12418,
            "range": "± 106",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 23674,
            "range": "± 92",
            "unit": "ns/iter"
          },
          {
            "name": "validate_public_url",
            "value": 272,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "reject_private_ip",
            "value": 5080,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 4684,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 12407,
            "range": "± 118",
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
          "id": "03439a3201fc58845501abea7ed26d175e3d2d36",
          "message": "feat(sarif): anchor drift findings to their manifest line via region.startLine (#36)\n\n* feat(sarif): anchor drift findings to their manifest line via region.startLine\n\nThe sarif module doc has promised 'inline PR annotations' since the format\nshipped, but every result carried only an artifactLocation with no region,\nso a SARIF consumer had nothing to anchor an inline annotation to. The\ndrift data genuinely had no line to offer: serde_yaml_ng 0.10 exposes\nposition info only on Error (no span surface for values that parse), so a\nloaded Manifest knows nothing about where its entries sit in the file.\n\nNew #[doc(hidden)] module src/manifest_lines.rs recovers lines positionally\nby a text scan: the Nth name: key inside the top-level services: block\nbelongs to the Nth ServiceEntry, because serde fills the Vec in document\norder. The scan FAILS CLOSED: attach() compares the match count against\nthe parsed service count and attaches nothing on disagreement, so exotic\nYAML (a block scalar body that looks like a name: key, a quoted key)\ndegrades to today's file-level findings, never to a wrong line.\n\nDriftItem gains line: Option<usize> (non_exhaustive struct; additive per\ndocs/API_STABILITY.md, which explicitly allows new fields in a minor;\nserde default keeps old snapshots and baselines loading, and\nskip_serializing_if keeps None out of every JSON document). analyze()'s\ncovered signature is untouched: main.rs attaches lines as a post-pass in\nthe check command, mapping against the FULL manifest regardless of any\n--team filter. The sarif renderer emits region.startLine when the item\ncarries a line; ping findings stay deliberately file-level (a ping failure\nis about a URL answering, not a line in the file), and the module doc now\nsays exactly what anchors where.\n\nTests 403 -> 418: 11 unit tests on the scanner and the fail-closed attach\ncontract, 3 sarif region unit tests, 1 binary-level test asserting the\nreal binary anchors the fixture's drift finding to line 7 while its ping\nfinding stays region-less.\n\nCo-Authored-By: Claude Fable 5 <noreply@anthropic.com>\n\n* docs: record the SARIF region feature in CHANGELOG [Unreleased] and ROADMAP\n\nCo-Authored-By: Claude Fable 5 <noreply@anthropic.com>\n\n---------\n\nCo-authored-by: Claude Fable 5 <noreply@anthropic.com>",
          "timestamp": "2026-07-26T22:14:06-05:00",
          "tree_id": "6d2b5f1127470679b3cc38970d5d93a9d6c42ae3",
          "url": "https://github.com/rodmen07/svccat/commit/03439a3201fc58845501abea7ed26d175e3d2d36"
        },
        "date": 1785122277250,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 12528,
            "range": "± 274",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 23519,
            "range": "± 149",
            "unit": "ns/iter"
          },
          {
            "name": "validate_public_url",
            "value": 309,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "reject_private_ip",
            "value": 5570,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 5139,
            "range": "± 57",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 12037,
            "range": "± 367",
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
          "id": "978d049bb93c8f6511b652a96aeb0919a20271c2",
          "message": "feat(annotations): anchor drift annotations to their manifest line via line= (#37)\n\nThe follow-through on the SARIF region.startLine change for the format\nGitHub Actions runs by default: a drift annotation whose item carries a\nmanifest line (DriftItem::line, attached on the check path by\nmanifest_lines) now emits it as the line= workflow-command property, so\nthe annotation appears inline at the service's name: entry in the PR\nview. Unanchored items render exactly as before (no line= rather than a\nwrong one), and ping findings stay deliberately file-level, mirroring\nthe sarif renderer. The line value is formatted from a usize inside the\none escaping-aware builder, so it cannot smuggle workflow-command syntax\nand no call site can bypass the builder.\n\nCo-authored-by: Claude Fable 5 <noreply@anthropic.com>",
          "timestamp": "2026-07-26T22:57:09-05:00",
          "tree_id": "a48d2b192d152413746027a1ecc1d18964569a44",
          "url": "https://github.com/rodmen07/svccat/commit/978d049bb93c8f6511b652a96aeb0919a20271c2"
        },
        "date": 1785124815453,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 12410,
            "range": "± 61",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 23980,
            "range": "± 236",
            "unit": "ns/iter"
          },
          {
            "name": "validate_public_url",
            "value": 260,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "reject_private_ip",
            "value": 5179,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 4734,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 12483,
            "range": "± 220",
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
          "id": "192fa76f3eabaeb9839e73fc939e30025a6c60f5",
          "message": "qa(coverage): first tests for the terminal renderer, plus the v1.6.1 orphaned-sidecar edge case (#38)\n\nsrc/output/terminal.rs was the oldest untested surface in the repo: its\ncontent dates to v0.1.0 (48ce7a1), was last touched for v0.7.0 (d183889),\nand had no mod tests and no integration test referencing it directly.\n\n- tests/terminal_output_tests.rs (11 tests): 5 in-process tests pinning\n  render_since_diff's return values and the kind|service|detail identity\n  contract (message and severity changes are NOT new drift; details keep\n  same-kind items distinct) — the same contract main.rs's --baseline\n  filter duplicates by hand; 6 binary-level tests (assert_cmd, per the\n  sarif_output_tests.rs precedent) for check's default terminal format,\n  --format compact, and the --fail-on-drift exit code.\n- src/snapshot.rs (+1 test): the one v1.6.1 SBOM edge case not already\n  covered — delete with a missing snapshot bails and leaves the SBOM\n  sidecar orphaned, which then blocks save_sbom for that name.\n\nThe other three v1.6.1 SBOM edge cases were already covered and are\nreconciled in the backlog: empty catalog (empty_manifest_keeps_arrays_\npresent), SPDXID collisions (spdxid_sanitization_and_collisions),\ndangling depends_on (depends_on_skips_unresolved_names).\n\nTests 423 -> 435, 0 failed. fmt + clippy -D warnings clean.\n\nCo-authored-by: Claude Fable 5 <noreply@anthropic.com>",
          "timestamp": "2026-07-27T00:17:11-05:00",
          "tree_id": "baa333b60747e1cb6de72b4772786a937050af61",
          "url": "https://github.com/rodmen07/svccat/commit/192fa76f3eabaeb9839e73fc939e30025a6c60f5"
        },
        "date": 1785129616478,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 12509,
            "range": "± 75",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 24065,
            "range": "± 777",
            "unit": "ns/iter"
          },
          {
            "name": "validate_public_url",
            "value": 314,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "reject_private_ip",
            "value": 5485,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 5163,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 12221,
            "range": "± 67",
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
          "id": "88ec3af93ba3388ce2416cf64206a7f9efcb5bae",
          "message": "refactor: one shared drift_identity_key for every drift-identity site (#39)\n\nThe kind|service|detail identity format was hand-duplicated at six sites:\nterminal.rs (drift_key), markdown.rs, junit.rs, github_annotation.rs (a\nprivate drift_key each), and two inline copies in main.rs's --baseline\nfilter plus two more in its markdown --since branch. If any one changed\nalone, --since and --baseline would silently disagree about what counts\nas the same drift.\n\nNow output::terminal::drift_identity_key is the single definition; every\nsite calls it. A guard test scans the whole src/ tree and fails the build\nif a hand-rolled copy grows back, and the --baseline flag gets its first\ntest coverage (binary-level, proving identity semantics: same identity\nwith different message/severity still suppresses; different identity does\nnot). src/diff.rs::drift_key is deliberately untouched: snapshot diffing\nkeys on service:message over DriftSummaryItem, a different contract.",
          "timestamp": "2026-07-27T01:00:50-05:00",
          "tree_id": "f41f6bf0c922498b7c9918a70d8e8907f7e27004",
          "url": "https://github.com/rodmen07/svccat/commit/88ec3af93ba3388ce2416cf64206a7f9efcb5bae"
        },
        "date": 1785132245581,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 12470,
            "range": "± 111",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 23622,
            "range": "± 99",
            "unit": "ns/iter"
          },
          {
            "name": "validate_public_url",
            "value": 259,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "reject_private_ip",
            "value": 5117,
            "range": "± 72",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 4710,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 12280,
            "range": "± 224",
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
          "id": "67ab12773054211d3be5cf9490fb25000e29fd8d",
          "message": "fix: orphaned SBOM sidecars are recoverable via snapshot delete (#40)\n\nThe v1.6.1 LOW bug from the PR #38 QA pass. delete on a missing\nsnapshot now removes an orphaned sidecar and succeeds; save --sbom\nchecks the sidecar precondition before writing the snapshot json so\nit can no longer half-finish; the sidecar-exists error names the\nrecovery command.",
          "timestamp": "2026-07-27T06:42:19-05:00",
          "tree_id": "5c913802d0516bc2a32d10a0b09cf297bca309ba",
          "url": "https://github.com/rodmen07/svccat/commit/67ab12773054211d3be5cf9490fb25000e29fd8d"
        },
        "date": 1785152725166,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 12568,
            "range": "± 81",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 23954,
            "range": "± 260",
            "unit": "ns/iter"
          },
          {
            "name": "validate_public_url",
            "value": 309,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "reject_private_ip",
            "value": 5466,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 5194,
            "range": "± 59",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 12612,
            "range": "± 178",
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
          "id": "24a7a28472a2128c6ee0a48ee8f0a8e35fcfd890",
          "message": "ci(fuzz): carry the working corpus between Continuous Fuzzing runs (#41)\n\nEvery daily campaign discarded everything it discovered: `fuzz/corpus/<target>`\nis gitignored per-job scratch space, so libFuzzer's newly found\ncoverage-increasing inputs died with the runner. The cost was invisible because\nevery run was green -- the scheduled runs of 2026-07-25 and 2026-07-31 report\nthe IDENTICAL `INITED cov:` figures for all four targets (fuzz_manifest 3496,\nfuzz_policy 1246, fuzz_url 654, fuzz_glob 128), i.e. a week of 121-second\ncampaigns produced zero cumulative progress.\n\nThe workflow now brackets the campaign with a per-target `actions/cache`\nrestore/save pair keyed `fuzz-corpus-<target>-<run id>` with a\n`fuzz-corpus-<target>-` prefix fallback. The key must be run-unique because a\nsave onto an existing key is a no-op, which would freeze the corpus at whatever\nthe first run happened to find. The committed seeds are NOT cached, so a cold or\nevicted cache degrades a run back to exactly the previous behaviour and never\nbelow it.\n\n`fuzzing_workflow_carries_the_working_corpus_between_runs` is the PR-time guard,\nsince `fuzzing.yml` never runs on `pull_request`.",
          "timestamp": "2026-08-01T13:53:27-05:00",
          "tree_id": "0f731308d25591cf5542570db566346e059cddaf",
          "url": "https://github.com/rodmen07/svccat/commit/24a7a28472a2128c6ee0a48ee8f0a8e35fcfd890"
        },
        "date": 1785610601379,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 12500,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 23341,
            "range": "± 130",
            "unit": "ns/iter"
          },
          {
            "name": "validate_public_url",
            "value": 305,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "reject_private_ip",
            "value": 5456,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 5067,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 12148,
            "range": "± 79",
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
          "id": "5519d0d8e245b26073466648dfc5d2c15ef3cc59",
          "message": "fix(gates): an empty string no longer satisfies a required field (#42)\n\nAdding the first tests for src/stats.rs surfaced that svccat had six\nseparate answers to 'does this service declare <field>?'. stats and lint\nread declared-and-non-empty; scorecard, policy and both of drift's field\nchecks read Option::is_some, so team: \"\" satisfied a policy requiring\nteam, and scorecard credited blank fields toward completeness.\n\nAll six now route through manifest::ServiceEntry::has_field.\n\nCo-authored-by: Claude Opus 5 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-08-01T14:50:34-05:00",
          "tree_id": "f8c7c33bd6da8523fbcaf409824d46d6aa35a545",
          "url": "https://github.com/rodmen07/svccat/commit/5519d0d8e245b26073466648dfc5d2c15ef3cc59"
        },
        "date": 1785614030757,
        "tool": "cargo",
        "benches": [
          {
            "name": "load_manifest_small",
            "value": 12849,
            "range": "± 126",
            "unit": "ns/iter"
          },
          {
            "name": "load_manifest_medium",
            "value": 23756,
            "range": "± 230",
            "unit": "ns/iter"
          },
          {
            "name": "validate_public_url",
            "value": 312,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "reject_private_ip",
            "value": 5465,
            "range": "± 88",
            "unit": "ns/iter"
          },
          {
            "name": "reject_ipv6_loopback",
            "value": 5118,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "analyze_dependencies",
            "value": 12334,
            "range": "± 106",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}