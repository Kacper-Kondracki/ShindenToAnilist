# AGENTS.md

Guidance for AI agents working in this repository. Follow this file before making changes, and keep changes consistent with the architecture already present in the codebase.

## Project Shape

ShindenToAnilist is a desktop app with four main layers:

- Rust core crate: `crates/shinden-to-anilist-core`
- Rust C FFI driver crate: `crates/shinden-to-anilist-driver`
- Go/Wails backend: root Go files plus `internal/`
- Svelte 5 frontend: `frontend/`

Generated files and build assets exist in `frontend/bindings/`, `crates/shinden-to-anilist-driver/include/`, and `build/`. Do not hand-edit generated outputs.

## Source Of Truth And Boundaries

Preserve the source-of-truth chain when adding features or changing behavior:

- Rust core owns high-performance domain functionality: offline anime database loading/updating, provider interactions such as Shinden, matching, search, export, parsing, normalization, and reusable algorithms.
- Rust driver owns stateful application objective state. It coordinates core procedures, stores loaded database/list/match state, exposes the C ABI, and should preserve core performance through Rayon and low-copy or zero-copy views where that is sound.
- Go `internal/stadriver` owns CGO wrapping. It converts C data into Go values, maps driver errors, manages lifetimes/free calls, and hides unsafe details from the rest of Go.
- Go `internal/appsvc` owns Wails-facing validation, app lifecycle, OS/application concerns such as dialogs and data paths, and the stable service contract exposed to the frontend.
- The frontend consumes the Wails API through generated bindings and handwritten TypeScript adapters under `frontend/src/lib/api/` and `frontend/src/lib/domain/`.

Keep boundaries narrow. Do not move provider logic, matching logic, or export logic into Go or Svelte when it belongs in core. Do not expose unsafe driver details above `internal/stadriver`. Do not bypass the TypeScript API/domain adapter layer from Svelte components.

When changing a public shape, update every layer that depends on it in order: core model/behavior, driver FFI representation, C header generation, Go CGO wrapper/model, Wails service type, generated frontend bindings, TypeScript domain/API adapter, then UI.

## Rust Core

- Keep core APIs modular, deterministic where possible, and independent from Wails/UI concerns.
- Prefer pure functions and explicit input/output types for matching, parsing, exporting, and normalization.
- Preserve performance characteristics of database and matching code. Avoid unnecessary allocations, clones, and serial iteration in hot paths.
- Use existing dependencies and abstractions before introducing new ones.
- Before implementing nontrivial parsing, matching, indexing, text processing, concurrency, compression, or file/database logic, check whether a mature crate already solves the problem better.
- Keep external-provider assumptions explicit. If a provider format or offline database contract is relied on, document the contract near the boundary that parses it.

## Rust Driver And FFI

The driver is stateful and must be both performant and memory-safe across the C ABI.

- Every exported FFI change must define ownership, nullability, lifetime, allocation/free pair, panic behavior, UTF-8/string handling, and Go conversion expectations.
- No undefined behavior. Avoid dangling borrowed views, double frees, unchecked null dereferences, invalid UTF-8 assumptions, cross-boundary panics, data races, and unsound sharing across threads.
- Do not let panics unwind across `extern "C"` boundaries. Convert panics and domain errors into `StaError`.
- Any pointer accepted from C must be checked or have a documented safety precondition. Null plus nonzero length is invalid unless explicitly documented otherwise.
- Any pointer returned to C must have a matching free function, or must be documented as borrowed with a lifetime tied to a returned owner.
- Prefer borrowed string/slice views only when the owner remains alive for the full consumer lifetime. If that guarantee is unclear, copy into owned memory instead.
- Keep C ABI structs `#[repr(C)]`. Be careful with field ordering and type widths.
- Do not manually edit `crates/shinden-to-anilist-driver/include/shinden_to_anilist_driver.h`; regenerate it with `cbindgen` through the existing task when the exported FFI surface changes.

## Concurrency And State

- Use the smallest correct synchronization primitive. Choose `RwLock`, `Mutex`, atomics, or scoped locking according to actual read/write semantics.
- Do not serialize a code path just because it is easier if the underlying operation is safe to run concurrently and performance matters.
- Do not replace a read-heavy state path with a broad `Mutex` without a correctness reason.
- Document lock ordering and state transitions when adding shared driver state.
- Avoid holding locks while calling long-running provider/network/database/matching work unless the locked state truly must remain exclusive.
- Preserve cancellation/abort behavior. New long-running driver operations should check abort state at sensible boundaries.

