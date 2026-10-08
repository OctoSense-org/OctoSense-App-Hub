# Submit an app to the App Hub

English | [简体中文](SUBMITTING.zh-CN.md)

You submit an app as a bundle at a tagged commit in your own public
repository, then open an issue on this repository. Sign the bundle with your
publisher key; only an app's first version may be unsigned. A reviewer, one
of the App Hub maintainers, checks the exact bytes at your tag and runs the
gate (`hub check`) on them again. If the app passes review, the reviewer
publishes those bytes in the signed catalog that every OctoSense store reads.

The three reference apps in catalog sequence 10, the tenth signed version of
the catalog, are the worked examples. Unless a step says otherwise, the
example values come from GitHub Notes' first submission, version 0.1.0. The
rules themselves (gate checks, capability names, manifest and listing fields,
signing) are in [PUBLISHING.md](PUBLISHING.md).

```text
1 repository → 2 manifest → 3 listing → 4 screenshots → 5 stamp, check, scan, sign
→ 6 commit, tag, check a fresh clone → 7 issue → 8 review → 9 updates
```

## Before you start

A host runs your bundle: `card-host` while you develop, and an OctoSense shell
(the desktop or phone app) once people install it. A host service, such as
`github` or `model`, is shell code that an app calls through `host.request`.

### Get the tools

