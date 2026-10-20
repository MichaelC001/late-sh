# AI Context

`late-ssh/src/app/ai/`: the model calls behind chat. The ghost bots (@bot, @graybeard, @bartender) and their mention cooldowns, screening of text a player writes, chat translation, and the `/summary` catch-up. The daily paper also calls `AiService`, from `app/paper` (its own context). Each module opens with a doc comment that carries its rules; this file is the map plus the flows that cross into chat and chips.

## Files

- `svc.rs`: `AiService`, the Gemini client: `generate_reply`/`generate_short_reply`/`generate_ungrounded` for text, `generate_json`/`generate_json_with_search` for schema-enforced JSON. Which path a caller picks (grounded with Google Search or cheap) is the call policy at the top of `ghost.rs`.
- `ghost.rs`: the always-on chat characters (@bot, @graybeard, @bartender), their mention responders, the clubhouse tutorial's scripted @bartender welcome, and the bartender's order and round flow (The Round below). Each bot registers with `fingerprint: None`, so it stays out of the human headcount.
- `ladder.rs`: escalating per-user, per-room mention cooldowns for @bot and @bartender, process-global; the composer reads it to warn at submit time. Graybeard keeps a flat cooldown in his own responder loop.
- `screen.rs`: screening for text a player writes: the Shop's custom title and the profile bio.
- `translate.rs`: `TranslationService` (Translation below).
- `summary.rs`: `SummaryService` (Summary below).

Tests sit beside each file as `<file>_test.rs`; run them with `make test-llm ARGS="-p late-ssh -E 'test(app::ai::)'"`.

## The Round


**A single drink first.** `ChipService::buy_drink` is one transaction: a
floor-guarded `ChipMove::DrinkPurchase` debit (it pours only if the balance
keeps the 100-chip floor; `source_ref` the drink name) and the buyer's
`user_drinks` upsert (`late-core/src/models/drinks.rs`: drunk points decay at
read on wall clock alone, online or not, capped at `MAX_DRUNK_POINTS` 4000;
`DRUNK_SOBER_UP_HOURS` (12) is the one dial). @bartender's model
(`app/ai/ghost.rs`, ungrounded schema JSON) names the drink, the price and the
required `intoxicating` flag: intoxicating drinks cost 100-1000, a
non-intoxicating one exactly `SOFT_DRINK_PRICE` (50); any other price, or one
the patron cannot afford, is refused uncharged, so the debit always matches
the quoted line. A non-intoxicating pour, paid or comped, moves only the count
and the spend, never the buzz or its decay clock. Drunk levels print a
`(word)` label on chat author headers from level 1 and slur public-room text
(`../chat/CONTEXT.md` §14 Drunk Text).

One patron buys everyone at the bar a drink. The chips are burned like the
crown's; what the room gets is a #lounge line and a free drink each.
The same credit also serves a one-person gift: `@bartender buy @user a drink`
or another entry on the closed `GIFT_PHRASES` list (a verb, the name and what
is bought: "get @user a beer", "send @user a round", "pour @user one"; what is
bought for whom: "drink for @user", "one for @user"; or whose is on whom:
"@user's next one is on me"), read on the
round's rules (any case, opening a clause anywhere in the message, not a
`?` sentence, not in backticks; see "The trigger is a literal phrase" below) by
`drink_round::gift_drink_target`, which also refuses a message naming any
second handle besides the bartender's, so the recipient is never a guess. A
message places one order at most: `drink_round::bar_order` reads both lists
once for the bartender, and a gift and a round in the same message ring up
neither and go to the model as talk. The handle is resolved against the
account username, trailing dots dropped as sentence punctuation (a username
never ends in one, `sanitize_username_input`).
It costs 200 chips (`GIFT_DRINK_PRICE`, twice a round's head, since the one
person it is aimed at will drink it), pours the recipient the same 400 points
a round does, works for offline humans, shares the 24h expiry and
three-open-credit cap, and does not pour the buyer a drink. The gift and
its floor-guarded `drink_gift` debit (`ChipMove::DrinkGift`, its own reason
so the profile ledger reads "bought a drink for @user" off the round's
single credit and the round dashboards never count it) commit together
through `ChipService::buy_drink_for`; self-gifts, bot targets, unknown
names, full tabs and short balances are refused uncharged. A successful
gift gets a scripted receipt ahead of the bartender's mention ladder and
AI, while refusals step the ladder. `metrics::record_gift_drink_bought`
counts settled gifts and their burn, `record_gift_drink_refused` labels
`late_ssh_gift_drinks_refused_total` by the closed `GiftDrinkRefusal`, and
the pour itself lands under the shared `round_drinks_cashed_total`.
Redemption uses the same oldest-expiring credit path and names the buyer of
the credit actually spent.

