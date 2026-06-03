window.BENCHMARK_DATA = {
  "lastUpdate": 1780464804334,
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
      }
    ]
  }
}