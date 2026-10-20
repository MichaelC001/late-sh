# deadchannel Context (late-ssh/src/app/deadchannel)

## Metadata
- Domain: the deadchannel game (`GAME.md`). Staff only (admins and moderators) by decision in code (`haunt::svc::arm` and `bootstrap_gate`): only staff are haunted, and only they can finish the ladder and join. Built for several replicas (root `CONTEXT.md`, multi-replica rule).
- Subdomains and their docs:
  - `haunt/`: the first-contact haunting ladder (`haunt/CONTEXT.md`).
  - `runner/`: the character layer, the runner and its look (§2 below).
  - `city/`, `street/`, `tailor/`, `guide/`: the night city under the clubhouse on a second `0`, the shared street over `app/presence`, the mirror that edits the look, the undercity guide (`city/CONTEXT.md`).
  - `fight/`: the runner's sheet, the day's road, the round of cards, the wire's news, the shops, crystals, Dead Air (`fight/CONTEXT.md`, with `fight/MONEY.md`, `fight/BALANCE.md`, `fight/CRYSTALS.md`, `fight/CROSSING.md`).
  - `lounge/`: the spawns above #lounge's composer (`lounge/CONTEXT.md`).
- Design sources in this directory: `GAME.md` (the game: thesis, first contact, the runner) and `DIGEST.md` (the feed budget and the welcome-back paper).
- Status: Active, staff only. Parent context: `../../../../CONTEXT.md`.

## 1. Summary

The game is never announced; it arrives. This domain will grow into the
whole character layer; what exists today is **first contact**: the
escalation ladder that onboards a person through haunting instead of a
tutorial. The chain is the spec, and the ladder never skips a rung:
three clock bursts quiet
the clock and open stage 2, the third name hit arms the stage-3 whisper
(it fires on the next fresh connect, and once more on a later day: two
doors, two different lines), and 20 hours after the second delivered
whisper the next own send plays the stage-4 breakthrough, which carries
the invitation DM in. The daily caps, not the dice, do the pacing, and
the ladder is built for a person who connects once a day: day 1 two
bursts, day 2 the third burst and the first name hit, days 3 and 4 the
other two, day 5 the first door, day 6 the second, day 7 the breakthrough
and the DM. The
day-scale gaps (whisper gap, invitation delay) are 20 hours rather than
24 so evening-to-evening connects at different times never slip a day.

Who is haunted (GAME.md, "the eligibility gate is a whisper campaign"):
**stage 1 is universal, stages 2-4 need the gate.** Stage 1 arms for
staff (admins and moderators) and nobody else, a rule in code rather than
a switch, so nothing fires for real users while copy and thresholds await
design review; opening it up is a code change. Stages 2-4
arm when the gate passes: at least `ACTIVE_MIN_HOURS` (8) of lifetime
connected time (`user_online_time.total_milliseconds`, the
online-time leaderboard's table, one primary-key read at bootstrap;
account age is not tenure, hours spent here are), at least
`TOUCHED_SETTINGS_MIN` (2) keys from the
closed `TOUCHED_SETTINGS_KEYS` list in late-core `user.rs`, and a bio of
at least `BIO_MIN_CHARS` (100) that the AI screen passed (the length is
only the floor under which no screen is spent; the screen decides
whether it reads as a person). The gate is
evaluated once at session bootstrap (`svc::bootstrap_gate`: the user row
that already loads plus the one online-time read, which fails closed to
zero hours) and is never stored: filling
your bio tonight means the static can find you tomorrow. The free legs
come first: the bio screen (a paid AI call) is only claimed once the
hours and the touched settings already pass. Eligibility
gates entering the funnel, never continuing it: any stage-2 hit on
record arms stages 2-4 whatever the bio later becomes. All three
thresholds are placeholders pending design review.

**Replica rule.** Nothing in this domain is a process-local source of
truth. The daily and lifetime caps are enforced by conditional
claims on the user row (`User::claim_first_contact_glitch_burst`,
`claim_first_contact_name_hit`): a machine decides *when to ask*, holds
its schedule, and the beat shows on the tick the claim comes back won.
The whisper stamp and the invitation are claims. The bio screen is a
claim keyed on a hash of the bio text, so any number of sessions on any
number of replicas spend one AI call per text.

## 2. Module map