## Go And Wails Backend

- Keep `internal/stadriver` as the only layer that directly touches CGO driver types and unsafe pointer conversion.
- Always pair C allocations with their matching driver/C free function or `C.free`.
- Convert C data into Go-owned values before freeing C-owned results.
- Keep `internal/appsvc` focused on validation, lifecycle, app paths, dialogs, and Wails-visible service behavior.
- Validate frontend/user input at the Wails service boundary before calling the driver.
- Return clear Go errors. Do not leak C status codes or unsafe implementation details to the frontend.
- Keep root `AppService` methods thin delegates over `internal/appsvc`.

## Frontend

Use Svelte 5, TypeScript, Tailwind, and DaisyUI in a way that keeps UI, state, and domain logic clearly separated.

### Frontend File Roles

- `.ts` files are for pure logic that works without Svelte: types, state-machine definitions, transition functions, validators, parsers, formatters, API mappers, and pure utilities.
- `.svelte.ts` files are for reusable reactive logic that needs runes: controllers, view models, reactive state machines, feature-local stores, context values, shared singleton-like module state, and reusable interaction logic.
- `.svelte` files are for UI and composition: markup, props, bindings, event wiring, snippets/rendering, context provider/consumer wiring, small local UI state, and DOM effects.
- Keep TypeScript domain types in `frontend/src/lib/domain/` and backend calls in `frontend/src/lib/api/`. Components should not call generated Wails bindings directly.
- Move non-Svelte domain logic out of `.svelte` files. If logic can be tested or understood without the component tree, it usually belongs in `.ts`.

### State Ownership

- UI/UX logic should be separate from UI markup. Components should mostly render state and wire user events to named handlers.
- Use module state in `.svelte.ts` when data is global, singleton-like, or shared by the whole app/session.
- Use context state when data belongs to a component subtree or a specific component instance, such as trees, forms, editors, menus, and workspace-local controllers.
- Use component-local state only for small, local UI concerns that do not define feature behavior.
- Source-of-truth state belongs in `$state`. Computed views of current state belong in `$derived`. External synchronization belongs in `$effect`.
- Do not duplicate source-of-truth state across components. Bubble state up only to the smallest owner that can make the relevant decision.

### State Machines And Transitions

- Model meaningful UI and workflow state as an explicit finite state machine.
- Prefer discriminated unions over scattered booleans such as `loading`, `error`, `data`, and `ready`.
- Never enter a state whose required data is missing. Before changing state, ensure all data required by the next state exists.
- Do not overload `null` or `undefined` with several business meanings. Use discriminated unions or explicit fields to distinguish omitted, unknown, empty, invalid, and present values.
- Reserve `undefined` for omitted fields. Use `null` only for explicitly empty values. Do not use either as a vague state marker.
- In API-facing types, distinguish omitted, unknown, empty, and present values explicitly in the type system.
- All state transitions must happen inside named transition functions or named event handlers. Avoid inline state mutation scattered through markup or effects.
- Async transitions should commit only when the next state is ready to be fully presented. Avoid reactive flicker caused by partially loaded or pending async state.
- Avoid one-tick pending flicker for fast async operations. Keep the previous presentable state until a meaningful loading state is actually needed, or transition atomically into the completed state.
- Use draft state when partially valid input is expected, then commit through a validation/transition function.

### Runes And Reactivity

- Prefer `$props()` over `export let`.
- Prefer callback props over `createEventDispatcher`.
- Prefer snippets/rendering over slot-heavy patterns.
- Use `$bindable` sparingly. Bindable props assume every change is valid, which can make data flow harder to understand.
- Use two-way binding when any value is valid and immediate propagation is intended.
- Prefer function handlers when input must be validated, normalized, debounced, or interpreted before it becomes source-of-truth state.
- Use a hybrid approach when draft values are acceptable locally but must be validated before commit.
- Use `$derived` only for computed views of current reactive state. Do not fetch, mutate, log as behavior, call DOM APIs, or trigger transitions from `$derived`.
- Use `$effect` only for side effects outside the reactive state graph: DOM APIs, important debug logging, animations, timers, subscriptions, persistence, and external synchronization.
- Do not use `$effect` as a substitute for derived/computed state.
- Do not use `$effect` to drive state-machine transitions. Transitions belong in named transition functions or event handlers.

### UI Composition And Layout

