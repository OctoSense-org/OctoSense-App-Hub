# Connected app preview admission

English | [简体中文](README.zh-CN.md)

Catalog sequence **7** admits three independently installed macOS developer
previews from ymote. The user's instruction to finish runtime delivery and
catalog acceptance authorized publication. Separate reviewer agents examined
the publisher packages, capabilities, privacy disclosures and compatibility fix.
This records agent review and user publication authorization; it does not claim
an independent human completed live provider acceptance.

| App | Publisher commit | Submission |
| --- | --- | --- |
| GitHub Notes | `5f0c4c6b13bddae87a4945b2b76ca2a416793f14` | [#121](https://github.com/OctoSense-org/OctoSense-App-Hub/issues/121) |
| Inbox Assistant | `28dd24a39a7e9668b877dd84d9577d10b3f87363` | [#122](https://github.com/OctoSense-org/OctoSense-App-Hub/issues/122) |
| Google Calendar | `c0783291a528689008d7213f1b1163c9c15ac874` | [#123](https://github.com/OctoSense-org/OctoSense-App-Hub/issues/123) |

[admission.json](admission.json) records bundle digests, source tags, public
release verification and catalog identity. Each exact publisher-signed bundle
passed `hub check`; `hub publish` reran admission, copied unchanged files and
signed the catalog with the existing certified working key. Every artifact file
was compared by SHA-256 with its publisher source. Index entries are exact copies
of the admitted entries; neighboring pack files contain the same bundles.
No signing keys, provider credentials, private profiles or raw runtime logs are
included. The existing public trust anchor is unchanged.

Before admission, the publisher scan answers were reviewed for visible function,
platform/category, least necessary grants, misleading UI, embedded assistant
instructions, abusive/private content, tool risk and routing. The result is
approval for the disclosed macOS developer preview scope. All tools exposing
private provider data are non-shareable by default. Notes exports no write tool;
Inbox exports no send tool; Calendar exports no booking tool. Account management
and external-action approval remain host-owned. Provider/model traffic is
explained in the publisher privacy policies and the corrected Store summary.

Compatibility review found that old strict catalog readers rejected `host_method`
in display metadata. The fixed publisher omits only that field from catalog tool
summaries. Original signed `tools.json` mappings remain byte-identical and are
loaded for execution. Regressions verify old field-shape parsing and canonical
signature bytes, unchanged installed mappings, and tamper/forbidden-route refusal.
This is a legacy wire-shape test, not execution of an old desktop binary.

These records establish admission and integrity. They do not establish a live
GitHub or Google connection, provider writes, physical approval, Android Google
authorization, Windows/Linux UX, or absence of long-run memory growth. Existing
native/synthetic-provider and model evidence retains its original source hashes
in the publisher repositories. Official HTTP/native Store acceptance is recorded
separately after publication; it is not inferred from this staged catalog.
