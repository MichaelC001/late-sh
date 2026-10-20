# The haunting (late-ssh/src/app/deadchannel/haunt)

Part of Dead Channel (`../CONTEXT.md`; design in `../GAME.md`, first contact). The first-contact ladder that haunts staff into the game: the clock glitch, the whisper, the splash, the breakthrough, and the name hit on the wire.

## Files

| File | Owns |
|---|---|
| `haunt/state.rs` | The pure machines and data: `HauntState` (the one `App` slot), `FirstContactMarks` (persisted marks bundle), `FirstContactGate` + `BioStanding` + the thresholds and `bio_hash` (the eligibility gate), `ClockGlitch` (stage 1), `NameFlicker` (stage 2, the person being haunted), `ActiveHit` (one hit's playback, holding the wave seed: the roller's own hit and the `witness` slot share it, so every screen corrupts identically), `WhisperState` (stage 3), `Breakthrough` + `InvitationClaim` (stage 4's full-screen beat and the claim answer), the voice/invitation/breakthrough-line constants (stage 4), `PendingClaim`/`HitStage` (claims in flight), `HauntCommand` + `parse_haunt_command`. No I/O, no clock reads. |
| `haunt/svc.rs` | Orchestration: `bootstrap_gate` (gate + bio screen claim at connect), `arm` (session start), one `tick(app)` (claim drain, splash door, glitch scheduler, name-flicker roller, witness replay, breakthrough (claim answer, scene, due send), `/haunt` drain), `swallows_splash_input`, `replay_whisper`, the bio screen task, and `publish_name_hit` (a won or forced hit goes on the wire through `ChatService::publish_name_hit`). The only haunting layer touching `App`, logging, metrics, and persistence. |
| `haunt/ui.rs` | Pure render helpers: whisper frame + splash overlay + static surge, breakthrough frame + full-screen draw, `apply_clock_glitch`, `glitched_name`, `name_flicker_for`. Deterministic per burst seed, stateless like the sidebar equalizer. |

## Seams

Root integration is deliberately thin: `App.haunt` (the one field),
`haunt::svc::tick(self)` in `tick.rs` (plus the splash block consulting
`HauntState::holds_splash_door` before self-expiring), two input lines
(splash input to the held door, and a swallow while the breakthrough
plays, first thing in `App::handle_input` so no door passthrough sees
the keys), `HauntState::breakthrough_playing` keeping `wake_hint` hot, and
the draw hooks in `render.rs` (clock transform, whisper and breakthrough
frames for `DrawContext`, splash overlay, the breakthrough painted last
over every modal).
Chat's seams: the `/haunt` submit hook (admin-gated), the
`requested_haunt` slot, the `own_send_succeeded` flag (a `SendSucceeded`
for a request this session submitted: the breakthrough's trigger), the
`own_message_landed` slot set in
`push_message` (the message id *and* its room, since the won hit is put
on the wire for that room a tick later), the stage-2 wire itself
(`ChatService::publish_name_hit` does the `pg_notify`; chat's message
listener, the one that already carries gild markers, turns the notify
into `ChatEvent::NameHit`; `note_name_hit` hands the beat over through
the `witnessed_hit_landed` slot as soon as the message is on screen,
holding it in `pending_name_hits` until `push_message` lands the message
if a replica's delta is behind), `name_flicker` threaded through the chat
view structs into the rows cache key (the row builder corrupts
whichever message id it is handed, so witnessing costs the chat renderer
nothing), and `ChatService::send_first_contact_invitation_task` (answers
the claim on a oneshot, then sends the DM after a delay). Outside the domain:
`app/ai/screen.rs::screen_bio` (the
bio verdict), `ProfileService`'s first-contact tasks (the row claims),
`late-core`'s `models/deadchannel_name_hit.rs` (the wire's channel,
payload, and parse), and `metrics::record_first_contact_beat` /
`record_first_contact_bio_screen`.

## The four stages (behavior contract)

1. **Clock glitch (deniable).** The sidebar clock (pinned core block,
   Home/Arcade: the most stable, most-glanced-at element) renders one or
   two time characters from the glyph alphabet for ~200ms
   (`GLITCH_HOLD_TICKS`, spanning the sidebar's ~132ms wake cadence),
   then heals. Scheduled per session with independent dice: the first
   burst 5-20 min after connect (so an hour-long evening sees one), the
   next 20-60 min later, at most `GLITCH_DAILY_CAP` (2) per UTC day,
   deferred a few minutes whenever the clock is off screen so a burst is
   never spent unseen. A due burst is a `GlitchTick::Due`: the service
   claims it on the row (`claim_first_contact_glitch_burst`, both caps
   enforced in the `UPDATE ... WHERE`), the machine holds its schedule,
   and the burst starts on the tick the claim comes back won; a capped
   answer re-dices and mirrors the row's count, a failed one defers a
   few minutes. At `GLITCH_TOTAL_CAP` (3) the clock goes quiet for good
   and stage 2 opens (the quiet is part of the escalation). Chrome,
   never content; timezone label untouched. Armed for every staff
   session, gate or no gate.
2. **Name flicker (personal, witnessed).** Only once stage 1 has spent its share
   (glitch hits at the cap): on the landing echo of this session's own
   send (the one moment of guaranteed attention), a 1-in-3 roll may
   corrupt two or three characters of that message's author label for
   ~800ms, then a different two or three for ~800ms more (two waves,
   `NAME_WAVES`, each with its own seed), heavier and longer than the
   clock: name characters only,
   never the body (the escalation is targeting, not content). Only a
   send that renders its own author header is a target: the landing
   hook in `chat/state.rs` skips grouped continuations (a fast
   follow-up to your own message, `MESSAGE_GROUP_WINDOW_SECS`), whose
   label never draws, so a hit is never spent invisibly. A landed roll
   is a `NameRoll::Claim`: the service claims it on the row
   (`claim_first_contact_name_hit`, `NAME_DAILY_CAP` (1) per UTC day and
   `NAME_TOTAL_CAP` (3) ever, both in the `WHERE`), no other send rolls
   while the claim is out, and the label corrupts on the tick the claim
   comes back won. The corruption rides the chat rows cache key, so
   start and heal rebuild rows exactly once. Chosen only.
   **The room watches.** The hit is not private to the
   session that rolled it: a won (or forced) hit goes onto the
   `deadchannel_name_hit` wire and every session in that room replays it
   on the same name. Rolling, capping, and claiming are unchanged and
   still belong to one session; a witness decides nothing, it replays.
   Three properties carry it: the *seed* rides the wire, so every screen
   swaps the same characters in the same two waves (`ActiveHit` is the
   one playback both sides use; the witness's copy sits in
   `HauntState.witness`); the beat *waits for its message*, which only
   matters across replicas (on the sender's own replica the chat
   broadcast lands the message before the claim is even out), so chat
   holds a beat whose message the room delta has not brought yet and
   hands it over as the message lands; and past `NAME_HIT_WAIT` (30s,
   chat's constant) it is **dropped rather than played late**, so
   somebody opening the room a minute afterwards sees a clean name. The
   person being haunted declines their own copy off the wire by
   recognising their live hit (a second device of theirs holds no live
   hit and witnesses it normally), and the audience is exactly stage 1's:
   only staff are in it, so nothing of the haunting reaches a real user.
3. **Whisper (the held door).** Plays `WHISPER_TOTAL_CAP` (2) times per
   person, at least `WHISPER_GAP_HOURS` (20) apart, each from its own
   line pool: the first door says the static noticed you, the second
   that something is trying to get through. Arms at connect only when
   name hits have reached `NAME_TOTAL_CAP` and
   `FirstContactMarks::whisper_due` holds (under the cap, and the last
   delivery a day or more ago): the haunting follows you home, and comes
   back. The splash neither skips nor expires while held,
   and the scene waits on nobody: the static pulses on its
   own rhythm (~1.3s, each noise pattern held ~130ms) from the first
   frame, the voiced line types itself once
   the base splash line is done (`VOICE_TICK`), the skip hint dissolves as
   it starts, and every key, Esc included, is swallowed and does nothing
   (a scene that waits on a keypress reads as a quiet line under the cup,
   and people miss the door). A hard cap (~10s) releases whatever the
   phase. Delivery claims
   one mark (`claim_first_contact_whisper`: increments
   `first_contact_whisper_hits` and stamps `first_contact_whisper_at`,
   conditional on the cap and the gap in the row, so two devices that
   both played leave one mark and the same evening never counts twice;
   the loser is logged, the one race the claim-on-delivery shape
   accepts, because claiming at arming would burn a whisper on every
   dropped SSH session). A hard-cap drop or lost session leaves the
   mark unspent.
4. **Breakthrough, then the invitation (the whole game is opt-in).**
   `INVITE_DELAY_HOURS` (20) after the second delivered whisper the
   breakthrough comes due (`FirstContactMarks::breakthrough_due`), and the
   next send this session submits plays it (the DM
   alone, met cold, reads as spam). Only a `SendSucceeded` for a
   request this session submitted counts, never the same person's send
   from another device: every session of a user hears every send, and an
   idle one would play the scene to nobody. The send asks
   `ChatService::send_first_contact_invitation_task` for the once-ever
   claim; the task answers `Won`, `Taken`, or `Failed` on a oneshot before
   it sends anything, the scene plays only on `Won` (a `Taken` stamps the
   marks; a `Failed` is logged by the task under
   `first_contact_invitation_failed`, and this session stops asking until
   `/haunt invite`), and the task sends the DM after
   `Breakthrough::dm_delay`, the moment the line has typed, whether or not
   the session is still there. The scene is private and full screen: the
   door's static, heavier and pulsing quicker, over the whole frame and
   every modal, `BREAKTHROUGH_LINE` typing in a gap torn out of the middle,
   about seven seconds, every key swallowed ahead of door passthrough and
   the parser. Known gap to close before the haunting leaves staff: the
   claim is stamped `dm_delay` before the DM sends, so a
   replica stopping in that window leaves a stamp with no DM, and nothing
   repairs it but `/haunt reset` by hand. The DM comes from the game's first voice - `afterglow`
   (GAME.md reserved the name for something inside the world), a
   bartender-shaped ghost user (fixed fingerprint `afterglow-fp-000`)
   that is never auto-joined into public rooms - sends one persistent DM.
   The voice row is ensured at process startup
   (`ChatService::ensure_first_contact_voice_task`, main.rs), the same
   reservation move as the `system` user: the first deploy creates the
   row and the case-insensitive unique username index holds the name
   from then on; the invitation task re-runs the ensure, so a lost boot
   or a squat self-heals. The DM is
   a plea ending in the only instruction the haunting ever gives,
   `/join #deadchannel`. Self-serve: the chosen one's own session notices
   the due date; `User::claim_first_contact_invitation` (a conditional
   settings stamp) keeps racing devices to exactly one DM. **Order is
   load-bearing:** the voice user and the DM room are ensured *before*
   the claim is taken (any failure there, say a squatted `afterglow`
   username, leaves the claim untaken so a later session retries; a
   racing loser's `User::create` heals by re-finding the winner's row by
   fingerprint), the claim guards only the send, and a failed send
   releases the claim (`release_first_contact_invitation`; losing the
   release too is logged as a burned claim). The `deadchannel` room slug
   is reserved in `normalize_topic_slug` (late-core `chat_room.rs`,
   beside the `lounge` reservation, case-insensitive, refusal message
   "only static on that channel") so no user-created room can be waiting
   where the invitation points. **The invitation is the key:** `ChatService::open_public_room` routes the
   `deadchannel` slug (case-insensitive) into `join_deadchannel_room`,
   which requires `first_contact_invited_at`; without the stamp the
   caller gets the same static line the reserved slug gives, so from
   outside the door and the wall are indistinguishable. An open door
   would let people skip the bio/settings/tenure eligibility funnel
   that the haunting exists to drive. The room has its own
   `kind='deadchannel'` (migration 170,
   `ChatRoom::get_or_create_deadchannel_room`, seeded on the first
   invited join, never auto-joined): every room listing is a kind
   whitelist (browse lists only `topic`, IRC lists
   lounge/language/topic, and IRC JOIN filters through
   `is_irc_channel_kind`), so the channel is hidden from browse and
   IRC by construction, the same way game rooms already are, without
   inheriting the game-room join path. Once joined it does show on the
   rail: the last room in Core, under `#voice`, above Discover, never in
   Channels (`chat/state.rs::is_deadchannel_room`, read by
   `visual_order_for_rooms` and both rail builders in `chat/ui.rs`).
   Copy and name face design review before real users ever see them.

## `/haunt` (admin composer command)

Parsed in `chat/state.rs::submit_composer` **only when `is_admin`**
(enum + parser live in `haunt/state.rs`); for everyone else, moderators
included, the line posts as plain text, exactly as if the command did
not exist. Admin-only on purpose while the ladder runs for staff: a
moderator who could type `/haunt` would know what the glitches were.
Drained by `haunt::svc::tick`.

- `/haunt` - status: whether stage 1 and the chosen
  stages armed for this session, the gate's three legs (active hours,
  touched settings, bio length and standing), glitch schedule, glitch
  and name hit counters against their caps, the witness (whether a beat
  of somebody else's is on this screen), door, whisper, and the
  breakthrough or invite.
- `/haunt arm` - forces this session chosen and arms the repeatable
  machines, so a beat (and the gate) is testable without reconnecting or
  a passing bio. Session only; nothing is written.
- `/haunt glitch` - fire a clock burst on a ~7s fuse (the banner covers
  the clock for ~5s), bypassing schedule and caps.
- `/haunt name` - force the next own send to flicker. Skips the row's
  caps, not the wire: the forced beat travels like a real one, which is
  how the public half of stage 2 is watched from a second session.
- `/haunt replay` - re-run the splash whisper now, ignoring the marks.
- `/haunt invite` - the next own send breaks through, skipping the delay;
  the DM follows exactly as for a real one.
- `/haunt reset` - wipe every mark; the chain starts over.
- `/haunt welcome` - post the runner's welcome on the wire for this user
  now, exactly what a fresh invited join posts; nothing else changes.