| File | Owns |
|---|---|
| `haunt/` | The first-contact haunting: see `haunt/CONTEXT.md`. |
| `lounge/` | The lounge spawns: see `lounge/CONTEXT.md`. |
| `glyphs.rs` | `GLYPH_ALPHABET`, the game's shared character vocabulary. Game-level: the haunting borrows it, stage-4-era spawns will render with it (the clock glitch is retroactive foreshadowing). Distinct from the static shades `░▒▓` (noise, not creatures). `MARK_ALPHABET` is what a runner can wear as a mark: the alphabet less the Signal's `╬`, which is the paragon count in the badge and nobody's for free (migration 206 moved the marks that wore it to `┼`). |
| `runner/state.rs` | The look: `PIECES` (the closed table, one five-cell row per piece, `Slot` hood/eyes/coat, each with the `level` that unlocks it: three per slot at 1, 4, 7, 10, 13; no earned or legendary pieces, every piece is reachable by level), `Tint` (the closed palette of seven, each with its unlock `level`, white alone at 15; gold deliberately absent), `UNLOCK_LEVELS`, `unlocked_pieces` / `unlocked_tints` / `next_unlock` (the level gate), `Look` + `Worn` (typed, table references), `Look::random(level, rng)` (the join's dice at 1, the tailor's shuffle at the runner's level), `Look::to_json` / `Look::parse` (the JSON contract on the runner row; unknown codes are a `LookError`, never a blank), `PORTRAIT_WIDTH` / `PORTRAIT_HEIGHT`. No I/O. `state_test` asserts every row is five single-width cells and pins the unlock ladder whole. |
| `runner/ui.rs` | `portrait_spans`: the look as three styled spans, one per worn piece in its tint; `badge_text` (the mark and the level, `▚7`, with the marks behind the Signal's glyph once there are any, `▚3╬2`) and `level_color` (the newest tint the level unlocked: static to 3, phosphor to 6, cyan to 9, magenta to 12, red to 14, white at 15) for the wire's author header and the profile; `tint_color` maps the palette onto the theme (cyan and magenta fixed, the theme has neither). Pure. |
| `runner/data.rs` | `welcome`: the voice's welcome for a runner whose row was just created, one message, one paragraph per line (the runner mentioned by name, the story so far, `0` twice as the way down, `/leave` and `/join #deadchannel`). Names no key on the street: those are the guide's. Placeholder copy at feed-template standards. Pure. |
| `runner/svc.rs` | `RunnerLookService`: the process-shared runner directory (`watch<Arc<HashMap<Uuid, RunnerEntry>>>`, an entry being the look, level, peak level, and marks of a standing runner), seeded and refreshed from `deadchannel_runners` (`list_standing`) on the `deadchannel_runner_changed` LISTEN, re-read whole on any change. A look that fails to parse is logged and skipped. `fixed_looks_rx` for test apps. |
| `street/state.rs` | Pure: `street_view`, the street derived from presence records (one `StreetRunner` per user: the latest mover's cell, present if any session is looking), and `StreetPresence`, the `App` slot: `descend` (on the street until the session ends), `sync` (the stand this session publishes; the move stamp moves only on a move), `leave`, and the derived `view` the renderer reads (`set_records` says whether it moved). Stands off this map are dropped here. The wire is `app/presence`. |
| `tailor/state.rs` | `Draft`, the mirror's editor over one `Look` at the runner's `peak_level`: four `Row`s (hood, eyes, coat, mark), `up` / `down`, `next` / `prev` around the row's unlocked rack (wrapping), `tint` around the unlocked tints (nothing on the mark row: a colored mark is earned), `shuffle` (the join's dice at the peak); `new` snaps a piece or tint the peak has not unlocked onto the rack's first entry, so the walks never meet one they cannot place. Pure. |
| `tailor/svc.rs` | `TailorService`, the look's writer after the join: `wear_task` runs `DeadchannelRunner::store_look` (one statement, standing runner only, last write wins) and answers `TailorOutcome::{Worn, NoRunner, Failed}` on the session's `mpsc`; the metric, the log line per outcome. The change trigger carries the look to every replica's directory. |
| `tailor/session.rs` | `TailorSession`: the `draft` while the panel is open, `worn` (what the row wears as far as this session knows), the tailor's `word`, one write in flight (`saving`); `open(runner)` / `close` / `changed` / `wear` / `tick`. Decides nothing. |
| `tailor/input.rs` | Keys while the tailor's panel is open: up/down (`k`/`j`) row, left/right (`h`/`l`) pick, `t` tint, `r` shuffle, `s` wear, Enter leaves; digits, Tab, `q` stay global (`?` is the guide's, taken in `city/input.rs` first), everything else is swallowed. |
| `guide/data.rs` | The undercity guide's copy: `SECTIONS`, a heading, its neon, and its `Block`s each (`Loop`, `Prize`, `Keys`, `Figures`, `Rule`), the short version first and built for the runner who reads nothing else (the loop as a chain, the prize boxed: 40,000 chips once every 30 days and the mark, the keys, the day's numbers, six bullets), then every key and rule of the street (with the lit shops, one line each), the sheet, the road (the map, the step down, the bright glyph, the rest, the cache), the crossing, the hand (the cards, the glyph's moves, static, the run, the `a` key), new cards, the Old Signal, the armorer, patch, the lockers, the bits machine, dead air, the blade shop, ink, the ledge, the tailor, the rest of the row, the lounge, and the wire. Prose marks `` `key` ``, `*strong*`, `‹card›` (drawn in the card's neon, marks kept; a name that is no card panics, and `ui_test` reads every line), and `{action}` (the action green, or a glyph's move in its color; braces dropped); nothing else uses a backtick, an asterisk, an angle mark, or a brace. Kept out of `app/help_modal` on purpose (that one is fed to the bot). **Always current**: see `city/CONTEXT.md`. Pure. |
| `guide/state.rs` | `State`: open, scroll, and the page the renderer last measured (`record_page`, a `Cell`), so `scroll_by` holds at the end. Pure. |
| `guide/svc.rs` | `GuideService`: `claim_first_descent_task` runs `DeadchannelRunner::mark_guide_seen` (one conditional update) and answers `GuideOutcome::{FirstDescent, SeenBefore}` on the session's `mpsc`; a failed claim answers nothing and logs. |
| `guide/session.rs` | `GuideSession`: the `state`, `descend` (the claim), `tick` (opens the guide on `FirstDescent`). Decides nothing. |
| `guide/input.rs` | Keys while the guide is open: `j`/`k` and the arrows scroll a line, PageUp/PageDown a screen, Enter, `q` and `?` close it; digits and Tab stay global, everything else is swallowed. `opens` names the key (`?`). |
| `guide/ui.rs` | `draw`: the box over the street in the city's palette, the copy's marks read through the fight's vocabulary (`city/CONTEXT.md`, "The vocabulary"), each section under a neon `▚ heading ───` rule, each block drawn its own way (the loop joined by arrows, the prize in a rounded amber box, keys as amber chips in as many columns as fit and one wrapped column when narrow, figures bright beside their meaning, rules as bullets with a hanging indent, the marks read into chips and bright spans), wrapped here through `common/markdown::wrap_spans` so the row count is the page the scroll holds against; the keys under; `body_lines`. Pure. |
| `tailor/ui.rs` | `mirror_lines` for the city's panel: the draft as a portrait with the mark under it, four rack rows beside it (the cursor, the label, the tint's name, a window of up to five pieces around the worn one, bracketed, the whole rack once while it is shorter; the mark alphabet on the mark row), the keys (`[s] wear it` lit only when the draft differs from what is worn), what the next unlock level opens. Pure; `city/ui.rs::draw_panel` puts the tailor's word at the panel's foot. |