| Tool | Source | Use |
| --- | --- | --- |
| `hub` | App Hub `main` | Stamp the bundle (write its digest into the manifest), check, scan and sign it. Reviewers run the same code. |
| `card-host` | App Hub `main` | Run the unsigned bundle, drive it and capture screenshots. |
| `tools/octo` | [Design Flow](https://github.com/OctoSense-org/OctoScript-App-Design-Flow) | Create, run and capture an app. It wraps `card-host` and `hub`. |
| OctoSense desktop | The [0.1.0-beta.2](https://github.com/OctoSense-org/OctoSense/releases/tag/desktop-v0.1.0-beta.2) release for macOS on Apple silicon | Run apps that use host services, including [connected accounts](PUBLISHING.md#connected-accounts). |

Set up the workspace as described in
[QUICKSTART §1](https://github.com/OctoSense-org/OctoScript-App-Design-Flow/blob/main/docs/QUICKSTART.md#1-prerequisites),
then build `hub` and `card-host` from current `main`:

```sh
cd ~/octosense-ws/OctoSense-App-Hub
git pull
cargo build --release -p octosense-app-hub --bin hub
cargo build --release -p octosense-card-host --bin card-host
export PATH="$PWD/target/release:$PATH"
hub
```

`hub` with no arguments prints its usage, starting with
`hub — the OctoSense app hub command (ADR 0003)`. So does any command with
`--help` or `-h`, such as `hub check --help`. If `hub check --help` prints
`hub: No such file or directory (os error 2)` instead, your `hub` is older
than `main`: pull and build it again.

If the build fails with `no variant … TextInputStateQuery`, see
[`card-host` fails to build](DEVELOPMENT.md#card-host-fails-to-build).

An older or patched `hub` can pass a bundle that the reviewers' build refuses.
Use an unpatched build of `main`, and leave that checkout as it is until you
submit: `--catalog` reads its `catalog.json`, and the issue asks for its
revision ([step 6](#6-freeze-and-verify-the-release)).

### Know where your app runs

| Host | Runs | Does not |
| --- | --- | --- |
| `card-host` | One unsigned bundle, with a remote bridge to drive and capture it | Serve any host service except `runtime` discovery: every other `host.request` fails with `no service answers "<family>" on this device`. It also refuses signed bundles and apps that require `host-api-v1`, `backend-api-v1` or `script-tools-v1`. |
| desktop-v0.1.0-beta.2 (macOS, Apple silicon) | Installed apps, including apps that use `auth`, `github`, `gmail` and `gcalendar` | Sign in to GitHub or Google until the host has an OAuth registration in `oauth/clients.json` ([setup](https://github.com/OctoSense-org/OctoSense/blob/desktop-v0.1.0-beta.2/crates/oauth-service/README.md)); the release ships none. Sign an app in to its own backend. Install an app that requests `wasm`: its store refuses it with `unknown capability "wasm"`. Unverified on this release: live provider sign-in. |
| desktop-v0.1.0-beta.1 and home-v0.1.0-beta.1 (the only released phone build) | Store apps that use only capabilities their older app contract knows | Install an app that requests `auth`, `github`, `gmail`, `gcalendar`, `calendar`, `photos`, `youtube`, `wasm` or `palpo.*`. Their stores refuse it, for example with `unknown capability "auth"`. |

OctoSense `main` differs from desktop-v0.1.0-beta.2 in these ways, and no
release has them yet.

- A distributor can compile GitHub and Google registrations into its build. A
  build you make from source has no registrations until you add them, for
  example in `oauth/clients.json`
  ([Connected accounts](PUBLISHING.md#connected-accounts)).
- On macOS and Android, an app can sign in to the backend that its manifest
  declares and call the backend operations the manifest names
  ([Sign in to your own backend](PUBLISHING.md#sign-in-to-your-own-backend)).
- An app that declares `host-api-v1` gets the device-permission methods on
  macOS and Android ([Host API compatibility](HOST-API.md)).
- Approving a GitHub or Google Calendar save takes a physical press on the
  host's confirmation sheet, as approving a Gmail send does on both builds.
- An app agent's `glance.publish` refuses executable Splash (`script`) and L1
  card source.
- The host keeps Google Calendar events from 30 days back to 366 days ahead,
  not the calendar's whole history.

To try your app in a shell before you submit, publish it to a local catalog
([rehearsal](https://github.com/OctoSense-org/OctoScript-App-Design-Flow/blob/main/docs/PUBLISHING.md#4-rehearse-the-store-path-locally)).
The rehearsal is verified only with a shell built from source.

### Check your platform

- **macOS on Apple silicon** is verified: every command in this guide ran on it.
- **Windows** is unverified on current `main`. Open issue
  [#41](https://github.com/OctoSense-org/OctoSense-App-Hub/issues/41) records
  a native Windows 11 build and run at an earlier revision, verified by a
  community member rather than the maintainers. Run Design Flow's tool as
  `python tools/octo`; it finds `hub.exe` and `card-host.exe`. Add the
  `.gitattributes` file from [step 1](#1-lay-out-the-repository) before you
  commit the bundle, and run the final check from a fresh clone
  ([step 6](#6-freeze-and-verify-the-release)).
- **Linux** is unverified, and so are reports that frame capture (`/g`) times
  out under software rendering (llvmpipe, WSL).

### Protect your publisher key

Your publisher id names you in the catalog; your publisher key signs every
version you publish. Once a version signed by the key is in the catalog, the
Hub refuses your later versions unless the same key signs them. No command
replaces a lost key: if you lose it, ask a reviewer in an issue.

> **Warning:** The key file is the only copy of your publisher key. Keep it
> outside every repository, back it up and never share it. `hub keygen`
> refuses to overwrite an existing file, and on macOS and Linux it creates the
> key readable only by you. On Windows, keep the key in a folder that only
> your user can read.

## The three reference apps

Catalog sequence 10 holds three macOS developer previews from the publisher
`ymote`, each in two versions: 0.1.0, the first, and 0.1.1, which addresses the
three problems in [Lessons from 0.1.0](#lessons-from-010). Each app lives in
its own public repository, tagged `v0.1.0` and `v0.1.1`. The table describes
0.1.1:

| | [GitHub Notes](https://github.com/ymote/octosense-github-notes) | [Inbox Assistant](https://github.com/ymote/octosense-inbox-assistant) | [Google Calendar](https://github.com/ymote/octosense-google-calendar) |
| --- | --- | --- | --- |
| App id | `org.octosense.samples.githubnotes` | `org.octosense.samples.inbox` | `org.octosense.samples.googlecalendar` |
| Demonstrates | Markdown drafts saved as GitHub commits the person approves | Gmail reading, one shared reply draft, background triage of new mail | An agenda with event drafts, saves the person approves and an advisory app agent |
| Capabilities | `storage`, `auth`, `github` | `storage`, `auth`, `gmail`, `model`, `glance`, `octos.session.open`, `octos.turn.start` | `storage`, `auth`, `gcalendar`, `glance`, `octos.session.open`, `octos.turn.start` |
| App agent | `read-only` profile, foreground only; 3 `read` tools | `read-only` profile, `background: true`, triggered by `inbox.new_message`; 1 skill; 9 tools (4 `read`, 5 `act`) | `read-only` profile, advisory; 4 `read` tools |
| Glance card | None | Template file `glance-workspace.splash`, published with `template` and `initial` | L0 card in `main.splash`, published with `source` and `data` |
| Protected write | Commit through `github.review_save`, approved on the host's confirmation sheet | Send through `gmail.draft.review`, approved by a physical press on the host's send confirmation | Save through `gcalendar.review_save`, approved on the host's confirmation sheet |

An app agent is the AI agent that OctoSense runs for one app, with that app's
tools. `read-only` is the permission profile of the agent's own session: the
agent asks before every write to its workspace (the app's storage). It does
not cover the tools in `tools.json`, whose `risk` levels decide when a call
waits for the person using the app. Inbox's 5 `act` tools edit reply drafts,
record triage decisions and publish Glance cards without asking.

Design Flow's
[connected-apps README](https://github.com/OctoSense-org/OctoScript-App-Design-Flow/blob/main/examples/connected-apps/README.md)
explains how each app works. Its copies of GitHub Notes and Inbox have the
same fixes as 0.1.1; its copy of Google Calendar does not show the date range.

### Lessons from 0.1.0

Version 0.1.0 of each app passed review with a problem that version 0.1.1
addresses. Avoid each one from the start:

- **Never accept `script` in a card tool.** Inbox 0.1.0's background tool
  `inbox.notify`, mapped to `glance.publish`, accepted a `script` card of up
  to 16 KiB as well as a `template`. OctoSense desktop 0.1.0-beta.2 runs an
  agent's `script` card under the app's policy, so a prompt-injected
  background turn could publish arbitrary Splash code. Inbox 0.1.1
  accepts only its admitted template with `initial` data, and OctoSense `main`
  (in no release yet) refuses an agent's `script` card. Accept only
  `template` with `initial`, or an L0 `source` with `data`.
- **Declare `agent` if you ship tools.** GitHub Notes 0.1.0 shipped three tools
  in `tools.json` with `"agent": null`. The Hub admitted them as an app agent,
  and OctoSense offered that app agent, yet the gate's `grants:` line printed
  `agent none` and the store in OctoSense desktop 0.1.0-beta.2 said "Runs no
  assistant." GitHub Notes 0.1.1
  declares a `read-only` `agent` block with `AGENT.md`, so the gate prints
  `agent read-only` and every store shows the app agent
  ([An app with `tools.json` has an agent](PUBLISHING.md#an-app-with-toolsjson-has-an-agent)).
  Disclose the app agent in your privacy policy. If the app should have no
  agent, ship no `tools.json`.
- **Show a date range.** Google Calendar 0.1.0 listed the calendar's entire
  history, oldest events first: OctoSense desktop 0.1.0-beta.2 syncs every
  event, and the agent's `cached` tool returns them all. Google Calendar 0.1.1
  shows "past 30 days / next 366 days" when the host reports that range and
  "date range unavailable" when it does not, and no longer reports an event
  outside the range as deleted. OctoSense `main` (in no release yet)
  syncs only that range. Show a date range, and give the agent the selected
  event, not the whole cache.

One pattern remains in 0.1.1. Inbox's background tools can rewrite a reply
draft, including its `to` address, and only the host's send confirmation,
which takes a physical press, stops a redirected reply. Limit background tools
to reads and to writes the person approves.

## 1. Lay out the repository

The Hub admits only `bundle/`. The gate reads nothing else in the repository,
but reviewers load your privacy policy and support pages. The reference apps
also carry the files below, which reviewers used. GitHub Notes at `v0.1.0`:

```text
octosense-github-notes/
  bundle/          the submission: manifest, listing, code, tools, assets, screenshots
  PRIVACY.md       the page privacy_policy_url points to
  SUPPORT.md       how to report problems
  README.md        what the app does and how to verify a release
  publisher.json   publisher id, algorithm "Ed25519" and public key
  review/          gate output for the signed bundle (GATE.txt) and scan answers (ANSWERS.md)
  LICENSE, NOTICE
  .gitignore       keeps keys, build/ and .local-state/ out of Git
```

Keep development READMEs, keys, review packets and `.local-state/` outside
`bundle/`. Keep required asset licenses and attribution inside as `.txt` or
`.md`; links in that documentation do not grant network access. The gate checks every file
in the bundle against its rules and the 8 MiB size limit. `tools/octo new`
creates a `.gitignore` that already excludes `build/`, `.local-state/` and
`*.key`.

Add a `.gitattributes` file so that Git stores and checks out the bundle's
bytes unchanged. The reference apps lack one, so a checkout with
`core.autocrlf=true`, common on Windows, converts their text files to CRLF,
and their digests no longer match.

```sh
cd ~/apps/my-app
printf 'bundle/** -text\n' >> .gitattributes
git check-attr text -- bundle/manifest.json
```

Success prints `bundle/manifest.json: text: unset`. If the bundle is already
committed, run `git add --renormalize bundle` and repeat
[step 5](#5-produce-the-final-bytes).

Git matches the pattern from the repository root, so `bundle/**` covers only a
bundle there. If your bundle is deeper, name its real path, such as
`apps/my-app/bundle/** -text`, or write `**/bundle/** -text` to cover a
`bundle/` at any depth. Then run `git check-attr` on that bundle's
`manifest.json`, and `git add --renormalize` on its directory.
`text: unspecified` means that the pattern misses the bundle, and Git can
still convert its files.

## 2. Get the manifest right

Edit `bundle/manifest.json`. GitHub Notes' 0.1.0 manifest before stamping and
signing:

```json
{
  "schema": 1,
  "id": "org.octosense.samples.githubnotes",
  "version": "0.1.0",
  "name": "GitHub Notes",
  "integrity": { "bundle_blake3": "" },
  "capabilities": ["storage", "auth", "github"],
  "storage": { "accounts": true, "max_bytes": 4194304 }
}
```

- **`id`** is permanent. Use `[a-z0-9.-]`, at most 64 characters. The gate
  refuses ids under `os.`. The last segment becomes your tools' namespace
  (`githubnotes.*`) and must not be a reserved name; the
  [identity row](#common-refusals-and-how-to-fix-them) lists all 23.
  `tools/octo new` refuses them too.
- **`version`** is new for every submission. Your tag is `v` followed by it.
- **`capabilities`** lists only capabilities that your screens use and that a
  host serves
  ([What the Hub cannot do yet](#what-the-hub-cannot-do-yet)). The store shows
  each one to the person before install.
- **`integrity`** holds the bundle digest, which `hub stamp` writes. Never
  edit it by hand.

Then stamp the bundle:

```sh
hub stamp bundle
```

Success prints the bundle digest: 64 hex characters of BLAKE3 over every file
in `bundle/` except `manifest.json`. If `hub stamp` prints
`hub: manifest is not valid: unknown field …`, the manifest holds a field the
contract does not define; the message lists the valid ones. `hub stamp` does
not judge capabilities or the listing. `hub check` does, in step 5.

## 3. Write the listing

`bundle/listing.json` is what the store shows before install. The publisher
signature covers it through the bundle digest, so changing one word later
takes a new version. Write nothing that will go stale.

| Field | Rule | Reference apps |
| --- | --- | --- |
| `subtitle` | At most 80 characters | `"Markdown drafts with reviewed GitHub saves"` |
| `description` | At most 4000 characters | States what is fictional and what is unverified |
| `category` | One of the 15 in [PUBLISHING.md](PUBLISHING.md#the-listing) | `productivity` |
| `keywords` | At most 10 | 4 each |
| `screenshots` | 1 to 8 PNG or SVG files inside the bundle | 2 to 4 real PNG captures |
| `icon` | A square SVG or PNG inside the bundle ([ICONS.md](ICONS.md)) | `assets/icon.svg` |
| `platforms` | Only platforms you ran the app on | `["macos"]` |
| `publisher` | `name`, `support` (a URL or an email address) and an `https://` `privacy_policy_url` | The repository's Issues page and `PRIVACY.md` |
| `age_rating` | `all`, `12+`, `16+` or `18+` | `all` |

Replace every placeholder in the template's listing (the `example.com` URLs
and the "Replace with …" text), and set `platforms` to the platforms you ran
the app on. The gate accepts placeholders; `tools/octo check` flags them.

Write the privacy policy before you submit. Say what leaves the device and
where it goes: provider APIs reached through host services, text sent to the
person's model provider, Glance cards and whether the app has an app agent.
GitHub Notes corrected its policy after publishing 0.1.0, because its
`tools.json` gave it an app agent the policy did not mention.

Check the listing's links before you sign: after you push the tag, a wrong URL
takes a new version to fix. Publish the privacy page first, then request every
URL in the listing's `publisher` block:

```sh
python3 -c 'import json; p = json.load(open("bundle/listing.json"))["publisher"]; print(p["privacy_policy_url"]); print(p["support"])' \
  | grep -E '^https?://' \
  | while read -r url; do echo "$(curl -sL -o /dev/null -w '%{http_code}' "$url") $url"; done
```

Success prints `200` for each URL. GitHub Notes:

```text
200 https://github.com/ymote/octosense-github-notes/blob/main/PRIVACY.md
200 https://github.com/ymote/octosense-github-notes/issues
```

The command skips a `support` email address. `-L` follows redirects, so a
privacy page on your own domain reports its final status. `404` means a page
that is not public yet, a private repository or a typo.

## 4. Capture the screenshots

A screenshot is the one listing claim that a reviewer can check against the
running app. Run the unsigned bundle in `card-host`, drive the app to each
state over the remote bridge
([QUICKSTART](https://github.com/OctoSense-org/OctoScript-App-Design-Flow/blob/main/docs/QUICKSTART.md)
lists the routes), and capture each state. From your Design Flow checkout:

```sh
tools/octo run ~/apps/my-app/bundle --port 8141 --detach
tools/octo shot 8141 ~/apps/my-app/bundle/screenshots/01-main.png
curl -s 127.0.0.1:8141/quit
```

Success:

```text
wrote …/bundle/screenshots/01-main.png (824x1784, 32539 bytes). Look at it before you ship it.
{"ok":1}
```

Open every PNG before you use it. Never ship an error frame, an empty first
frame or a mock-up. If the app keeps animating, `shot` saves the last frame
anyway and adds
`(still changing after 2s, e.g. an animation; this is the last frame)` to its
output; look at that frame. For other capture problems, see
[QUICKSTART's troubleshooting](https://github.com/OctoSense-org/OctoScript-App-Design-Flow/blob/main/docs/QUICKSTART.md#troubleshooting).

Unverified, Linux only: if `shot` times out under software rendering
(llvmpipe, WSL), set `MAKEPAD_WRITE_FRAMEBUFFER_PNG=<file>` before you start
`tools/octo run` or `card-host`. Makepad's OpenGL backend then saves the window
to that PNG each time it draws. On macOS, `shot` stays the verified path.

A screen that needs a host service, or a native widget such as GitHub Notes'
Markdown editor, does not render in `card-host`. Capture such a screen with
fictional data in a shell that provides the service or widget. Otherwise,
capture its unavailable state, as Google Calendar's `03-host-required.png`
does. Say in the description which screenshots use fictional data.

The `hub scan` packet carries no screenshots. Reviewers see them in the bundle
and in your issue.

## 5. Produce the final bytes

Sign your bundle, as the reference apps did. The Hub also admits an unsigned
first version, but every later version of the app must then be signed
([Signing](PUBLISHING.md#signing)). To submit unsigned, skip substeps 4, 5
and 7, and run each later `hub check` with `--allow-unsigned` in place of
`--publisher-key`.

Sign last. A signed bundle no longer runs in `card-host`, and any edit after
signing means stamping and signing again. Run these commands from your app
repository:

```sh
cd ~/apps/my-app
mkdir -p build review
```

1. Stamp the final unsigned bundle.

   ```sh
   hub stamp bundle
   ```

   On an unsigned copy of GitHub Notes 0.1.0, this prints
   `d5565438b78e01b98baf268661087b7efd0f5c01799f4841daa140955f82a13b`, the
   digest in its published 0.1.0 manifest. Stamp again after any change to any
   file in `bundle/`.

2. Run the gate on the unsigned bundle.

   ```sh
   hub check bundle --allow-unsigned
   ```

   Success, for GitHub Notes 0.1.0:

   ```text
   org.octosense.samples.githubnotes 0.1.0 — PASSED
     [warning] publisher-signature: unsigned: accountability rests on the hub alone
     grants: capabilities {"auth", "github", "storage"}, hosts {}, storage 4194304 bytes, agent none
   ```

   The unsigned warning is expected here. `agent none` shows the 0.1.0 problem
   in [Lessons from 0.1.0](#lessons-from-010): tools without an `agent` block.
   GitHub Notes 0.1.1 prints `agent read-only`. If the `grants:` line lists more
   than the app visibly needs, remove the extra capabilities or storage from
   the manifest. On `REFUSED`, fix each
   `[refused]` line ([Common refusals](#common-refusals-and-how-to-fix-them))
   and start again at substep 1.

3. Write the review packet and answer its questions.

   ```sh
   hub scan bundle --packet build/review.json
   python3 -c 'import json; print("\n".join(json.load(open("build/review.json"))["questions"]))'
   ```

   `hub scan` prints two lines. The second command prints the packet's
   questions, one per line. GitHub Notes 0.1.0, trimmed:

   ```text
   wrote the review packet to build/review.json
   no --reviewer given; the packet holds 8 questions for one
   Does the app do what its name, subtitle and description claim? Cite the text in its source.
   Do the listing's platforms and category fit an app of this kind?
   …
   Route: pass, human-review, or reject. Give reasons a publisher can act on.
   ```

   The packet holds 7 questions, or 8 when the bundle ships `tools.json`,
   `AGENT.md` or skills. All three reference apps got 8. Answer each one in
   `review/ANSWERS.md` and cite the bundle file behind each answer; you paste
   these answers into the issue. Keep `build/` out of the bundle and out of
   Git.

4. Create your publisher key, once.

   ```sh
   KEY=<key-file>                 # replace with a path outside every repository
   mkdir -p "$(dirname "$KEY")"
   hub keygen "$KEY"
   PUB=$(hub pubkey "$KEY")
   ```

   `hub keygen` prints the public key, 64 hex characters, and on macOS and
   Linux creates the key file readable only by you (`-rw-------`). If the file
   already exists, `hub keygen` leaves it untouched and fails with
   `hub: cannot create new signing key "<key-file>": File exists (os error 17)`:
   keep using that key, or choose a new path. `hub pubkey` prints the public
   key whenever you need it. Pick a publisher id that nobody else has on
   record, such as your GitHub username. It becomes the key id of every
   signature you make.

5. Sign the manifest.

   ```sh
   hub sign-manifest bundle --key "$KEY" --key-id <publisher-id>
   ```

   Success prints `signed <app id> <version> with <publisher-id>`. Signing
   rewrites `manifest.json` with every default filled in, which is why the
   published manifests contain `null` fields their authors never wrote.

6. Check the signed bytes against the published catalog, as reviewers do, and
   keep the output.

   ```sh
   hub check bundle --publisher-key "<publisher-id>=$PUB" \
     --catalog ~/octosense-ws/OctoSense-App-Hub/catalog.json | tee review/GATE.txt
   ```

   The unsigned warning is gone. GitHub Notes' `review/GATE.txt` for 0.1.0:

   ```text
   org.octosense.samples.githubnotes 0.1.0 — PASSED
     grants: capabilities {"auth", "github", "storage"}, hosts {}, storage 4194304 bytes, agent none
   ```

   Without `--publisher-key`, the gate refuses a signed bundle, even with
   `--allow-unsigned`: `[refused] publisher-signature: publisher key
   "<publisher-id>" is not registered with this hub`. `--catalog` adds the
   version and publisher-continuity checks against the catalog in your App
   Hub checkout.

7. Scan the signed bundle, as the reference apps did. A scan runs only on a
   bundle the gate passes, so this confirms that your final signed bytes still
   produce the questions you answered.

   ```sh
   hub scan bundle --publisher-key "<publisher-id>=$PUB" \
     --catalog ~/octosense-ws/OctoSense-App-Hub/catalog.json --packet build/review-signed.json
   ```

   Its packet holds the same questions as substep 3. Without
   `--publisher-key`, `hub scan` stops with
   `hub: the gate refused this bundle; a scan is not offered`.

If you change anything in `bundle/` after signing, even one word of the
listing, `hub check` refuses with
`digest: the bundle hashes to …, the manifest claims …`. Restamping fixes the digest but not the signature: the next
`hub check` refuses with `publisher-signature: the signature from key
"<publisher-id>" does not match the manifest`. Repeat substeps 1, 5, 6 and 7.

## 6. Freeze and verify the release

Reviewers check out your tag and verify those bytes. The tag must point at the
commit that holds the final signed bundle.

1. Commit and tag. The tag is `v` followed by the manifest's `version`.

   ```sh
   git add .gitattributes bundle review
   git commit -m "Release 0.1.0"
   git tag -a v0.1.0 -m "My App 0.1.0"
   ```

2. Before you push, check a clone of the tag. This catches files you forgot to
   commit, a manifest stamped after the commit and converted line endings.

   ```sh
   CHECK="$(mktemp -d)/my-app"
   git clone -c core.autocrlf=false --branch v0.1.0 ~/apps/my-app "$CHECK"
   (cd "$CHECK" && hub check bundle --publisher-key "<publisher-id>=$PUB" \
     --catalog ~/octosense-ws/OctoSense-App-Hub/catalog.json)
   ```

   Success is the same `PASSED` report as in step 5. `-c core.autocrlf=false`
   gives you the bytes exactly as Git stores them, which is what reviewers
   check out. If the check fails, fix the bundle, stamp and sign it again,
   commit, run `git tag -d v0.1.0` and tag again. The tag exists only on your
   machine so far.

3. Push.

   ```sh
   git push origin main v0.1.0
   ```

   > **Warning:** Never move, delete or recreate a tag after you push it.
   > Reviewers check that the tag still resolves to the commit in your issue.
   > To change anything, release a new version.

4. Get the full commit SHA, and confirm the tag on the remote.

   ```sh
   git rev-parse "v0.1.0^{commit}"
   git ls-remote origin 'refs/tags/v0.1.0*'
   ```

   For an annotated tag, plain `git rev-parse v0.1.0` prints the tag object
   instead, and `ls-remote` prints the commit on its `^{}` line. GitHub Notes
   0.1.0:

   ```text
   5f0c4c6b13bddae87a4945b2b76ca2a416793f14
   0320a3aab446ad85a3e3abb75cd70fd7c089907a	refs/tags/v0.1.0
   5f0c4c6b13bddae87a4945b2b76ca2a416793f14	refs/tags/v0.1.0^{}
   ```

   Success: `ls-remote` shows the same commit as `git rev-parse`, on its
   `^{}` line for an annotated tag. If `ls-remote` prints nothing, the tag is
   not on the remote yet: push it (substep 3). The issue needs this commit,
   all 40 characters.

5. Check a fresh clone of the pushed tag, as reviewers will, and record the
   App Hub revision your `hub` was built from.

   ```sh
   CHECK="$(mktemp -d)/my-app"
   git clone -c core.autocrlf=false --depth 1 --branch v0.1.0 https://github.com/<you>/my-app "$CHECK"
   (cd "$CHECK" && hub check bundle --publisher-key "<publisher-id>=$PUB" \
     --catalog ~/octosense-ws/OctoSense-App-Hub/catalog.json)
   git -C ~/octosense-ws/OctoSense-App-Hub rev-parse HEAD
   ```

   Success is the same `PASSED` report, followed by the revision. The issue
   asks for both. If the version is already in the catalog, the check refuses
   it, as it now refuses GitHub Notes: `version: version 0.1.0 of
   org.octosense.samples.githubnotes is already published; publish a new
   version`. Raise `version` and release it under a new tag.

6. Optional: publish a GitHub release for the tag with an archive of `bundle/`
   and a `SHA256SUMS` file. The reference apps did, and reviewers compared the
   downloads with their hashes.

## 7. Open the submission issue

Open the
[Submit an app form](https://github.com/OctoSense-org/OctoSense-App-Hub/issues/new?template=submit-app.yml)
in this repository, one issue per version, titled
`Submit <app id> <version>`. The table gives each field, GitHub Notes' value
from its 0.1.0 submission and where to find yours:

| Form field | GitHub Notes 0.1.0 | Where to find it |
| --- | --- | --- |
| App id | `org.octosense.samples.githubnotes` | `id` in `bundle/manifest.json` |
| Version | `0.1.0` | `version` in `bundle/manifest.json` |
| Repository URL | `https://github.com/ymote/octosense-github-notes` | The public repository that holds the tag |
| Tag | `v0.1.0` | Step 6 |
| Full commit SHA | `5f0c4c6b13bddae87a4945b2b76ca2a416793f14` | `git rev-parse "v0.1.0^{commit}"` |
| Bundle path | `bundle/` | The bundle directory |
| Bundle BLAKE3 digest | `d5565438…5f82a13b` | `integrity.bundle_blake3` in the manifest |
| Publisher id | `ymote` | Your `--key-id`; for an unsigned first version, the id you will sign updates with |
| Publisher public key | `dbda2ca2…828761bc` | `hub pubkey "$KEY"`, or `unsigned` for an unsigned first version |
| Privacy policy URL | `PRIVACY.md` at the commit | `publisher.privacy_policy_url` in the listing, or the same file at the commit |
| Support contact | The Issues page | `publisher.support` in the listing |
| Platforms tested | `macos`, on Apple silicon | Every platform the listing declares and the host you ran it on |
| App Hub revision the gate ran on | `5c7a13f92fa25d36ba1fe7fb99dc9fb1b235f8d3` | The revision step 6 printed with the fresh-clone check |
| Gate output | The complete `PASSED` report | The fresh-clone check in step 6, the same report as `review/GATE.txt` |
| Scan answers | All 8, each after its question | `review/ANSWERS.md` |
| Screenshots | Links to each PNG at the commit | `bundle/screenshots/` |
| What is not verified | Live GitHub sign-in, remote commits, Windows and Linux | Everything you did not test |
| Confirmations | — | No secrets in the bundle, a tag you will not move and the fresh-clone check in step 6 |

Do not open a pull request that edits `catalog.json`, `index/` or
`artifacts/`. Reviewers write them when they publish, and every store
refuses a catalog that the Hub's own key did not sign.

## 8. What reviewers check

A reviewer verifies your submission first. The admission record for the
reference apps' 0.1.0 submissions,
[`reviews/connected-apps-0.1.0/admission.json`](../reviews/connected-apps-0.1.0/admission.json),
lists what was confirmed for each one:

| Check | Field in the 0.1.0 record | Check it yourself |
| --- | --- | --- |
| The tag resolves to the stated commit | `tag_verified` | `git ls-remote` (step 6) |
| The privacy and support URLs return HTTP 200 | `public_privacy_and_support_http` | The link check in step 3 |
| Release downloads match their hashes | `release_download_hashes_match` | `shasum -a 256 -c SHA256SUMS`, if you publish a release |
| The publisher signature verifies on the downloaded bundle | `downloaded_publisher_signature_verified` | The fresh-clone check (step 6) |
| The gate passes on the downloaded bundle | `downloaded_gate_output` | The same check |
| The bundle digest matches your issue | `bundle_digest` | `integrity.bundle_blake3` |

The record for 0.1.1, the current versions, is
[`docs/admissions/connected-apps-0.1.1`](../docs/admissions/connected-apps-0.1.1/README.md).
It also keeps each app's signed gate output and source review, and records a
test that installs each app with the store's code and upgrades it from 0.1.0.

Reviewers compare each scan answer with the bundle: the listing's claims,
platforms and category, least grants, deceptive UI, text written as
instructions to an AI agent, abusive wording and each tool's scope and risk.
A first submission always waits for a human reviewer. Reviewers promise no
review time.

`hub publish` then runs the gate against the current catalog, copies the
exact bytes into `artifacts/`, signs a new catalog and prints its sequence,
for example `published org.octosense.samples.githubnotes 0.1.1 (catalog sequence 9)`.

Admission does not prove that an app works with live providers. The 0.1.0
record states `"live_provider_login_and_remote_effects_verified": false`, and
the 0.1.1 record proves no native UI, provider traffic or physical approval.
Say in your listing what you have not verified.

## 9. After you submit

- A reviewer closes the issue with the catalog sequence your app appears
  in, or with findings to fix. Answer questions in the issue, and change
  nothing at your tag.
- **An update is a new version and a new issue.** To fix a finding or ship a
  change, raise `version`, repeat steps 4 to 6 (step 4 only if the UI
  changed) and open a new issue that links the old one. Never post a new
  version as a comment. The Hub never replaces a published version. The
  reference apps submitted 0.1.1 as new issues:
  [#130](https://github.com/OctoSense-org/OctoSense-App-Hub/issues/130),
  [#131](https://github.com/OctoSense-org/OctoSense-App-Hub/issues/131) and
  [#132](https://github.com/OctoSense-org/OctoSense-App-Hub/issues/132).
- Sign every later version with the same publisher id and key
  ([Protect your publisher key](#protect-your-publisher-key)).
- A reviewer can withdraw a version with a reason (`hub withdraw`). Stores
  stop running installed copies of that version at their next catalog fetch;
  other versions keep working. To ask for a withdrawal, open an issue with the
  app id, the version and the reason that people should see. A withdrawn
  version number stays taken, so publish the fix as a new version.

## Common refusals and how to fix them

`hub check` prints one line per finding:
`[refused|warning] <check> (<file or property>): <detail>`. A bundle the gate
cannot read gets no report, only one `hub: …` line. The full rules are in
[PUBLISHING.md](PUBLISHING.md#rules-the-gate-enforces).

| Check or message | Cause | Fix |
| --- | --- | --- |
| `digest: the bundle hashes to …, the manifest claims …` | The bytes changed after `hub stamp`: an edit, a stamp after the commit (`tools/octo check` restamps), CRLF line endings from a Git checkout or a Windows `hub` older than `main`, which hashed paths with `\`. | Build `hub` from `main`, add the `.gitattributes` from [step 1](#1-lay-out-the-repository), then stamp, sign, commit and check a fresh clone. |
| `publisher-signature: publisher key "<id>" is not registered with this hub` | A signed bundle checked or scanned without its key, even with `--allow-unsigned` or through `tools/octo check`. | Pass `--publisher-key <publisher-id>=<hex public key>`. |
| `publisher-signature: the signature from key "<id>" does not match the manifest` | The bundle was restamped after signing. | Sign again, then check. |
| `continuity: not signed by the key on record for "<id>"`, or `continuity: … is already published by "<id>"; an update must carry that key` | The publisher id is on record with another key (someone else's id or your old key), or an update is unsigned. | Use an unused id for your first app. Sign every update with the key on record. |
| `version: version … is already published; publish a new version` | That version is in the catalog. | Raise `version`, then sign, tag and check the new version (steps 5 and 6). |
| `policy: app … requests unknown capability "<name>"` | The name is not in the contract (`contacts`, `model.image`, a prefix such as `octos.`), or `hub` is older than the capability. | Use exact names from [PUBLISHING.md](PUBLISHING.md#the-manifest), and rebuild `hub` from `main`. |
| `hub: manifest is not valid: unknown field …`, with no report | `hub stamp` and `hub check` cannot parse the manifest. | Remove or rename the field. The message lists the valid ones. |
| `contents: <file> has extension "…", which a bundle may not hold` | `.DS_Store`, `LICENSE` or another file without an allowed extension. | Delete it or move it out of `bundle/`. |
| `contents-invalid (<file>): cannot decode the image: …` | A corrupt image, another format renamed to `.png` or an image over 4096 pixels a side. | Capture or export it again. |
| `resource-invalid (…/font_src): not a portable bundle path: "makepad_widgets:resources/…"` | A built-in font outside the shipped allowlist. | Use the exact Inter, LXGW WenKai Regular or Bold resource name, or bundle a licensed font subset. See [Fonts](PUBLISHING.md#fonts) for names and older-host limits. |
| `assets: <file> contains https://…` | Card data names an external URL, or script/agent guidance names an undeclared host. | Bundle the asset; declare needed script hosts and `net`. Plain `.txt`/`.md` documentation links do not trigger this check or grant network access. |
| `identity: … is under os.`, or `identity: app id "…" ends in "…", which is reserved` | The id is under `os.`, or the id or its last segment is one of `agents` `apphub` `appcard` `browser` `calculator` `card` `clock` `dev` `notes` `octos` `octoscode` `os` `reference` `reminders` `rinx` `sheets` `shell` `system` `task` `terminal` `toolbox` `weather` `workflow`. | Choose another id before your first release. |
| `listing: listing has more than 10 keywords`, `… more than 8 screenshots` or `listing platform "…" is not one of […]` | The listing breaks a limit or misspells a name. | Trim the list, or use a name from the message. |
| `listing: screenshots/01-main.png is named by the listing but is not in the bundle` | The file does not exist. | Capture it ([step 4](#4-capture-the-screenshots)) or fix the path. |
| `hub: the bundle exceeds the size limit`, with no report (`--json` shows `bundle-invalid`) | The files other than `manifest.json` exceed 8 MiB (8,388,608 bytes). | Shrink images and subset fonts. |
| `entry (main.splash): the bundle has no entry: …` | No `main.splash` or `page.card` at the bundle root. | Put the entry file at the root of `bundle/`. |
| `secrets: <file> declares is_password:true: …` | A password or one-time-code field. | Remove it. Sign-in belongs to a host service. |
| `agent: AGENT.md is in the bundle but agent.instructions does not name it` | Agent files without a matching `agent` block. | Declare `"instructions": "AGENT.md"` in `agent`, or remove the file. |
| `card-host: refused: no signature verifier is installed, so the signature from key "<id>" cannot be checked` | `card-host` runs only unsigned bundles. | Run and capture an unsigned copy, and sign last ([step 5](#5-produce-the-final-bytes)). |

## What the Hub cannot do yet

| You want | Status | Instead |
| --- | --- | --- |
| Sign in to your own backend | Only on OctoSense `main`, not yet in any release, on macOS and Android. Declare the backend in the manifest; the app then calls only the backend operations it declares, and each write waits for the person's review on the host ([Sign in to your own backend](PUBLISHING.md#sign-in-to-your-own-backend)). | On a released build, identify the person with identity-only sign-in: `auth` with GitHub's `read:user`, or Google's `openid`, `email` and `profile`. For provider data, add `github`, `gmail` or `gcalendar`. |
| Keep an API key or token in the app | Not supported. The gate refuses only password and one-time-code fields, so it does not catch a key typed into a plain field or kept in storage. | Ship no keys. For text generation, use `model`, which calls the person's own AI provider. |
| Generate images, audio, video or embeddings | Not yet. `model` serves only `model.complete` and `model.budget`. `model.image`, `model.audio`, `model.video` and `model.embeddings` are unknown capabilities ([#85](https://github.com/OctoSense-org/OctoSense-App-Hub/issues/85)–[#88](https://github.com/OctoSense-org/OctoSense-App-Hub/issues/88)). | Use `model.complete` for text. |
| Use `llm`, `news`, `calendar`, `prompt`, `ledger.read`, `clipboard` or `palpo.*` | The gate admits them, but no host serves them to a store app. `llm` and `news` answer only `os.*` apps, and `calendar` only `os.calendar`. Nothing acts on the others. | Do not request them. For Google Calendar, use `gcalendar`. |
| Run your app's own logic in agent tools | Not in any release. OctoSense `main` runs `implemented_by: "app"` tools while the full app is open; a closed app answers `app_not_running`. OctoSense desktop 0.1.0-beta.2 refuses these tools with `app_tool_unavailable`. A `host-service` tool without `host_method` calls the service named by your app's namespace, which is not a capability, so the call fails with `not_granted`. | To reach a shared service, map the tool with `host_method` to a method of `github`, `gcalendar`, `gmail` or `glance` ([Map a tool to a shared service](PUBLISHING.md#map-a-tool-to-a-shared-service-host_method)). For your own logic, declare `requires: ["script-tools-v1"]` and implement the `app_tool` hook ([Script tool execution](PUBLISHING.md#script-tool-execution-script-tools-v1)). Test it in an OctoSense shell built from `main`. |
| Submit a system app (`os.*`) or a native app | No route here. System apps ship with the shells, and native code needs a shell release ([delivery paths](DEVELOPMENT.md#choose-a-delivery-path)). | Build a store app with an id of your own. |
| Install an `auth` app on a phone | No released phone build can. | Test on desktop-v0.1.0-beta.2. |
| Name Makepad's built-in CJK font in a card kit | Supported by the current Hub and pinned runtime for the exact Regular/Bold resources. | See [Fonts](PUBLISHING.md#fonts) for names, bundled fonts and older-host limits. Native `card-host` rendering is verified on Mac; every shell/platform is not yet verified. |

Design Flow's
[HOST-SERVICES.md](https://github.com/OctoSense-org/OctoScript-App-Design-Flow/blob/main/docs/HOST-SERVICES.md)
lists which shell serves which host service.
