---
name: embed-app-origin-traces
description: Embed project-specific, offline origin evidence into existing Swift macOS apps during development without adding a removable shared package. Use when a user asks to add encrypted repository provenance, a hidden multi-click reveal path, distributed source markers, or forensic origin traces across SwiftUI/AppKit app code and build flows for later authorship or fork analysis.
---

# Embed App Origin Traces

## Goal

Add a small, project-specific provenance path to an existing Swift macOS app. Integrate it into a few natural locations instead of introducing a shared runtime package or a single trace-named source file.

Treat every marker as removable corroborating evidence. Do not claim tamper resistance, exclusivity, or prevention of GPL-authorized modification and resale.

## Required workflow

1. Read the target repository's `AGENTS.md` and applicable project instructions completely.
2. Inspect `git status`, `Package.swift` or the Xcode project, app entry points, long-lived models, About/Settings UI, build scripts, verification scripts, licenses, and existing source links.
3. Confirm the requested repository URL and normalize SSH Git remotes to an HTTPS URL. Resolve the current full Git commit, bundle identifier, app version, and build number when available.
4. Select one harmless trigger and three to five existing integration files. State the exact files, line regions, and reason for every file before editing.
5. Preserve unrelated and pre-existing changes. Do not refactor or create new abstractions beyond what is required for the provenance path.
6. Generate a fresh encrypted record and private evidence manifest with `scripts/seal_origin_record.swift`. Generate into a temporary directory first.
7. Read [references/swift-macos-integration.md](references/swift-macos-integration.md) completely before modifying a Swift macOS target.
8. Integrate the generated values across the selected existing files with `apply_patch`.
9. Copy the private manifest outside the target repository, defaulting to `~/.codex/app-origin-evidence/<bundle-id>/<origin-id>/manifest.json`. Tell the user the exact path. Set permissions to `0600`.
10. Add integration point paths and stable code anchors to the private manifest. Never add that manifest, the trigger sequence, or a plaintext copy of the encrypted record to the target repository.
11. Build and run the narrowest existing verification appropriate to the app. Exercise the trigger when practical and confirm decryption failure has no effect on normal app behavior.
12. Review the final diff for scope, code-signing order, concurrency, UI side effects, GPL notices, and downstream verification scripts.

## Integration constraints

- Modify at most five existing app-owned source or build files unless the user approves more.
- Do not create `OriginTrace.swift`, `Watermark.swift`, or another single removable runtime implementation file.
- Keep each contribution local to an existing responsibility: build metadata, app configuration, long-lived UI state, About/Settings presentation, or bundle verification.
- Distribute the sealed payload and three key fragments across distinct app-owned files. Do not place them in dependencies, generated files, license text, third-party notices, or vendored source.
- Keep trigger state in an existing long-lived reference type. Do not store it in a transient SwiftUI `View` value.
- Prefer passive, repeatable interactions such as taps on version text, About, Help, history, or settings navigation. Do not use Quit, Delete, Reset, purchase, update installation, permission prompts, or other destructive or externally visible actions.
- Keep trigger state only in memory. Reset on timeout, mismatch, reveal, app restart, or scene recreation.
- Decrypt only after a complete trigger. Present a local `NSAlert` or existing SwiftUI sheet containing repository, commit, version, and origin ID.
- Never contact a server automatically. Opening the repository URL requires a separate explicit user action.
- Decryption, decoding, or presentation failure must silently reset the trigger and leave all normal features available.
- Preserve visible GPL notices and normal corresponding-source delivery. The hidden path is never a substitute for license compliance.

## Marker selection

Use a bounded combination of these markers:

1. **Encrypted record**: AES-GCM sealed JSON containing repository URL, bundle identifier, commit, version, build, issue time, and random origin ID.
2. **Split key material**: three XOR shares placed in unrelated but appropriate app-owned types. All shares are required only during the reveal operation.
3. **Operation path**: a short, harmless sequence of existing UI interactions with a five-to-ten-second timeout.
4. **Build binding**: include the Git commit and bundle identity in the record; optionally add the origin ID to an existing build manifest that is already intended to ship.
5. **Private evidence**: retain plaintext record, encrypted payload, key shares, trigger tokens, integration anchors, and hashes outside the repository.

Do not add every available marker merely to increase count. Prefer three or four independent signals that fit the app naturally.

## Generate the record

Create an isolated temporary directory and run:

```bash
swift <skill-directory>/scripts/seal_origin_record.swift generate \
  --repository "https://github.com/owner/repository" \
  --bundle-id "com.example.application" \
  --commit "$(git rev-parse HEAD)" \
  --app-name "Example App" \
  --version "1.2.3" \
  --build "123" \
  --trigger "version,version,about,version" \
  --payload-output "$temporary_directory/payload.json" \
  --manifest-output "$temporary_directory/manifest.json"
```

Verify the generated files before integration:

```bash
swift <skill-directory>/scripts/seal_origin_record.swift verify \
  --payload "$temporary_directory/payload.json" \
  --manifest "$temporary_directory/manifest.json"
```

The payload file is temporary input for implementation. Do not copy it wholesale into the app. Place its sealed box and key fragments separately according to the selected integration plan.

The generator intentionally provides concealment rather than authenticity. If the user already has a release signing key or Sigstore workflow, sign the private manifest or bind its hash to the release evidence. Never create, expose, or commit a private signing key automatically.

## Private evidence manifest

After integration, append an `integrationPoints` array to the private manifest. Each item must record:

- repository-relative path;
- stable type, function, property, or view name;
- marker role without relying only on line numbers;
- post-change SHA-256 of the containing file;
- whether the point carries sealed data, a key fragment, trigger state, a trigger hit, or reveal logic.

Also record the final build artifact SHA-256 when an artifact is produced. Avoid machine identifiers, usernames, absolute source paths, serial numbers, device identifiers, or user data.

## Validation checklist

- The app builds with its normal SwiftPM or Xcode command.
- Existing tests or verification scripts still pass.
- The normal UI behaves identically before the complete trigger.
- Wrong order and timeout reset the sequence.
- The complete sequence reveals the expected record.
- A malformed payload or missing fragment does not crash, block, or degrade the app.
- No automatic network request, telemetry, persistence, or device collection was introduced.
- App signing occurs after all bundle or binary modifications.
- Existing file inventories and bundle verification are updated only when a shipped file actually changes.
- The private manifest is outside the repository and mode `0600`.

## Prohibited techniques

- Remote keys, callbacks, device fingerprinting, sales tracking, or hidden analytics.
- Remote disablement, timed failures, crashes, deletion, or feature restrictions.
- Anti-debugging, anti-disassembly, executable packing, or self-modifying code.
- Unicode bidirectional controls, zero-width source tricks, misleading identifiers, or code hidden in legal text.
- Changes to third-party, generated, security-critical, authentication, payment, update-signing, or data-deletion code.
- A secret required to build, run, modify, or redistribute the GPL-covered app.
- Claims that the marker cannot be removed or that its removal violates the software license by itself.

## Delivery

Report:

- every modified file and its marker role;
- the private evidence manifest path;
- the selected trigger in a private user-facing note, not in committed documentation;
- build and verification results;
- assumptions, removal limitations, and any downstream verification changes.

Do not print key fragments or the complete private manifest in the final response.