The runner's seams are as thin: `ChatService::join_deadchannel_room`
creates the row (`DeadchannelRunner::ensure_for_user`, a conditional
insert, so two devices joining at once share one face; a fresh row is
the `RunnerCreated` beat and posts the voice's welcome on the wire
(`runner/data.rs::welcome` through `post_wire_line_task`, a mention of
the runner, once per person because only the winning insert lands there;
a second device or a return finds it in the room's history), a return
the `RunnerDoor::Returned` one),
`ChatService::leave_room` stamps the leave for the `deadchannel` kind
(`mark_left`, the `RunnerDoor::Left` beat), `State.runner_looks` holds the directory
service (`main.rs` starts its listener), `App.runner_looks` is the
session's owned copy refreshed on the 1 Hz edge in `tick.rs` (bumping
`chat_ctx_epoch`, so the rows rebuild once per change; the same edge
reloads `App.fight`'s sheet for a runner and drops it for a leaver), and
chat's rows builder takes `runner_looks: Option<&HashMap<Uuid, RunnerEntry>>`,
`Some` only while the rendered room is #deadchannel: a runner's author
header opens its badge stack with the level badge (`runner/ui.rs::badge_text`,
`▚7`, painted in the level's band through `AuthorTint.runner`; the
first badge, so it sits right after the name, crown, and title),
every entry in the room wraps `PORTRAIT_GUTTER` (6) cells short, and a block-opening message by a
runner gets `attach_portrait` (the face right-aligned on the entry's
first rows, the hood level with the header, wearing what the entry has
rows for: a one-liner, header plus one body row, shows hood and eyes
and hands the coat to the first row of a continuation right under it
(none, or a divider between them, and it goes bare), anything taller
wears the coat itself, so no message grows a row for its face). The blank separator above a block stays blank, so two faces
stacked down the wire never touch. The profile modal grows a runner
column beside the late.fetch grid for a standing runner (`ProfileSnapshot.runner`,
loaded by `ProfileService::do_find_profile` from the row; no frame, its own
heading carrying the level badge, the face wounded by the missing signal beside
one key column: the signal and exp bars, the bits, the kit, the glyphs down), shown only when
the viewer is a runner too (`profile_modal::ui::draw`'s
`viewer_is_runner`, one argument to drop at the public flip;
`late-ssh/src/app/profile_modal/CONTEXT.md`). Continuations and system lines carry
no face; every other room renders exactly as before.

## 3. The four stages (behavior contract)

`haunt/CONTEXT.md`: the clock glitch, the whisper, the splash, the breakthrough and the name hit, with the root and chat seams that carry them.

## 3b. The night city

`city/CONTEXT.md`: the undercity under `0`, its map, shops, panels and art, and the `city/` files.

## 3c. The road and the round

`fight/CONTEXT.md`: the day's road, the cards, the round, the crossing, and the `fight/` files (money, crystals and balance in the docs beside it).

## 3d. The lounge (the snack: a glyph above the composer)

GAME.md, "The lounge pass": the first shared world object, a flicker or
a howler in a scene above #lounge's composer that runners put down together with
`/hit`, three flickers and a howler a day by dice, paid through the
sheet's one writer. The contract (the day row and its claims, when one
lands, the hit, the pay before the kill is told, the wire, the scene,
the copy on the channel, the beats) is `lounge/CONTEXT.md`; the file
roles and seams are there too.