**What each purchase costs and pours.** Points follow the bar that sold the
credit (`drink_rounds.bar`, `Bar::drink_points`), not where it is cashed: a
credit is good at either bar and is worth what its seller wrote.

| Purchase | Buyer pays | Each drinker gets | Buyer drinks too? | Ledger reason |
|---|---|---|---|---|
| Tavern round ("round for everyone") | `ROUND_PRICE_PER_PATRON` (100) a head | `ROUND_DRINK_POINTS` (400) | yes, 400 | `round_purchase`, bar `tavern` |
| Gift ("@bartender buy @user a drink") | `GIFT_DRINK_PRICE` (200) | 400 | no | `drink_gift`, bar `tavern` |
| Nightcap round (`r` on the stools) | `ROUND_PRICE_PER_PATRON` (100) a stool | 100 | yes, 100 | `round_purchase`, bar `nightcap` |

Every drink taken off a round, the buyer's own included, is logged in
`drink_pours` like any other pour (`UserDrinks::record_pour`), under the bar
that poured it, not the bar that sold it; buying a round or a gift counts
nothing for the buyer beyond their own drink.

`late-core/src/models/drink_round.rs` owns both tables (migrations 164 and
168), the price, the cap, and the phrase list; `GhostService::bartender_round`
(`app/ai/ghost.rs`) owns the transaction's caller, the refusals, the
telemetry, and what @bartender says; `ChipService::buy_round` owns the money.
It touches chat twice, which is why it is documented here: the phrase gate
reads a chat message, and `chat/slur.rs` has to leave that phrase alone.

