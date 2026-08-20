# The Thundersmith, a class design

Status: design only, not implemented. Drafted 2026-08-06; revised 2026-08-20 after
a code-verified balance pass. Every engine claim below was re-checked against
source; the benchmark class is now the Ranger (not the Rogue), measured at the
Lv30-60 band where the game is actually played, not at L100.

*A bulky master smith with a storm-cell scattergun. He is not stronger than you.
He is prepared, and that's worse.*

---

## 1. Identity

Iron Man by way of a dwarven forge. Gloamwright glasscraft (the glass-and-obsidian
artificers of Kaelmyr's black deserts) fused with Stormheld storm-spire tech; both
peoples already exist in Kaelmyr's lore, so the craft has an in-world origin.

Doctrine: efficiency and smartness. He wins fights before they start, at the forge
and in his field notes, not in the exchange of blows. Lightning is his brand; the
crack and ozone of storm-cells is the sound of the class.

**The contract.** The class is the widest gap in the game between prepared and
unprepared. Fueled and informed he is the strongest class, full stop: +15-20%
over the Ranger, the current verified #1. Dry he is a tanky, honest, bottom-third
brawler. The spread is the identity, and the cost of the top end recurs per fight;
it is never a one-time unlock. If preparation can be paid once and forgotten, the
design has failed.

Not locked. The class is available at class select like any other. Access is free,
power is gated: being good costs Smithing ~50, materials, and a survived recon
fight per zone. An unlock gate on top adds annoyance, not depth.

## 2. Frame

| field | proposal | note |
|---|---|---|
| Resource | **Charge** (new `Resource` variant) | one enum arm + label |
| Primary score | Intelligence | smartness doctrine |
| max_hp | `44 + l*10` | bulky: between Berserker/Valewalker (42+10) and Paladin (46+11) |
| attack | `5 + l*2` | caster tier; the gun's damage comes from cells (multiplicative, see §3) |
| max_resource | `60 + l*4` | |
| resource_regen | `8` | mid: below Rogue 12/Monk 11, above Mage 7 |

Dry, this frame reads as a Warrior's bulk with a caster's damage: sturdy,
unremarkable, honest. Everything above mid-tier flows through cells.

## 3. The gun

His auto-attack is a scattergun shot whose damage school and power are decided by
the loaded cell. This is the genuinely new mechanic, verified against the engine:
every other auto-attack in the game is hardcoded Physical (the combat round calls
`profile.apply(atk, DamageType::Physical)`; the pet bite too), so he is the only
class whose basic attack can bypass a physical resist or land on a weakness.

**Cells are multipliers on `attack()`, never flat adders.** Verified: ability
magnitudes are flat table constants with no level, gear, or score term, which is
why every class's kit decays into irrelevance as gear scales (endgame gear alone
is ~+523 attack). A flat "+N per shot" cell would decay on the same curve and the
whole crafting gate would buy a rounding error by the Frontier. A multiplier rides
the gear curve instead, so tier-5 cells matter as much at the top as in the band.

| ammo | school | multiplier | supply |
|---|---|---|---|
| Scrap shot | Physical | **x0.90** | unlimited, free; the never-empty floor |
| Storm-cells t1-t5 (signature) | Lightning | x1.10 / x1.15 / x1.20 / x1.27 / **x1.35** | crafted |
| Counter-cells (ember / frost / holy / ...) | one line per school | one notch below the same tier's storm | crafted |

- Cells are consumed per shot. Combat runs one auto per 2s tick, so a 20-shot
  cell is ~40 seconds of fighting; cells therefore craft in **bandoliers**
  (batches), and cost is tuned as a rate, not a price (see §9).
- Tier-5 cells require Smithing ~50 and masterwork-grade materials. This is the
  power ceiling and the crafting system's endgame consumer.