## 4. Persistence (`users.settings`, late-core `User`; `deadchannel_runners`)

- `deadchannel_runners` (model `late-core/src/models/deadchannel_runner.rs`):
  one row per user (`user_id` unique, cascade on delete). Columns:
  - `look` JSONB, `{"hood": {"piece", "tint"}, "eyes": ..., "coat": ...,
    "mark": {"glyph"}}`.
  - `left_at`, the leave stamp; `guide_seen_at`, the first descent's claim
    (`mark_guide_seen`, a conditional update on the standing row, fires no
    trigger).
  - The sheet: `level`, `exp`, `signal`, `weapon_tier`, `armor_tier`,
    `bits`, `rations_left`, `day` (the UTC date of the last roll), `fight`
    (JSONB, the fight in progress or null: the quarry
    `{"quarry": {"glyph": n}}`, the round, and the three piles of the
    deck), `crystals` (`CHECK >= 0`), `drink` (the day's glass, a code from
    the closed `Drink` menu, cleared by the day roll), `peak_level` (the
    highest level ever reached, the tailor's gate), `marks` (Old Signal
    kills), `unpaid_mark`, `stash` (the locker) and `debt` (the bits
    machine, whole bits, `CHECK >= 0`), `road` (JSONB, the day's `RoadRun`:
    `{"path": [{"lane", "mark"}], "static_cards", "patched"}`, null before
    the first step and after the roll; `patched` defaults when absent),
    `cards` (JSONB, the drafted cards in draft order, `["brace",
    "bulwark"]`, null with none taken), and `inks` (JSONB, the draft levels
    inked at INK in the order bought, `[3, 9]`, null with none; kept
    through a mark and the ledge).
  - The tally: `kills` (ever), `kills_today`, `runs_today` (zeroed by the
    day roll; the last-ration line's numbers).
  - `reset_generation`, bumped by every nuke and by nothing else; the Old
    Signal's payout key is `<runner row id>:<generation>:<mark>`, so a
    mark earned again after a nuke is paid, and the nuke leaves
    `unpaid_mark` standing (chips the house still owes).

  `deadchannel_nuke_runners()` puts every sheet back to a fresh one for a
  balance test (the row, the look, and the leave and guide stamps stay).
  To nuke, ship a migration holding only `SELECT
  deadchannel_nuke_runners();`. A migration that adds a sheet column adds
  it to that function, and a column whose stored values the new code
  cannot read (a card a draft no longer offers, a fight with no deck)
  ships with a nuke.

  Rows are created only by the invited join with the column defaults
  (level 1, signal 10, 50 bits, ten rations, today). The look is written
  again only by the tailor's mirror (`store_look`, standing runner only).
  An insert, or an update of `look`, `left_at`, or `level` (`peak_level`
  and `marks` only move with the level), fires
  `deadchannel_runner_changed` (payload: the user id, for logs only;
  listeners re-read every standing row); a sheet write that leaves the
  level alone fires nothing, so ten fights a day per runner wake no
  directory, and a level gained reaches every badge and HUD.
  - The sheet is written whole by `store_sheet` under `lock_standing`
    (`SELECT ... FOR UPDATE` on the standing row: `None` for a runner who
    left, so the door being shut also shuts the fight). The service is
    the only caller of both.
  - `/leave #deadchannel` stamps `left_at` (`mark_left`, conditional on
    the stamp being absent, so leaving twice writes and notifies once) and
    never deletes: the character keeps its row, its id, and its face, and
    an invited rejoin clears the stamp and gets that face back
    (`ensure_for_user`, whose `RunnerOrigin` is `Created`, `Returned`, or
    `Existing`, one statement per outcome). A stamp is also what keeps the
    directory honest, because the trigger fires on insert and update only:
    a delete would notify nobody and leave the undercity open on every
    replica that missed it.
  - `list_standing` serves only rows with `left_at IS NULL`, so one write
    closes the gate and drops the portrait everywhere. A runner who left
    disappears from the #deadchannel gutter retroactively, old messages
    included: going dark takes the face with it.

- `first_contact_glitch_hits` (int) + `first_contact_glitch_day`
  (YYYY-MM-DD) + `first_contact_glitch_day_hits` (int): stage-1 bursts.
  `claim_first_contact_glitch_burst` increments all three in one
  conditional `UPDATE` (lifetime under `total`, today's under `daily`,
  the day rolling in the same statement) and returns `Won { hits }` or
  `Capped { hits }`; `record_first_contact_glitch_hit` is the uncapped
  increment for forced (`/haunt glitch`) bursts only.
- `first_contact_name_hits` + `first_contact_name_day` +
  `first_contact_name_day_hits`: stage-2 hits, same shape
  (`claim_first_contact_name_hit`; `record_first_contact_name_hit` for
  forced hits). The third hit arms stage 3.
- `first_contact_whisper_hits` (int) and `first_contact_whisper_at`
  (RFC3339): stage-3 deliveries and the last one's time, written only by
  `claim_first_contact_whisper` (conditional on the cap and the gap);
  the counter at its cap schedules stage 4 from the stamp.
- `first_contact_bio` (object `{hash, verdict, at}`): the bio screen
  cache. `claim_first_contact_bio_screen` stamps `pending` for a hash
  when no verdict exists for it, or the one on record is not `passed`
  and is older than `BIO_RESCREEN_AFTER_HOURS` (24);
  `set_first_contact_bio_verdict` lands `passed`/`failed` only while
  that hash is still on record. Not a chain mark: `/haunt reset` leaves
  it alone (rewrite the bio to re-screen).
- `first_contact_invited_at` (RFC3339): stage-4 claim, written only by
  `claim_first_contact_invitation` (conditional on absence), taken back
  by `release_first_contact_invitation` when the send after a won claim
  fails.
- `reset_first_contact` wipes the six chain keys and both stamps (the
  `/haunt reset` hook).
- The stage-2 wire has **no table on purpose**
  (`late-core/src/models/deadchannel_name_hit.rs`): a name hit is a second
  and a half of theater whose mark is already a conditional claim on the
  user row, so there is no truth to store. The channel carries a
  self-contained payload
  (`<message>:<room>:<user>:<seed>`), the publisher's pooled
  connection is not the listener's (so Postgres hands the beat back to the
  publishing replica too, and every session hears it exactly one way), and
  a replica that boots mid-beat misses it like a person who was not
  looking. Nothing to sweep, nothing to migrate.