- Prefer DaisyUI and Tailwind utilities first. Use scoped component CSS when it gives better layout control, protects dense tool ergonomics, or matches existing component style.
- Keep the UI a focused desktop workflow tool. Avoid marketing-page composition, decorative hero layouts, and visual noise.
- Extract shared components, stores, styles, and helpers when a pattern repeats or when it clarifies ownership.
- Preserve stable dimensions for dense workspace controls and panes. Avoid layout shifts from dynamic labels, loading states, hover states, or long text.
- Design text containers so unexpected wrapping, long labels, or translated/user-provided content do not break the layout.
- Prefer explicit min/max sizes, grid tracks, flex constraints, overflow behavior, and truncation/wrapping rules for toolbars, panes, list rows, cards, tabs, and status bars.
- Do not add visible instructional copy to explain ordinary UI mechanics unless the product behavior genuinely requires it.

## Dependencies And Libraries

- Before writing nontrivial infrastructure yourself, check whether a well-maintained library already exists. This applies especially to parsing, matching/search helpers, FFI helpers, concurrency, virtualization, state management, and UI primitives.
- Prefer existing project dependencies when they fit.
- New dependencies must have a clear reason, maintenance confidence, acceptable license, and a good architectural boundary.
- Do not add a dependency to avoid understanding a small local problem.
- Do not vendor new artifacts manually unless the repository already follows that pattern and the reason is documented.

## Generated Files

Do not hand-edit generated or vendored files:

- `frontend/bindings/**`
- `crates/shinden-to-anilist-driver/include/shinden_to_anilist_driver.h`
- lockfiles unless dependency changes require them
- vendored package archives such as `frontend/prettier-plugin-tailwindcss-0.8.0.tgz`

Regenerate bindings only when the exported API surface changes:

- exported FFI structs/functions or C ABI layout changed
- Go Wails service methods or Wails-visible model types changed
- frontend-visible binding types changed

Use the existing generation tasks. Do not regenerate bindings just because unrelated code changed.

## Verification Policy

Use targeted verification first. Broaden checks only when touching shared behavior, public APIs, FFI, generated bindings, concurrency, or cross-stack contracts.

Avoid running expensive checks repeatedly during small incremental edits, especially frontend hot-reload tweaks. Run a reasonable check after the meaningful change is complete.

Do not run production builds unless the task is explicitly packaging, release, or production-build related. Debug/dev builds are the default for development because the driver is dynamically linked in dev and avoids unnecessary Go build cache issues.

### Rust

- Format Rust with `cargo +nightly fmt`.
- Use `cargo clippy --workspace --all-targets` when Rust changes warrant it.
- Prefer narrower package/target checks for small isolated changes.
- Use `wails3 task crates:build DEV=true` when a development driver build is needed.

### Go

- Use `go test ./...` when Go behavior, validation, service contracts, or CGO wrapper behavior changed.
- Prefer focused tests/checks for small validation-only changes.
- Avoid production `go build` or Wails production builds during normal development.

### Frontend

- Use `pnpm --dir frontend check` when Svelte/TypeScript behavior or types changed.
- Use `pnpm --dir frontend format:check` when formatting is relevant.
- For visual/layout tweaks under hot reload, do not run full checks after every tiny iteration.

### Wails And App Builds

- Do not call `wails3 dev`, they are reserved for manual app launching.
- Use debug/dev build paths by default, such as `wails3 build DEV=true`.
- Use package/production tasks only when explicitly requested or when validating packaging/release work.

## Testing

This application is changing quickly and much behavior depends on stateful flows and external data. Do not add tests by default.

Add or update tests only when they are trivial and valuable, especially for:

- Go service input validation
- small pure Rust logic
- driver/backend behavior that can be tested without fragile external data
- regression fixes where a focused test prevents the same bug

Do not add broad fixtures, snapshots, or high-maintenance integration tests unless the user asks for them or the change clearly requires them.

## Documentation And Comments

- Document behavior at public boundaries: FFI ownership, Wails service contracts, state transitions, and non-obvious matching/exporting rules.
- Keep comments focused on unsafe code, invariants, surprising behavior, and source-of-truth transitions.
- Avoid comments that restate obvious code.
- When changing behavior that crosses layers, update the boundary documentation closest to the source of truth.

## Change Discipline

- Read the relevant code before editing. Match local style and existing patterns.
- Keep edits scoped to the requested behavior and the layers it actually touches.
- Do not refactor unrelated code while implementing a feature or fix.
- Do not revert user changes unless explicitly asked.
- If a change affects public API shape, update dependent layers deliberately and regenerate only the necessary generated files.
- At the end of work, report what changed and which checks were run. If checks were skipped, say why.