- Scrap keeps the dry state playable but honest, and it stings twice: x0.90 on
  the multiplier, and Physical is the worst-expectation school in the game.
  Corrected census (116 authored profiles): Physical is 4th by resist count
  (Frost 23, Shadow 19, Fire 18, Physical 15) but **nothing in the world is weak
  to it** (0 of 116), giving it the near-worst expected multiplier (0.934).
  Notably, every Aelunor boss resists Physical: an entire continent already
  punishes the unprepared shot.
- Why lightning as the brand holds up: resisted by 2 profiles, a weakness on 9,
  expected multiplier 1.031, behind only Holy (1.140) and Fire (1.057). Nothing
  resists Arcane (0 of 116), but nothing meaningfully seeks it either (weak on 3).

## 4. The loop (recon fights)

No free information. A mob's resist/weak profile is never shown up front (true for
everyone; the engine only reveals it via the post-hit `defense_tag` log line).
The Thundersmith's edge is that he can *act* on what he learns.

1. **Probe.** Each school fired reveals its result (weak / resisted / neutral),
   reusing the existing tag machinery. Every probe round is a real combat round;
   recon costs HP and shells. Sharp players binary-search schools; sloppy ones
   empty a bandolier.
2. **Fighting retreat.** Signature utility ability: a concussion blast that stuns
   and withdraws cleanly (unlike flee's uncontrolled first-exit). Probe damage
   persists on the mob (shared-world HP), so the return pass faces a dented foe.
3. **Re-arm.** Swapping loaded cells from the carried bandolier is an
   out-of-combat action, doable at the boss door. Crafting *new* cells requires a
   craft station (forge in Embergate), so expeditions are provisioned in advance.
4. **Execute.** Return, auto-chamber (below), shred.

The loop self-balances: trash dies to anything, so the recon dance only activates
on fights that matter.

## 5. The Ledger

Field notes, **keyed by zone, not by species.** Verified data model: resist/weak
is per-zone in practice. Every generated region's regulars carry no profile at
all (Frontier, Reaches, Kaelmyr, archipelago, lakes, Broceliande, Aelunor's 100
creatures), the three dungeons and the three Wildbound biomes carry exactly one
profile per zone, and only ~116 authored spawns vary individually. A per-species
ledger over 426 regular foes would be almost entirely duplicate rows (the same
clutter wall that got per-species titles removed); a zone-keyed ledger makes the
first probe in a place a real "you've read this land" moment and shrinks the
schema bump to a small persisted set of zone keys.

**Auto-chamber (QoL):** on engage, if the zone is in the ledger and the
counter-school is carried, it loads itself with one log line ("You know this
plating. Ember rounds chamber with a click."). Priority: known weakness if
carried > storm (affinity) > whatever is loaded > scrap. Zero keypresses;
knowledge does the work.

## 6. Traits and systems

- **Scrapwright** (the passive class trait): mob kills refund one shell of the
  loaded cell type. A short trash fight spends 4-6 shells, so this is roughly a
  20% rebate on clean play. It fires on mob kills only, never in pvp (see §8).
- **Storm affinity:** the storm line runs one multiplier notch above every
  counter-cell line. Keeps storm the default answer and the brand.
- **Doorway advantage:** the foe's opening strike is denied on engage. Priced
  deliberately from verified mob damage: one denied hit is worth 50-95 damage
  per pull in the Lv30-60 band (overworld mobs land 25-75 per hit, x1.25 after
  dark) and 110-290 in the Frontier and beyond. That is roughly one tick of
  effective HP on *every single pull*, strictly better than Opportunist in a
  grind. It is the defensive half of the kit's budget alongside the bulky frame;
  if the class lands hot, this is the first lever to pull, not the cells.
- **Capacitor plating:** while charged, a small shield trickle between hits
  (existing `shield` field, no new machinery).
- **Overclock:** spend Charge to empower the next few rounds. Note: empower
  feeds `attack()` and rides the cell multiplier, which is correct and intended;
  the interaction is priced into the +15-20% ceiling.
- **Capstone (L100):** a rail-lance that dumps the entire remaining battery into
  one discharge, damage scaling with shells left.

## 7. Archetypes (level 10, two paths)

The engine forces the choice (`archetype_choices` gates the screen at
`ARCHETYPE_LEVEL`), so the class needs exactly two:

- `dps("siegesmith", "Siegesmith")`: "Every shot a breach. Your cells discharge
  far harder." (standard dps template: +18% attack)
- `tank("aegiswright", "Aegiswright")`: "Plate the rig and hold the doorway;
  what lands is turned aside." (standard tank template: +22% mitigation, +12% max HP)

The cell multiplier stacks multiplicatively with the archetype percent, same as
gear does. Priced into §9's targets.

## 8. PvP: the armor-breaker

Verified engine fact: `strike_player` reduces incoming damage by `armor/2`
against Physical but only `armor/4` against every other school. A chambered cell
therefore halves the armor term against plate-stacking duelists.

This is embraced, not patched: **the Thundersmith is the designated counter-pick
to armor.** Tank archetypes and masterwork-plate stackers are his prey. The
pricing that keeps it a matchup rather than dominance:

- Cells burn in pvp with **no Scrapwright rebate** (the refund is mob-kill only),
  so dueling is the most expensive thing he does. Winning a duel on tier-5 cells
  should feel like spending real money to make a point.
- Doorway advantage does not fire in pvp; players are not mobs to be ambushed at
  a threshold.
- No pvp-only bonus anywhere. His whole edge is the armor formula itself, which
  means glass casters (who never stacked armor; their `armor/4` was already
  nothing) fight him even or better. Rock-paper-scissors, not a throne.

## 9. Benchmark: the Ranger (the bar to clear)

The Ranger is the verified #1, not the Rogue. Hunter's Instinct is the only trait
that multiplies both auto-attacks *and* abilities (+25% below half health, ~x1.125
averaged over a fight), on the top martial attack tier (`6 + l*2`), with regen 9
and a dps/tank archetype choice. The Rogue's Opportunist doubles one auto per
engage: ~+33% on a three-tick trash pull, ~3% on a boss. Rogue wins short fights,
Ranger wins everything else.

Measured at the band (Lv45, realistic gear ~+100 attack, mid pet ~76/tick;
sustained damage per tick over 30 ticks, dps archetype, sim verified against the
combat round's actual hooks). Broceliande drops Frontier-tier gear (its loot
borrows Frontier tiers 0-9), so this gear level is genuinely reachable in-band:

| build | dmg/tick | note |
|---|---|---|
| Ranger + pet (the bar) | 382 | |
| Rogue + pet | 365 | |
| Mage + pet | 344 | |
| bottom three + pet (Paladin/Cleric/Druid) | 272-306 | |

A maxed pet is ~247/tick at cap (Aurora Worldserpent, pet level 10, all
auto-skills; recomputed from `pets.rs`/`taming.rs`, replacing the earlier 281
estimate), and any class can hold one, so the pet cancels out of every
comparison.

### Thundersmith targets (Lv45 anchor)

| state | target dmg/tick | vs Ranger |
|---|---|---|
| Tier-5, known weakness chambered | **440-460** | **+15-20%** |
| Tier-5, neutral foe | 415-430 | +9-12% |
| Tier 2-3 cells (grind fuel) | 355-375 | Rogue/Mage tier |
| Scrap shot (dry) | 280-300 | bottom three |

The spread is the design contract: unambiguous #1 while fueled and informed,
Druid's neighborhood while dry. Re-verify the same table at Lv55 before shipping;
the multipliers are level-independent by construction, so drift there means a
frame or roster bug, not a cell bug. If the fueled number lands above +25%,
tighten the tier-5 multiplier or shots-per-cell, not the frame.

## 10. Costs, as rates

Every cost is a recurring rate, never a one-time gate. Smithing ~50 is the
*access* grind; cells are the *power* bill.

| gate | what it costs |
|---|---|
| Smithing ~50 + masterwork materials | access to tier-5 cells, the ceiling |
| Tier 2-3 upkeep | target ~10-15 min of gathering/smithing per hour of fueled combat |
| Tier-5 upkeep | steep by design; rationed for bosses, not for grinding |
| The recon fight, per zone | survive learning each land the hard way |
| PvP | pure burn, no Scrapwright rebate |

Scrapwright turns skill into margin: clean trash play runs a ~20% shell rebate,
sloppy play pays list price. Waste is the tax on ignorance, efficiency the
dividend on knowledge.

## 11. Engine cost map

Cheap where it counts, reusing existing patterns (all verified present):

- Loaded ammo: the `weapon_poison` transient pattern (`Some((school, mult_pct, shots)`-shaped),
  no save state for the chambered cell; bandolier contents are inventory items.
- Cell application: the two auto-attack call sites (the combat round's mob strike
  and the pvp strike) swap `DamageType::Physical` for the chambered school and
  scale `attack()` by the cell's percent. Two call sites, one helper.
- Probe reveal: existing `Defense`/`defense_tag` machinery.
- Auto-chamber: a hook in `engage`.
- Doorway advantage: skip the mob's first strike after engage (flag like
  `opening_strike`), pve only.
- Capacitor plating: existing `shield` field, topped in the upkeep loop.
- The Ledger: one persisted set of zone keys (the only schema bump).
- Standard new-class wiring: an arm in every `match self` in `classes.rs`
  (name / primary_score / resource / tagline / description / trait_name /
  trait_desc / stats_at / as_key / from_key), entry in `ALL`, a 10-ability
  roster in `abilities.rs` (ids 2200+), the two archetypes, trait hooks in
  `svc.rs` (engage, combat round, kill_mob for Scrapwright, strike_player for
  plating), cell items + recipes in `items.rs`/`crafting.rs`.

## 12. Open tuning knobs

- Shots per cell (~20) x cells per bandolier craft (~5): together they set the
  minutes-of-prep-per-fueled-hour rate, the single most important dial.
- The tier multiplier ladder (x1.10 to x1.35 proposed) and the counter-cell
  notch-below rule.
- Doorway advantage: one denied strike, or two vs non-boss. First nerf lever.
- Scrapwright refund rate (1 shell per mob kill proposed, ~20% rebate).
- Whether probe results are account-wide or per character (per character
  proposed; the ledger is the character's story).

## 13. Handoff brief: the world resist/weak pass

A companion change, run as its own piece of work (own session, own review). This
design works without it (the cell economy carries the class) and gets better
with it. Everything below is code-verified as of 2026-08-20.

### Why

The recon fantasy is only as rich as the world's profile data, and today most of
the Lv30-60 band has nothing to learn: every generated region's regulars are
`(None, None)` (Frontier, Reaches, Kaelmyr, archipelago, lakes, Broceliande,
Aelunor's 100 creatures). Only ~116 authored spawns vary. Meanwhile resist/weak
already applies to all ability damage, so a placement pass hands *every* class a
school game, not just the Thundersmith.

### Locked decision: resist/weak is per zone

One profile per zone for regulars, derived from the zone's theme (the Frontier's
20 zones already carry strong themes: the Ashen Wastes resist Fire and fear
Frost, and so on). Bosses and marked elites may deviate from their zone.

Rationale: this matches how the world data already works (the three dungeons and
three Wildbound biomes are exactly one profile per zone, and it plays fine), it
aligns with the Thundersmith's zone-keyed Ledger, and it keeps the whole pass
tunable as a small table instead of 400+ per-species rows.

### Verified facts the brief inherits

- **Everyone is an auto-attacker.** At band gear (Lv45, ~+100 attack), every
  class including the Mage is ~75% auto damage; autos ride `attack()` and gear,
  ability magnitudes are flat constants. "Melee vs caster" here is trait and
  roster, not the basic attack. Nobody's school lever rides the dominant source.
- **Flat riders decay by construction.** The existing weapon poison is
  `POISON_PER_TICK = [4, 8, 14, 22, 34]`: flat, capped, ~10% of output at Lv45,
  ~5% at endgame. Anything built on this pattern cannot compound with gear.
- **Current school census (116 authored profiles):** expected multipliers Holy
  1.140, Fire 1.057, Lightning 1.031, Arcane 1.013, Frost 0.974, Physical 0.934,
  Shadow 0.921. Nothing resists Arcane. Nothing is weak to Physical. Holy has
  zero resists anywhere.
- **Resist halves, weak adds 50%, one of each per mob, minimum damage 1.** The
  engine multipliers are fixed and stay fixed.

### Design rules

1. **Weak-forward on regulars.** Every zone gets a theme weakness; weaknesses
   reward the right answer and are fun. Resists on regulars are rare, roughly a
   third of zones at most; walls are only fun when they are events.
2. **No Physical resist on regular mobs, ever.** Zone-wide Physical resist is a
   50% tax on the seven Physical-locked classes with zero counterplay. Physical
   resist lives on bosses and marked elites only, where it reads as "bring a
   caster, an oil, or the smith" (Aelunor's bosses already do this and it plays
   fine).
3. **Nothing is ever weak to Physical.** Preserves the current 0/116 reality and
   the Thundersmith's scrap-floor economics.
4. **Holy gets predators.** Add Holy-resist zones (demonic, hallowed-corrupt
   themes) or Cleric/Paladin quietly become the school winners. Counterpoint to
   weigh deliberately: those are two of the weakest classes today, so a couple
   of Holy-weak lanes could be an intentional buff. Decide on purpose, not by
   accident.
5. **Oils are the martial lever, and they are flat riders only.** Elemental oils
   (fire, frost, holy, ...) via Alchemy, literally the `weapon_poison` code path
   with a school parameter: a flat, charge-limited DoT/rider *added* to the
   Physical auto. Never a conversion of the auto's school, never a multiplier on
   `attack()`; both are the Thundersmith's monopoly and the line that keeps the
   pass from breaking the game. An oil at x1.5 in the right zone is ~+5% total
   damage: a decision lever, not a power lever.

### Expected net effect (the balance argument)

- Weakness zones only touch school damage; autos are Physical and nothing is
  weak to Physical, so a matchup zone gives a caster +50% on the ~25% ability
  slice (~+12% total) and a martial's autos nothing. The pass is a small
  situational caster buff.
- Physical-resist bosses tax whoever leans hardest on Physical: a Warrior's kit
  is ~90% Physical, a Mage's ~75%. Both hurt, the martial more, exactly at the
  fights where composition and prep should matter.
- Baselines do not move: the neutral case (auto, Physical, unthemed foe) is
  untouched by construction.

### Don't-break-the-game protocol

1. **Data-only.** No engine changes: same `DamageProfile`, same 50/150
   multipliers, one resist + one weak per mob. The whole pass is placement in
   the builders, reviewable as a diff of themed assignments.
2. **Invariants as tests**, in the codebase's existing balance-test style (cf.
   `no_beast_is_out_classed_by_an_easier_one`): no Physical resist on regulars,
   every zone weakness derivable from its theme table, per-school resist/weak
   counts within declared bands so Holy cannot silently stay predator-free.
3. **A before/after sim budget.** Run a per-class grind-rate model across every
   zone. Within a zone, matchups may swing a class up to ±15%; each class's
   *average* across the band must stay within a few percent of today.
   Redistribution yes, rebalancing no. A class whose average moves past the
   budget means the placement is wrong, not the class.

### What the Thundersmith gets out of it

Real ledger data everywhere: every zone becomes probeable, auto-chamber fires on
real information across the whole band, and the counter-cell lines earn their
crafting cost. The pass also gives every other class its own smaller school
game (casters: situational rotations; martials: oils and geography), which keeps
the Thundersmith's edge legible as *degree*, not *kind*: everyone plays
matchups, he industrializes them.