- `deadchannel_ambushes` (model
  `late-core/src/models/deadchannel_ambush.rs`): one row per fight
  between two runners (`fight/CONTEXT.md`, "The crossing"): `day`, the attacker and the
  defender (ids and usernames as they were), `step` (null for an
  answer), `stake` (the most a win takes: `CROSSING_TAKE_PERCENT`,
  half, of the defender's bits on hand at the start),
  `answer_to` (the crossing an answer settles), `started_at`,
  `ended_at` with `outcome` won, lost, ran, or dropped, and
  `bits_moved` from the attacker's side. The three unique partial
  indexes are the caps: a sleeper crossed once a day, a pair crossing
  once a day, a crossing answered once; an insert that loses one is the
  cap refusing (`Ambush::insert` answers `None`). Rows are kept.
- `deadchannel_lounge_days` and `deadchannel_spawns` (model `late-core/src/models/deadchannel_spawn.rs`): the lounge's day
  budget (the dice and the spent flags, `last_spawn_at` for the gap) and
  its spawns (`kind`, `hits_needed`, `hits` as JSON `[{"user_id",
  "username", "at_ms"}]`, `spawned_at`, `expires_at`, `ended_at` with
  `outcome` slain or slipped, `paid_at` the pay claim). The
  `deadchannel_spawn` channel carries the spawn row as JSON, so a
  listener paints without a lookup. Rows are kept; nothing sweeps them.
- Everything else is render-only and session-local: no chat rows, no IRC
  projection (the invitation DM is the deliberate exception: stage 4 is
  where the fiction goes real, and an invitation that vanishes cannot be
  followed three days later). A witnessed beat is render-only too: the
  message body, the row, and the IRC projection are all untouched, so the
  corruption exists only on the screens that were looking.

## 5. `/haunt` (admin composer command)

`haunt/CONTEXT.md`: admin-only, parsed in `chat/state.rs::submit_composer`; for anyone else the line posts as plain text.

## 6. Gotchas

- A hit shows one tick after the claim wins, not on the tick the dice
  landed (one DB round trip). For the flicker that is still on the
  landing echo's ~800ms hold; the glitch never had a moment to miss.
- On this replica the room sees a hit within a tick of the person being
  haunted: the chat broadcast (`ChatEvent::MessageCreated`) lands the
  message in every local session before the sender's claim is even out.
  Only a session on *another* replica learns of the message from the chat
  snapshot refresh (`CHAT_REFRESH_INTERVAL`, 10s), so there the beat is
  heard first and the name corrupts as the message *arrives*, then
  heals. That is why chat holds the beat rather than the haunting
  painting on receipt, and why `NAME_HIT_WAIT` (30s) has to stay
  comfortably above that cadence. Do not reach for a shared clock to make
  the waves simultaneous; it would cost persistence and buy nothing the
  fiction wants.
- A witnessed beat is spent when it starts, and "started" only means the
  message is in this session's copy of the room, whether or not that room
  is on screen. A witness who is off in the arcade spends the beat
  without seeing it. That is the same bargain the own-flicker makes (it
  fires on the landing echo and trusts the eye is there) and cheaper than
  teaching the haunting what is on screen; if it turns out to matter, the
  fix is a visibility gate like the clock glitch's, not a queue.
- If stage 2 looks dead from a second session, check that the session
  holds the room (a beat for a room it is not in is dropped on receipt),
  that chat's message listener is running (tests and headless paths do
  not start it, so there stage 2 stays private to the session that rolled
  it), and that the message landed inside `NAME_HIT_WAIT`.
