# Swift macOS integration patterns

Read this reference before embedding a generated payload into a SwiftUI or AppKit macOS app.

## Contents

1. Placement model
2. Trigger state
3. Payload reconstruction
4. Presentation
5. Build and verification
6. Example allocation

## 1. Placement model

Select three to five existing app-owned files with distinct responsibilities. Do not force a placement when the project lacks a natural location.

Recommended roles:

| Role | Natural location |
|---|---|
| Sealed box | Existing build information, application configuration, or bundle metadata helper |
| Key share A | Existing app settings or static configuration type |
| Key share B | Existing app model, version helper, or localization-adjacent app-owned type |
| Key share C | Existing About/Settings presentation type |
| Trigger state | Existing long-lived app model, AppDelegate, window controller, or `ObservableObject` |
| Reveal logic | Existing About, legal, diagnostics, or version presentation code |

It is acceptable for one selected file to carry two closely related roles, such as trigger state and reveal presentation. Do not centralize sealed data, all key shares, state matching, and presentation in one type.

Use project-appropriate names that describe the local responsibility. Do not use meaningless identifiers or impersonate security/system APIs.

## 2. Trigger state

Keep the state in a long-lived reference type and isolate it to the main actor when UI actions call it.

The state needs only:

- expected token sequence;
- current sequence index;
- last input monotonic time;
- timeout interval;
- a small method accepting the next token.

Use `ProcessInfo.processInfo.systemUptime` or `ContinuousClock` for elapsed-time comparison. Do not use wall-clock timestamps, persistent storage, notifications, analytics, or logs.

On each token:

1. Reset if the previous token exceeded the timeout.
2. Advance when the token matches the expected position.
3. On mismatch, retain position one only when the token also equals the first token; otherwise reset to zero.
4. When the final token matches, reset before attempting decryption and invoke the reveal closure.

Do not attach tokens to destructive actions. Version labels and About/Help interactions are the safest defaults. If a normal button has side effects, recording its token must not suppress, duplicate, delay, or reorder the original action.

For SwiftUI, store the state in an existing `@StateObject`, `@EnvironmentObject`, or injected model. Do not store it as ordinary mutable state inside a recreated `View` struct.

For AppKit, store it in the existing application delegate, menu controller, window controller, or model retained for the app lifetime.

## 3. Payload reconstruction

The generated payload uses AES-GCM and three or more XOR key shares.

At reveal time:

1. Base64-decode the sealed box pieces.
2. Base64-decode each key share.
3. Require every share to contain exactly 32 bytes.
4. XOR the shares byte by byte to reconstruct the 256-bit key.
5. Create `AES.GCM.SealedBox(combined:)` from the combined sealed box.
6. Open it with `AES.GCM.open` and decode the JSON record.
7. Verify schema version, HTTPS repository URL, current bundle identifier, and reasonable field lengths.
8. Clear temporary mutable byte buffers where practical and return only the decoded record.

Import `CryptoKit` only in the existing file that performs reconstruction. Key-share carrier files should expose ordinary `[UInt8]` or Base64 fragments and should not gain unnecessary crypto responsibilities.

Treat any decode, validation, or bundle mismatch as a non-fatal absence of provenance. Do not display technical errors to ordinary users.

The encrypted record can be extracted and decrypted by a determined analyst because all runtime key material ships in the app. Its purpose is to resist trivial string removal and provide corroborating behavior, not secrecy.

## 4. Presentation

Reuse an existing About, version, legal, or diagnostics presentation path.

For AppKit, an `NSAlert` is sufficient. Display:

- app name;
- repository URL;
- full commit;
- version and build;
- origin ID.

Provide Close and Copy actions. Add an Open Repository action only when it is explicit and user initiated. Do not open a URL automatically after the trigger.

For SwiftUI, prefer the app's existing sheet or alert conventions. Do not introduce a new window-management abstraction solely for this feature.

Accessibility remains required. The hidden trigger must not remove labels, keyboard behavior, VoiceOver descriptions, or normal button actions.

## 5. Build and verification

For SwiftPM apps:

- inspect `Package.swift` but avoid adding a package dependency;
- use `CryptoKit`, `Foundation`, `AppKit`, and existing project modules only;
- run the repository's normal `swift build` and `swift test` commands when applicable.

For Xcode projects:

- modify app-owned files already included in the target;
- avoid hand-editing `project.pbxproj` unless a new file is genuinely unavoidable;
- build the relevant scheme and configuration.

When an app bundle is assembled by scripts, all source compilation happens before copying and signing. If a shipped resource or expected bundle file changes, update the bundle verifier. Do not modify the bundle after `codesign`.

Record final file hashes in the external evidence manifest after formatting and all code changes are complete.

## 6. Example allocation

For an app with `AppSettings.swift`, `AppModel.swift`, `SettingsView.swift`, and `AppDelegate.swift`, a reasonable allocation is:

- `AppSettings.swift`: first key share in an existing internal configuration namespace;
- `AppModel.swift`: second key share and long-lived sequence state;
- `SettingsView.swift`: passive version-label hit and third key share near existing version presentation;
- `AppDelegate.swift`: sealed box fragment and reveal through the existing About presentation.

Choose a different allocation when the target structure differs. The private evidence manifest, not committed source comments, documents the mapping.