- **The trigger is a literal phrase.** `ROUND_PHRASES` ("round for everyone",
  "drinks for the house", "everyone a drink", "drinks on me", "round's on
  me", ...) matched case-insensitively on word boundaries, so "turn around
  for all of us" is not an order. Every entry names the whole house or says
  "on me"; a bare "a round" is not on the list. No model
  decides this: it is the only bartender action that spends more than one
  drink's worth, the price is the size of the room, so the phrase is the
  confirmation. An order is a statement: `contains_round_request` rejects a
  phrase whose sentence runs on to a `?` and never looks inside backticks, so
  "how much is a round for everyone?" is a question the model answers, not a
  bill. The `?` scan reads the whole message and skips code spans, so
  "round for everyone in `#lounge`?" is a question too. An order also has to
  open its clause (`is_order`, shared with the gift): between the last
  `, . ! ? ; :` or line break and the phrase there may only be the
  bartender's `@name` and entries from the closed `LEAD_INS` list: small
  talk ("a", "another", "please", "the next"), saying you are about to order
  ("I'll", "I'd like to", "let's", "can I"), and the verb when the phrase
  starts at what is bought ("buy", "get", "do", "put"). Nothing negative,
  past or conditional is on it. "I'll buy a round for everyone", "let's do a
  round for the house" and "don't worry, round on me" order; "no round for
  everyone tonight", "he said round on me", "we had drinks for everyone" and
  "@alice drinks on me" spend nothing and the model gives the words to say.
  (`round_phrase_spans`, the slur guard's view, still protects the words
  wherever they appear.) It also means a round costs no model call.
- **A settled round answers ahead of the mention ladder**, in both the event
  loop's pre-filter and the reply, because throttling a paid action would
  swallow a purchase in silence. A refusal is free, so it steps the ladder
  like any other answer and is dropped when throttled: repeating the phrase
  into an empty house costs the room one @bartender line per ladder window.
- **`slur.rs` never scrambles it.** Drunk text is stored, not rendered
  (`../chat/CONTEXT.md` §14 Drunk Text), so a wasted patron's order would otherwise reach the
  matcher as "ronud for eevryone" and the feature would break for exactly the
  people most likely to use it. `slur_segment` passes any token overlapping a
  `spending_phrase_spans` range (the round phrases and the gift phrases,
  each widened back over the lead-in words it opens its clause with, so a
  scrambled "please" cannot stop an order being one) through untouched, and
  `with_hiccup` will not drop a `*hic*` inside one; a hiccup right before the
  lead-in is itself on `LEAD_INS`.
  The guard and the matchers read the same lists in `drink_round.rs`, so they
  cannot drift apart. The words around the order
  still take their beating.
- **Price** 100 (`ROUND_PRICE_PER_PATRON`) for every credit that actually
  landed, never for the heads counted: the grant's own `RETURNING` is what
  the charge is computed from, in the same transaction, so two rounds racing
  cannot both bill for the same patron. `ChipMove::RoundPurchase`,
  floor-guarded, burned whole, out of Top Chips like the crown.
- **Presence is `state::online_human_ids_excluding`**, the in-process
  `active_users` roster minus the bots and the buyer. Single-replica by
  choice: there is no presence table (`users.last_seen` is written at connect
  only), building one was more work than the feature, and the round joins the
  multi-replica debt in the `SCALE.md` Pain Point 2 ledger; the credits it grants are DB
  rows and cash from anywhere. Excluding the buyer is what makes "nobody to buy for" a real
  refusal rather than a round bought for one.
- **Only the buyer is poured into**, on the spot, `ROUND_DRINK_POINTS` in the
  purchase transaction: they typed the order. That number is the tavern's,
  a premium over the price a head because this round buys for everyone
  online and most of them never walk up; the Nightcap's round is tagged
  `bar='nightcap'` and pours 1:1 instead (`Bar::drink_points`,
  `../clubhouse/nightcap/CONTEXT.md` §5). Everyone else gets a
  `drink_credits` row, not a drink, because a pour makes someone type drunk in
  public and they did not ask. It is cashed only by ordering from @bartender,
  24h to claim.
- **Credits stack, up to `MAX_OPEN_CREDITS` (3).** The cap lives in the
  grant (migration 168; there is no one-open-credit index), which counts a
  patron's open, unexpired credits and skips anyone at it: a round into a
  room that will not drink still reaches nobody and is still free, so the
  throttle is a number that can be tuned rather than a schema fact. A
  schema-level one-credit limit would do two jobs at once, keeping a patron
  from holding two drinks, which is not the point, and throttling the
  mechanic, and a patron heads-down through three rounds would end the night
  owed one drink while two buyers bought nothing. Expired credits are
  neither counted nor re-used, just rows nobody can drink.
- **Every grant takes an advisory lock** (`ROUND_GRANT_LOCK`) and holds it to
  commit. The cap is a read-then-write over rows a concurrent round is
  inserting and cannot see, so without it two rounds landing together would
  both read a patron at two credits and both grant a third. Rounds are rare,
  so serializing them against each other is the cheap fix, and it also pins
  the order the patrons' rows are locked in, which two overlapping rounds
  working off differently ordered presence reads could otherwise deadlock on.
- **A pour spends the credit closest to expiring**, picked `FOR UPDATE SKIP
  LOCKED`, so a banked drink is never lost to the clock while a fresher one
  sits behind it and two orders landing together take two different credits
  rather than fighting over one (two orders are two drinks). The same
  statement counts what is left, excluding the row it just cashed by id,
  because a data-modifying CTE's write is not visible to the rest of its own
  statement.
- **A cashed drink pours a flat 400 points** whatever the bartender named it:
  four times what the buyer paid. At exactly the buzzed threshold (300) the
  first decay tick (334 an hour) would drop the drinker back to tipsy seconds
  after the pour; 400 buys about eighteen minutes of the level
  the round is meant to hand out. `lifetime_spent` does not move.
  The open credit is read before the prompt is built and becomes the
  `BartenderTab` handed to `parse_bartender_order`: on a comped tab a "pour"
  (or an "offer", the model deciding they could not afford it) becomes
  `PourComped` and skips both price gates, since nothing is debited; that is
  what lets a patron sitting on the chip floor, or a model that obeyed "do
  not quote a price", still drink what was bought for them. If the credit is
  gone by the time the pour lands, nothing is poured or charged and a
  scripted line says so.
- **Scripted lines, not generated ones.** The announcement and all three
  refusals are consts in `ghost.rs`. This is the one line that has to quote
  the number the patron was charged, and it has to land the moment the chips
  move rather than after a model round trip that could time out or invent a
  different figure. The prompt hands out the phrase when asked but never
  claims a round happened. The same rule covers who bought the drink and what
  a patron has left: the pour picks the credit closest to expiring at pour
  time, which with a stacked tab is not always the one that was open when the
  prompt was built, so `ROUND_CREDIT_ON_LINES` names the buyer of the credit
  actually spent and, when the pour leaves credits behind it,
  `ROUND_CREDIT_REMAINING_LINES` follows with the count, both appended to the
  model's line in code. The prompt is told to guess at neither, because a
  name or a number in that line is a promise the bar has to keep.
- **Feed.** `ActivityKind::RoundBought` posts a ticker line and no headline:
  @bartender already says it in the room, and everyone it reached is online
  by definition, so a #lounge row would be the third telling of one drink.

## Translation


Chat messages translate on demand (`t`) or, opt-in, automatically. The model call lives in `app/ai/translate.rs` (`TranslationService`, a Gemini `generate_json` schema call); `ChatState` owns only per-session display state.

- **Cost scales with messages written, not readers.** Every result is cached in `message_translations` keyed `(message_id, target_lang)`, so the first viewer's call covers everyone after them, forever, including a session reconnecting tomorrow. Two guards keep that true: single-flight dedupe in the service (twelve sessions rendering the same new message make one call) and a bulk cache-only lookup when entering a room.
- **Same-language is a cached verdict, not an absence.** The model reply carries a `same_language` flag (an echoed body counts too, the guard for URL-only messages); the verdict lands in `message_translations.same_language` so nobody pays for the call again, renders as nothing (no `↳` line), and `t` on it banners "Already written in X". This is what lets English translate: `En` claims no script (`TranslateLang::script`), so French/Spanish/German bodies reach the model for English readers like any shared-script target, and the model, not the script check, decides what was already readable. English must not claim Latin: that would silently refuse every Latin-script language for the English majority.
- **The script precheck survives as the cheap first filter.** `needs_translation` (`late-core/src/models/message_translation.rs`) still clears unambiguous cases locally (a Han body for a Han target, unscripted bodies, over-cap walls) so they never reach the API. It is a *script* detector, not a language detector; targets without their own script (English and the Latin roster; Russian/Ukrainian on Cyrillic) send every scripted body to the model and let the cache absorb the cost.
- **Replies translate the reply, not the quote.** The composer bakes `> @author: preview` into the stored body, and `translation_source_text` strips that first line everywhere translation looks (the precheck, the model prompt, the cached text), so the quoted author never gets re-worded and the `↳` line carries only the reply. The staleness guard still compares the full stored body, since that is what an edit rewrites.
- **Authors can pre-share in English, and shared means shown.** `Ctrl+O` → Settings → Translation → "Translate my messages to English" (`users.settings.translate_mine_to_en`, off by default): the send and edit paths (`ChatService::pretranslate_for_author`) fire `TranslationService::request_shared` for the author's own message. The cache row lands with `author_shared = true`, the broadcast event carries the flag, and every session whose target language matches displays the `↳` line with no auto mode and no `t`, live via the event and later via the room-entry sweep. The author's own session never shows it (they wrote the original), and a reader's private `t` rows stay private: display is the author's choice, never a side effect of someone else reading. One call per message written by the opted-in author, spent from the same daily cap.
- **Auto mode is live-only, by policy.** A foreign-script message arriving in the room *on screen* auto-expands; history does not, which is what bounds auto mode's cost and matches what the feature is for (following a live conversation, not machine-translating the archive). History is always one `t` away. Cached history *is* shown pre-expanded on room entry, since reading the cache is free.
- **The render rule is one line:** foreign script + cache hit → show expanded; cache miss + live → fire a call; cache miss + history → collapsed, `t` on offer. The room-entry cache sweep runs for **every** session (it is cache-only, so it costs one batched read and zero API calls); the drain decides display: auto mode shows all hits, everyone else shows author-shared rows only. A `t`-collapse is a per-session override (`translation_hidden`) that wins over auto mode, the cache, and author-shared rows, so a message you dismissed does not spring back open next frame.
- **Invalidation is mandatory, not hygiene.** A cached translation describes the exact body it translated: `ChatService::edit_message` deletes the message's rows inside the edit transaction, and `ChatState::forget_translation` drops the session's copy on edit and delete. Skipping either leaves a translation asserting something the author no longer said. Changing the target language clears every stored translation for the same reason, and late results for the old target are dropped on arrival. The cache write itself is conditional (`upsert_if_current` checks the live `chat_messages.body` against the body that was translated), which closes the race where an edit lands while the model call is in flight: the edit's row delete finds nothing to delete, and the stale result is then discarded instead of cached.
- **Guardrails are for bugs and abuse, not for legitimate traffic**, which is orders of magnitude below them: a global daily call cap (`TRANSLATE_DAILY_CAP`, degrading to "translation unavailable" until UTC rollover), a 4-way concurrency gate so a burst queues instead of tripping API rate limits, and a body-length cap. Failures never retry on their own; `t` is the retry. `record_chat_translation` reports every outcome (`cache_hit` / `translated` / `same_language` / `failed` / `cap_exhausted` / `stale`), so the cache hit ratio and the cap are both visible.
- **DMs and private rooms are included.** The *viewer* opted in and it is their received text, unlike chat's Drunk Text (`../chat/CONTEXT.md` §14), which is excluded from private rooms because it rewrites what the *author* said.
- Target language, auto mode, and the author-side English share are per account (`users.settings`: `translate_to`, `auto_translate`, `translate_mine_to_en`), edited under `Ctrl+O` → Settings → Translation; the first two sync into `ChatState` each tick, the third is read by `ChatService` at send/edit time. The settings row reads "Target language" because `translate_to` decides two things: what `t` and auto mode translate into, and which authors' shared translations this session receives (a shared English row shows only to English-target readers).

## Summary


`/summary` is the AI room catch-up: one call summarizing the visible public room, either since the reader last left the app on this device or over a window they typed (`/summary 6h`). `app/ai/summary.rs` owns the whole pipeline (`SummaryService`, translate.rs-shaped); `ChatState` only parses the command, hands over the device mark (`parse_summary_arg` + `catch_up_window`), and drains `SummaryEvent`s in tick: ready summaries open the shared `Overlay`, waiting in `pending_summary_overlay` when another overlay is already up (a banner says so, and the summary takes the surface as soon as it frees); everything else banners.

- **Window**: the command layer parses the argument into a closed `SummaryArg` (`CatchUp`, `Window`, `Unparseable`, `TooShort`, `TooLong`, each with its own banner) and resolves `CatchUp` through `catch_up_window(device_left_at)` into a `SummaryWindow` arm. The mark is a fact about this terminal that lives only in the session (and on the device key), so the session is what supplies it, and the service keeps no cursor of its own. `summary::window_start` resolves every arm and reports a closed `SummaryBasis` plus a `capped` flag alongside the start, so the overlay head can say which sentence it is telling.
  - `SinceLeftApp(at)` (the device has a mark) → exactly that instant; basis `LeftApp`, head "since you left · stamp". **Nothing widens it.** A mark ten minutes old means a ten-minute catch-up, which is the entire point of resting on when the reader left.
  - `Default` (no mark: keyless, never ended a session here, or the write was lost) → `SUMMARY_DEFAULT_WINDOW_HOURS` (24h), basis `Default`, head "since stamp · the last 24h", with no claim about absence either way. The only window that is handed out rather than derived.
  - `Explicit(duration)` → what the user typed, bounded only by the 48h max; the mark is ignored outright.
  - A mark older than `SUMMARY_MAX_WINDOW_HOURS` is clamped to it and the head appends "· capped at 48h": the stamp is then the cap, not the moment they left, and the head says which.
- **The 24h default is never a blanket clamp.** A clamp only makes sense over a cursor that cannot know, and `chat_room_members.last_read_at` is that cursor: it records presence rather than reading, so a terminal parked on a visible room marks each arriving message read and a marker-only window answers "nothing new" to exactly the person who missed the day. The window is derived from the device mark instead of being padded.
- **Nothing is written on delivery.** The mark is the reader's, not the summary's; a second `/summary` reads the same stretch again (see `../chat/MARKS.md`). A failed summary therefore costs nothing but the request.
- **The typed window requires its unit** (`6h`, `90m`; case-insensitive): a bare `6` is refused rather than guessed at, and a window past 48h is refused rather than silently clamped, so the answer is never narrower than the question. Both refusals land as banners before any request goes out.
- **Bounded two ways** before the model sees anything: wall clock (48h) and transcript size (`SUMMARY_PROMPT_CHAR_BUDGET`, 200k chars, newest kept, oldest dropped). The SQL fetch limit is derived from the char budget (`SUMMARY_FETCH_LIMIT`, budget / minimum line size), so it bounds memory without being a third policy knob. The overlay header says when the caps cut messages.
- **`Empty` carries its basis**, because "nothing new since you left" and "nothing said in the last 24h" are different claims, and the banner tells them apart.
- **The head line is dated on the reader's clock.** The overlay opens with `3 messages since Aug 28 16:30 CEST`, formatted by `common/time.rs::instant_for_viewer` from `ChatState.viewer_tz`, a mirror of the account timezone synced by `App::tick` (and at bootstrap) the same way the translate settings are. No account zone, or an unparseable one, falls back to UTC (`Aug 28 14:30 UTC`): the zone is always named, so the window can never be read against the wrong clock.
- **Public rooms only**, enforced in the SQL (`ChatMessage::list_public_room_since`: membership required, `visibility = 'public'`, system lines and ignored users excluded) with a fast client-side banner first. Private rooms and DMs never reach the summarizer.
- **Guardrails**: a per-user-per-room slot, reserved under one lock before any work (a duplicate submit while a request runs collapses into it instead of spending a second fetch, cap slot, and model call) and armed as the cooldown on success only (`SUMMARY_COOLDOWN`, 10 min; failures release the slot so `/summary` is its own retry); a global daily cap (`SUMMARY_DAILY_CAP`, tripping logs the requesting user); a 2-way concurrency gate; and `record_chat_summary` telemetry per outcome. Results are per viewer (everyone's cursor differs), so unlike translation there is no shared cache; the slot is what absorbs repeats.
- The system prompt marks the transcript as untrusted chat content: instructions inside messages are reported, not followed.

The daily paper is the other AI catch-up and deliberately not this one: a fixed window printed once per room for everyone, not a per-viewer window from a device mark. It lives in `app/paper` (see its CONTEXT.md); chat only parses `/paper` and lends the rail order and membership to its layout.