- The wire is a delivery path, never a source of truth, so it is one of
  the few pieces of deadchannel state that is process-local by design.
  The caps it could be tempted to enforce already live in the row claim.
- `screen_bio` fails closed at every step: AI off means `BioStanding::AiOff`
  (no claim, no pass, unless a pass is already on record, which is
  final); a broken call leaves the pending claim to expire rather than
  releasing it, so a flapping API costs at most one call per bio text
  per day.
- Clock domains differ on purpose: the whisper runs on `splash_ticks`
  (the splash's own typing clock), the glitch and flicker on
  `marquee_tick` (wall-derived 66ms units).
- Input swallowed by the held door leaves the VT parser mid-escape; both
  the input path and the release path call `vt_input.reset()`. Input
  swallowed by the breakthrough is dropped before the feed, so it needs no
  reset.
- Every voiced or corrupted character obeys the screenshot test (static /
  signal / city / channel vocabulary, never Unix internals); the whisper
  pool and the invitation plea need feed-template-grade variety before
  leaving staff scope (GAME.md, Open questions).
- The invitation runs through `ChatService::send_message`, so DM
  delivery, unread badges, and IRC projection behave like any DM.
- Test apps pass `FirstContactMarks::spent_for_tests()` and
  `FirstContactGate::closed_for_tests()` so no stage can fire in a test
  unless armed on purpose (`test_helpers` compiles unconditionally, so
  those helpers carry no `#[cfg(test)]`).
- `right_sidebar_visible` was made `pub(crate)` for the glitch's
  visibility gate; it still lives in `tick.rs`.
- The look directory starts empty and fills on the listener's first
  load, so a portrait can be absent for the first seconds after a
  replica boots; a session copies the directory on its next 1 Hz tick.
  The `mark` is painted as the level badge in the #deadchannel author
  header and the heading of the profile's runner column; the clubhouse floor glyph
  is the next slice.
- Where to look when the ladder seems dead (per person, in the logs, all
  keyed by `user_id` and `username`, the two fields every deadchannel line
  carries): `first contact gate evaluated` at every staff connect
  with each leg's number, the bio standing, and the `GateVerdict`
  (nobody else gets a line: the gate is shut before any read); `first
  contact armed` for every session that can fire stage 1, with `chosen`
  and `whisper_armed`; then one line per hit, whisper, breakthrough, invitation, bio
  screen, and runner. How many the gate turns away, and on which leg, is
  `late_ssh_first_contact_gate_total{verdict}` (one count per
  connect, not per person); bio screens by outcome are
  `late_ssh_first_contact_bio_screens_total`; delivered beats are
  `late_ssh_first_contact_beats_total`. The street's wire is
  presence's, `late_ssh_presence_total{beat}` (`late-ssh/CONTEXT.md`). The
  game's own counters sit under the same row: fights by beat (the per-
  exchange `round` beat left out; the crossing's `crossed`, `answered`,
  `ambushed`, `downed` ride the same counter), the tailor and the door, Old Signal
  payouts, and the lounge (`late_ssh_deadchannel_lounge_total{beat}`:
  spawned, hit, slain, slipped, paid, pay_left, pay_failed, hit_refused,
  hit_failed). The name is a log field only, never
  a metric label: the three counters stay keyed on closed enums so the
  series count cannot grow with the player base. Grafana's "deadchannel"
  row (`monitoring/dashboards/observability.json`) reads both: the beat and
  gate counters as reset-safe `max_over_time` sums, and from the log
  lines a Runners table (one row per person: ladder hits, invited and
  joined times, last seen, latest gate verdict and legs), a Recent Beats
  table, and the Haunt Log, all three filtered by the `$runner` regex
  variable. The log tables use the `instant` query type and extract
  fields, because a `stats` query splits every group into its own series
  and cannot carry strings; logs keep 7 days, so older rungs show empty
  there and `users.settings` stays the truth. Counters live on the pod, so a
  deploy zeroes the live value; every panel there sums per-instance
  high-water marks instead. The row ships to prod with the dashboard
  ConfigMap, on a `-infra` release (`infra/monitoring.tf`), not on merge.
- A fight action holds a pooled connection while it waits for the row
  lock, so `FightSession::request` drops a press while one is out (the
  bonsai rule); the scene shows `the static is deciding.` meanwhile.
- The fight's dice are `rand::thread_rng` in the service; the state
  takes any `Rng`, so `state_test` drives a fight to the end with a
  seeded `StdRng` and asserts the whole sheet.
- The city map is generated: hand edits to `city/map.rs` are clobbered by
  the next `scripts/gen_city_map.py --write`. Move a prop in the script
  and its zone, reach, and animation cells move with it. The literal
  arrays carry `#[rustfmt::skip]`, so `cargo fmt` and `--write` agree.
- The city's copy (`city/data.rs`: shop lines, cart lines, draft move
  names, notices) is placeholder content at feed-template quality
  standards and faces design review with the rest of the phase 2 copy.
- Piece rows are five cells with no wide glyph; the state test guards
  that and nothing more. The rows are block, box-drawing, and shape
  glyphs (`◈ ◌ ●` and their kin), which are East Asian ambiguous width
  like the rest of the TUI's frames, so portraits assume the same
  ambiguous-narrow terminal the whole app does. Any new piece with a
  shape glyph should still be checked in the terminals people here use
  before it ships.
