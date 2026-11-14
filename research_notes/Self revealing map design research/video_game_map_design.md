# Video game map design: self-revealing maps and layered maps (outside text adventures)

Reader's note on evidence. Each cited finding is tagged **[Designer]** (a developer's own words or postmortem), **[Critic]** (a journalist, essayist or outside practitioner's reading), or **[Reference]** (wiki or encyclopedic description of a mechanic). Several findings come from search-result snippets where the full page could not be opened; these are marked *(snippet only)* and should be treated as weaker. Things I know from general familiarity with the games but could not source in this session are listed under Gaps as "unverified background", not as findings.

Three sources I tried and failed to read in full: PC Gamer "How to design a great Metroidvania map" (Team Cherry interview; page returned truncated), PC Gamer's Miyazaki interview on Elden Ring map fragments (truncated), and the Game Maker's Toolkit videos on maps and on "Following the Little Dotted Line" (transcript pages returned no text).

---

## 1. How do the well-regarded exploration maps work, and why are they praised or criticised?

### Takeaway
The praised maps fall into a few distinct families: the automap that fills in as you walk (Metroid, Symphony of the Night), the map you must earn and maintain (Hollow Knight, Elden Ring, Breath of the Wild), the map you make yourself (Etrian Odyssey, Miasmata), the map as a held object (Far Cry 2, Firewatch, Tunic), the map of knowledge rather than space (Outer Wilds, Obra Dinn), and the deliberate absence of a map (Dark Souls, Subnautica). In every well-sourced case the designer's stated reason is the same: the map is tuned to protect a feeling of discovery, and is praised when "incomplete" in a way that invites, criticised when incompleteness becomes unfair or the map falls out of step with the world.

### Cited Findings

**Hollow Knight (earn the map, update at rest)**
- [Designer] Ari Gibson (Team Cherry) says the game is "built around this sense of discovery"; the brief was "let's build an interesting world with lots of things to discover and see". William Pellen says layout "was mainly intuition" and "We started with the idea of the world" rather than a planned route. — [Game Developer](https://gamedeveloper.com/design/how-the-i-hollow-knight-i-devs-mapped-out-their-metroidvania-)
- [Critic] Abhishek Iyer describes the mechanics: each area is blank until you find Cornifer and buy that region's incomplete map; a paper trail and his humming lead you to him; Iselda's shop sells a compass (shows your position), a quill (lets the map fill in), and marker sets for benches, stag stations, shops; purchased markers only show places already visited; explored rooms are drawn in when the player saves at a bench or dies; the map is a physical item and the game does not pause when it is open. — [SUPERJUMP](https://www.superjumpmagazine.com/getting-lost-by-design-in-hollow-knight/)
- [Critic] Iyer's argument: making the player earn navigation aids that other games give free lets each player choose how much help they want, and he explicitly hedges that diegetic UI "isn't the right UI choice for every game". — [SUPERJUMP](https://www.superjumpmagazine.com/getting-lost-by-design-in-hollow-knight/)
- [Reference] If the player misses Cornifer, Iselda sells that area's map later at a higher price. *(snippet only)* — [Hollow Knight Wiki](https://hollowknight.fandom.com/wiki/Map)
- [Critic] A dissenting strand exists: a video essay titled "Hollow Knight's Map is Broken" is indexed, but I could not read it. *(title only; content not verified)* — [Amara subtitle index](https://amara.org/subtitles/stk1ABNP63rg/en/2/download/Hollow%20Knights%20Map%20is%20Broken.en.sbv)

**Breath of the Wild (towers reveal terrain only; the player marks the rest)**
- [Critic, professional cartographers] Ross Thorn and Alec Burch (Stamen) score the map 10/10 on "interactive", "immersive" and "incomplete". Towers reveal a region's terrain and names; unexplored regions are covered by a grid and swirl pattern that Thorn calls "a great 'undiscovered' area texture that isn't a traditional fog view". — [Stamen](https://stamen.com/cartographers-play-video-games-a-review-of-the-map-in-the-legend-of-zelda-breath-of-the-wild/)
- [Critic] Some named places do not appear until visited; shrines and towers have a third icon state, "discovered but incomplete", that changes colour when completed. — [Stamen](https://stamen.com/cartographers-play-video-games-a-review-of-the-map-in-the-legend-of-zelda-breath-of-the-wild/)
- [Critic] Mark Brown (GMTK) on the same design: after a tower "you're still expected to do the heavy lifting", using the scope to spot places and then placing pins. *(snippet only)* — [GMTK Substack](https://gmtk.substack.com/p/how-nintendo-solved-zeldas-open-world)
- [Critic] Stamen's only complaints are that the map does not track world changes (a felled tree stays on the map) and a duplicated label at some zooms. — [Stamen](https://stamen.com/cartographers-play-video-games-a-review-of-the-map-in-the-legend-of-zelda-breath-of-the-wild/)
- [Designer, via press report] The "triangle rule" for landmark placement comes from a CEDEC 2017 field-design talk by Fujibayashi and Yonezu, not the GDC 2017 talk. *(snippet only)* — [Nintendo Life](https://www.nintendolife.com/news/2017/10/zelda_breath_of_the_wilds_ingenious_design_is_all_about_triangles_apparently)

**Elden Ring (map fragments) and Dark Souls (no map)**
- [Critic, cartographers] Thorn describes two layers of concealment: a cloud overlay for unvisited ground, and beneath it an unfilled base that fragments fill in; before a fragment is found the faint base still shows "the main roads to help guide you a bit" and shows where the fragment pillar is. Katie Kowalsky: "it's not a complete blank slate", which she finds "so enticing". — [Stamen](https://stamen.com/cartographers-play-video-games-a-review-of-the-map-in-elden-ring/)
- [Critic] Thorn calls the empty map "such a motivating thing" with only "the implicit suggestion to go explore", and says the hand-drawn look makes it "really feel like it's an in-game artifact". Scores: interactive 6, immersive 6, incomplete 8, overall 6. — [Stamen](https://stamen.com/cartographers-play-video-games-a-review-of-the-map-in-elden-ring/)
- [Critic] Criticisms: guidance icons that "point in completely opposite directions" and linger after their purpose ends; markers placed on the surface map not showing on the underground map; contour-like lines whose meaning is unclear; map artefacts that cannot be zoomed enough to read. — [Stamen](https://stamen.com/cartographers-play-video-games-a-review-of-the-map-in-elden-ring/)
- [Designer, via press summary] Miyazaki is reported as saying fragments do not reveal the whole world so discovery remains, and that fragments are easy to get because exploring with a map is fun. *(snippet only; the interview page would not load)* — [PC Gamer](https://pcgamer.com/miyazaki-likes-to-play-a-tough-elden-ring-character-that-can-trade-blows)
- [Designer] Miyazaki says the first step in designing Elden Ring's map was deciding where the bosses go. *(snippet only)* — [Xbox Wire](https://news.xbox.com/en-us/2022/05/27/from-softwares-hidetaka-miyazaki-on-the-secrets-of-elden-rings-development/)
- [Critic] One essayist holds that having no map and no early fast travel was among Dark Souls' best decisions and that Elden Ring's map is "a significant concession". *(snippet only)* — [Dada Issues](https://dadaissues.substack.com/p/mending-the-elden-ring)

**Etrian Odyssey (the player draws the map)**
- [Designer] Director Kazuya Niinou: the handwritten map was "one of the things we definitely wanted to incorporate"; mapping belongs to a "study style" of play where you "sit at your desk, while drawing the precise map and writing memos on it"; he wants players to "relax, drink some coffee" and concentrate on mapping the unknown dungeon. The stylus is used only for the map. — [Niinou's Director's Diary, translated](https://www.dampfkraft.com/games/etrian-odyssey-directors-diary.html)
- [Designer] Niinou in 2008: "The foundation of this game is drawing a map on the Nintendo DS' touch screen". *(snippet only)* — [Engadget](https://www.engadget.com/2008-06-05-etrian-odyssey-ii-director-speaks-about-the-unexpected-hit-serie.html)
- [Designer] For the HD remaster the team reworked touch detection and tool layout and added a cursor-position option for finger taps. *(snippet only, repost of an IGN interview)* — [repost](https://sameteem.com/threads/exclusive-etrian-odysseys-devs-on-adapting-its-touchscreen-to-switch-and-its-80-price-point.21999/)

**Miasmata (triangulation)**
- [Designer] Bob Johnson (IonFX): "I think getting lost is everything in our game, honestly. It's the core mechanic". The aim was "to add significance to the simple act of traveling from one place to the next". The cartography idea came from his brother Joe, who wanted something more comprehensive; Bob "convinced him to develop a more rudimentary system". He finds it "both funny and disappointing" when players skip the system. — [Low Cutoff interview](https://lowcutoff.blogspot.com/2013/06/bonus-feature-interview-with-miasmata.html)
- [Reference] A landmark becomes "known" once its position has been calculated by triangulation. *(snippet only)* — [Miasmata Wiki](https://miasmata.fandom.com/wiki/File:Cartography_table.png)
- [Critic / players] PopMatters calls it a "hugely abstracted" version of real cartography; a GOG thread shows players who understood triangulation in principle but could not tell why some sightings revealed map and others did not. *(snippets only)* — [PopMatters](https://www.popmatters.com/181832-feeling-lost-in-miasmata-2495659408.html); [GOG forum](https://gog.com/forum/miasmata/please_explain_the_mapping_to_an_idiot)

**Firewatch and Far Cry 2 (held paper map)**
- [Critic] Firewatch has no minimap or menu map; Henry takes out a paper map and, separately, a compass. He draws paths and circles important places on it as the story proceeds; directions from the other character reference map landmarks. — [Game Developer blog, Abhishek Iyer](https://gamedeveloper.com/design/how-firewatch-s-ui-enhances-player-immersion)
- [Reference] The player has "a compass and a map that Henry appends additional information to throughout the game". *(snippet only)* — [Wikipedia](https://en.wikipedia.org/wiki/Firewatch)
- [Designer action] Campo Santo published printable versions of the in-game map, including a clean one meant to be marked up. *(snippet only)* — [DualShockers](https://www.dualshockers.com/campo-santo-releases-printable-firewatch-maps-for-cooperative-playthrough-or-your-wall/)
- [Critic] Far Cry 2: "When you bring up the game map, it's a physical map that you hold in your hand", usable while driving. *(snippet only)* — [bit-tech](https://bit-tech.net/features/gaming/pc/10-years-on-far-cry-2/1/)

**Outer Wilds (ship log, rumour mode)**
- [Designer] Mobius Digital, Feb 2018: a new ship computer in a "detective board style" was introduced to help players work out a next step when stuck; first playtests with it were a success; it gives a "more visual guide" without changing the core experience. — [Mobius Digital blog](https://www.mobiusdigitalgames.com/news/a-look-into-playtests)
- [Designer] Post-launch patch notes: "Question marks in rumor mode are now orange"; the log "now defaults to map mode when there is no new info to reveal". *(snippet only)* — [Mobius Digital news](https://www.mobiusdigitalgames.com/news/category/all/4)
- [Critic] Edge: what opens the game up is not upgrades but "snippets of knowledge" stored in the ship's log as an expanding web of rumours. *(snippet only)* — [Edge via PressReader](https://www.pressreader.com/australia/edge/20190620/282278141843961)

**Tunic (manual pages)**
- [Designer] Andrew Shouldice: "A key part of feeling like a world isn't made for you is feeling like it was made for someone else." Old manuals "were free to be vague and evocative". "A full-screen tutorial popup feels deeply invasive and can ruin any sense of wonder". He wants to give "the player space to explore — not just geographically, but mechanically". — [Game Developer](https://www.gamedeveloper.com/road-to-igf-2023/how-tunic-weaves-wondrous-unknowable-worlds-inspired-by-inscrutable-nes-manuals)
- [Critic] The map pages are static, with landmarks and paths drawn on them, so the player must compare surroundings to the page and trace the route mentally; some spreads are found half at a time. *(snippets only)* — [GameSpot](https://gamespot.com/articles/tunic-challenges-you-to-find-all-its-secrets-and-i-cant-wait-to-try/1100-6493563/)
- [Designer] For most of development the manual was blank pages with placeholder notes, which made playtesting hard. *(snippet only)* — [Game Informer](https://www.gameinformer.com/2022/05/30/tracing-threads-the-making-of-tunic)

**Subnautica (no map; beacons)**
- [Critic, working designer] Robert Yang: no map "helps exploration feel genuinely dangerous and confusing" and sidesteps the hard problem of legibly mapping a large 3D volume. Players navigate by remembered landmarks. But guidance beacons stop in the second half; he estimates he missed about a quarter of the world in roughly 40 hours; the world's edge is not distinguishable from other dark deep areas, so "Subnautica cries wolf but punishes me". His proposed fix: vague, incomplete map posters in ruined bases that the player can photograph. — [Radiator blog](https://www.blog.radiator.debacle.us/2018/02/mapping-sea-floors-of-subnautica.html)

**Metroid, Symphony of the Night, Obra Dinn, Animal Well**
- [Critic] Super Metroid's map works as a reminder of where its long main paths run and lead; item dots flag "something of significance" but not every hidden item; map stations supply area maps. *(snippets only)* — [VG Thought](https://vgthought.com/hesitant/maplessness_in_metroid); [Metroid Wiki](https://metroid.fandom.com/wiki/Map)
- [Critic] Metroid Prime's 3D automap is called "detailed, yet easy to use"; a level-by-level analysis says it is groundbreaking but that the winding, repetitive Phazon Mines expose its weaknesses. *(snippets only)* — [Game Sense review](https://gamesense15.substack.com/p/metroid-prime-review); [Goomba Stomp](https://goombastomp.com/metroid-prime-level-level-phazon-mines/)
- [Designer, via press] Koji Igarashi names A Link to the Past, more than Super Metroid, as the main model for Symphony of the Night's exploration. *(snippet only)* — [GameSpot](https://gamespot.com/articles/castlevanias-koji-igarashi-explains-the-influence-/1100-6481658/)
- [Critic] Obra Dinn's deck map lets the player keep track of flashbacks and discoveries and "gives the ship a sense of presence"; a separate sea chart traces the voyage; each crew member's entry has a map of where clues about that person are found. *(snippets only; the first is pre-release coverage)* — [Kill Screen](https://killscreen.com/articles/return-obra-dinns-ghostly-journey-starting-really-come-together); [Mechanics of Magic](https://mechanicsofmagic.com/?p=54104)
- [Designer, via press] Animal Well is a roughly 16 by 16 grid of rooms, about 256, with more than 500 rooms cut or reworked (Billy Basso to Edge). *(snippet only)* — [GamesRadar](https://gamesradar.com/games/platformer/solo-dev-behind-breakout-metroidvania-hit-animal-well-says-his-game-has-256-encounters-but-he-had-to-cut-at-least-500-of-them-during-development)

### Inferences
- The designers who are on record (Gibson, Niinou, Johnson, Shouldice, Mobius) all justify their map by a feeling, not by convenience: discovery, calm study, the fear and relief of being lost, a world not made for you, getting unstuck. A phone app should pick the feeling first and derive the map rules from it.
- The cartographers' "incomplete" criterion is the most transferable measure: a map scores well when its blank parts motivate. Both top-scoring examples leave something faint under the fog (Elden Ring's roads, Breath of the Wild's patterned void with named regions) instead of a flat black.
- Hollow Knight splits "the map" into separately earned parts: the outline, your position on it, the ability to record, and each icon family. That decomposition is the lesson, more than the specific prices.
- Updating the map at a rest point (Hollow Knight) creates a rhythm of venture then consolidate. A turn-based text game has natural equivalents (a save, a new session, leaving a region), but the same sources show this is also the most divisive choice.
- Outer Wilds' log was added to solve a stuck-player problem found in playtests, and was framed as a detective board. That is direct support for a "what do I know and what is unresolved" view living next to a spatial view.
- Miasmata and Subnautica both show the cost side: when the rule for what gets revealed is opaque, or the game withdraws guidance without saying so, players read it as a bug, not as mystery.

### Gaps
- No primary Team Cherry statement on why the map updates only at benches, why the compass costs a charm slot, or the reported "98% pure" remark; the PC Gamer interview that probably holds them would not load.
- No primary Miyazaki text on fragments; only a summary.
- No primary Mobius/Alex Beachum account of how rumour mode's cards, links and "more to explore here" flag were designed. Noclip's documentary and Beachum's thesis were not reached.
- No designer source found for Super Metroid, Metroid Prime or Symphony of the Night map screens specifically.
- Nothing usable found on Heaven's Vault's map and timeline, La-Mulana's map system, or Animal Well's map tools; nothing on Lucas Pope's own reasoning for Obra Dinn's deck plan.
- No Unknown Worlds statement on why Subnautica has no map; community reports only.
- Unverified background (from my own familiarity, not sourced this session; verify before use): Super Metroid distinguishes station-revealed rooms from visited rooms by colour and hides secret rooms from stations; Metroid Prime colours doors on the map by the weapon needed and lets the player rotate a 3D wireframe; Symphony of the Night shows a map completion percentage and sells a partial castle map; Hollow Knight's compass occupies a charm slot; Outer Wilds' rumour cards are colour-coded by storyline, show "?" for places heard of but not found, and flag "there's more to explore here"; Etrian Odyssey offers optional auto-fill of walked floor tiles while walls and icons remain manual; La-Mulana requires finding each area's map and loading reader software into limited laptop memory; Animal Well's map, stamps and pencil are separate findable items; Heaven's Vault narrows the search area for a new site as you gather clues and keeps an automatically built timeline; Firewatch has an option to show or hide the player's position on the map.

---

## 2. What layers of information do good maps offer, and how are layers switched or blended without clutter?

### Takeaway
Well-regarded maps keep the base layer quiet (terrain and visited space) and add the rest as states of icons, zoom-dependent labels, or a small number of opt-in overlays; the best-documented anti-clutter devices are a three-state icon, label density tied to zoom, markers that expire when used, and a separate non-spatial view for knowledge.

### Cited Findings
- [Critic] Breath of the Wild layers: contour lines and elevation tint for terrain, water-depth colouring, region names, place labels that appear only when zoomed in, and icons with undiscovered / discovered-but-incomplete / complete states. Zoom snaps to set levels with a blur transition. — [Stamen](https://stamen.com/cartographers-play-video-games-a-review-of-the-map-in-the-legend-of-zelda-breath-of-the-wild/)
- [Critic] "Where you have been" as an overlay: the Hero's Path draws the player's whole route as a line, which Thorn says shows "where you haven't been"; it can be replayed as an animation. — [Stamen](https://stamen.com/cartographers-play-video-games-a-review-of-the-map-in-the-legend-of-zelda-breath-of-the-wild/)
- [Critic] Danger is filtered, not exhaustive: only one class of enemy appears on the map; "the map doesn't show locations for beast-like enemies". — [Stamen](https://stamen.com/cartographers-play-video-games-a-review-of-the-map-in-the-legend-of-zelda-breath-of-the-wild/)
- [Critic] Two kinds of player mark with different lifetimes: pins are one-time beacons that "disappear when you reach them"; stamps persist until removed, are unlabeled so the player can "assign your own meaning", and number up to 100. — [Stamen](https://stamen.com/cartographers-play-video-games-a-review-of-the-map-in-the-legend-of-zelda-breath-of-the-wild/)
- [Critic] Elden Ring: ten marker symbols, up to a hundred markers, styled as wax stamps; clutter arises from system-placed guidance icons that never clear, and from a two-level world where marks on one level are invisible on the other. — [Stamen](https://stamen.com/cartographers-play-video-games-a-review-of-the-map-in-elden-ring/)
- [Critic] Hollow Knight sells icon families separately (benches, stations, shops and others), so each player's map carries only the layers they chose to buy, and those icons only appear for visited places. — [SUPERJUMP](https://www.superjumpmagazine.com/getting-lost-by-design-in-hollow-knight/)
- [Designer] Outer Wilds keeps knowledge in a separate mode from space: the log has a rumour mode and a map mode, and after a patch opens on map mode when there is nothing new to reveal. *(snippet only)* — [Mobius Digital news](https://www.mobiusdigitalgames.com/news/category/all/4)
- [Critic] Obra Dinn attaches a per-person map of clue locations to each crew entry, i.e. the spatial layer is filtered by subject. *(snippet only)* — [Mechanics of Magic](https://mechanicsofmagic.com/?p=54104)
- [Designer] Sorcery!'s map carries time and consequence: the drawn line of the journey is a progress meter, shows roads not taken, and is the rewind control. — [Jon Ingold postmortem, Game Developer](https://gamedeveloper.com/business/postmortem-i-steve-jackson-s-sorcery-i-series-by-inkle)
- [Critic] A map that does not reflect current world state is noticed as a flaw even by admirers. — [Stamen](https://stamen.com/cartographers-play-video-games-a-review-of-the-map-in-the-legend-of-zelda-breath-of-the-wild/)

### Inferences
- A workable layer stack for a text adventure map, each mapped to a proven device: rooms visited (base fill); exits seen but not taken (stub or faint outline, as Elden Ring's faint roads); locked or blocked ways (a third icon state, "found but unresolved", as Breath of the Wild's shrines); items left behind and characters (icons that show only for visited rooms, as Hollow Knight); route history and time (a path line with replay or rewind, as Hero's Path and Sorcery!); player notes (free stamps with no fixed meaning plus expiring "go here" pins).
- The three-state icon is the single cheapest way to encode "what opens this" without text on the map: unknown, known-unsolved, solved.
- Tying label density to zoom means the far view shows shape and regions, the near view shows names and contents. On a phone this matters more than on a television.
- Expiring pins versus permanent stamps is a clean answer to marker litter; Elden Ring's lingering guidance icons are the counter-example.
- "Filter by subject" (Obra Dinn) suggests a query style of layer: show me everywhere this object or person has been mentioned or seen.
- If the game world changes (a door opens, a bridge collapses), the map must change too, or it loses trust.

### Gaps
- No designer-authored source found that discusses layer toggling UI as such (legend screens, filter chips, hold-to-reveal).
- I did not find sourced treatment of maps that show time of day or schedules as a layer, nor of danger heat-maps.
- Unverified background: Metroid Dread lets the player highlight all doors of one type and blinks unexplored openings; many open-world games offer icon filter lists that critics blame for "checklist" play. Neither was sourced this session.

---

## 3. How do games relate a small always-visible map to a full map, and what makes the transition feel good?

### Takeaway
The clearest sourced statement is a division of labour: the small map exists only for moment-to-moment wayfinding and the full map for planning and reference, with player marks bridging the two; several praised games remove the small map entirely and make "looking at the map" a deliberate act that costs attention.

### Cited Findings
- [Critic] In Breath of the Wild the full map is for planning and reference while the mini-map is "entirely dedicated to supporting navigation goals during gameplay"; pins set on the full map dock to the mini-map edge as direction cues. — [Stamen](https://stamen.com/cartographers-play-video-games-a-review-of-the-map-in-the-legend-of-zelda-breath-of-the-wild/)
- [Critic] The mini-map changes scale with context: it zooms in on entering a village to show shops and inns, as navigation apps do. — [Stamen](https://stamen.com/cartographers-play-video-games-a-review-of-the-map-in-the-legend-of-zelda-breath-of-the-wild/)
- [Critic] Full-map zoom snaps between set levels "combined with a slight blur animation to make it look smooth". — [Stamen](https://stamen.com/cartographers-play-video-games-a-review-of-the-map-in-the-legend-of-zelda-breath-of-the-wild/)
- [Critic] Elden Ring's map fills the screen without pausing the game, so consulting it in danger is risky; Hollow Knight's map likewise does not pause. — [Stamen](https://stamen.com/cartographers-play-video-games-a-review-of-the-map-in-elden-ring/); [SUPERJUMP](https://www.superjumpmagazine.com/getting-lost-by-design-in-hollow-knight/)
- [Critic] Firewatch and Far Cry 2 have no small map; the character raises a paper map into view, and in Firewatch the compass is a second, separate object. Iyer argues the extra effort is intended because exploration is the point. — [Game Developer blog](https://gamedeveloper.com/design/how-firewatch-s-ui-enhances-player-immersion); [bit-tech](https://bit-tech.net/features/gaming/pc/10-years-on-far-cry-2/1/)
- [Designer] Outer Wilds puts its log only in the ship, a place the player returns to. — [Mobius Digital blog](https://www.mobiusdigitalgames.com/news/a-look-into-playtests)

### Inferences
- If the map is the main screen, the roles invert: the "small" thing is the text of the current room and the map is always full. The Breath of the Wild split still applies as two zoom states of one map: a near state for "where am I and what are my exits" and a far state for planning, with a snapped, animated transition between them rather than free pinch only.
- Context-driven zoom (tighter in dense places, wider in open ones) is a sourced precedent for auto-framing the current neighbourhood.
- Continuity is what makes the transition feel good in the sourced examples: the same marks appear in both views, the zoom is animated and lands on fixed levels, and the map is the same object at both scales (a held sheet, a wax-stamped parchment).
- The non-pausing map is a tension device for real-time games; in a turn-based text game the analogue would be a map that keeps showing the latest room text or turn state while open, so opening it is never a context loss.

### Gaps
- Mark Brown's "Following the Little Dotted Line" (GMTK), the best-known critique of mini-maps and waypoint lines, could not be read; its argument is not represented here.
- No designer source found on mini-map design in Metroid or Symphony of the Night.
- Unverified background: Super Metroid shows a small grid of nearby rooms in a screen corner and a full area map on pause; critics commonly say corner mini-maps draw the eye away from the world. Not sourced this session.

---

## 4. What do designers say about fog of war, showing unexplored exits, and avoiding spoilers?

### Takeaway
On record, designers defend partial concealment as the source of discovery and of the pleasure of becoming un-lost, while practitioners and critics insist the hidden part must still hint: a blank that suggests shape or direction is praised, and a blank that gives no signal that something remains is where complaints start.

### Cited Findings
- [Designer] Bob Johnson: being lost is "the core mechanic"; he links it to childhood trips where the fear of getting lost "was persistently in the back of our minds", and says the fear faded once unknowns became knowns through map and compass skill. — [Low Cutoff](https://lowcutoff.blogspot.com/2013/06/bonus-feature-interview-with-miasmata.html)
- [Designer] Shouldice: removing readable information is meant to leave the player a little adrift and let imagination fill gaps; discoveries players make themselves feel more personal. — [Game Developer](https://www.gamedeveloper.com/road-to-igf-2023/how-tunic-weaves-wondrous-unknowable-worlds-inspired-by-inscrutable-nes-manuals)
- [Designer, via press summary] Miyazaki: fragments do not show the whole world, to keep discovery; they are easy to get because a map is fun to explore with. *(snippet only)* — [PC Gamer](https://pcgamer.com/miyazaki-likes-to-play-a-tough-elden-ring-character-that-can-trade-blows)
- [Designer] Mobius added a visual log specifically to help stuck players find a next step, and reports it worked in playtests. — [Mobius Digital blog](https://www.mobiusdigitalgames.com/news/a-look-into-playtests)
- [Critic] Elden Ring's unrevealed map is "not a complete blank slate": faint roads and the fragment location show through, guiding without marking exactly; quest hints are given as simple sketch maps of an endpoint that the player must still locate. — [Stamen](https://stamen.com/cartographers-play-video-games-a-review-of-the-map-in-elden-ring/)
- [Critic] Breath of the Wild replaces fog with a patterned "undiscovered" texture and holds back some place names until visited, even inside a revealed region. — [Stamen](https://stamen.com/cartographers-play-video-games-a-review-of-the-map-in-the-legend-of-zelda-breath-of-the-wild/)
- [Critic] Hollow Knight's icons never pre-announce: bought marker sets show only places already visited, so the next bench is always unknown. — [SUPERJUMP](https://www.superjumpmagazine.com/getting-lost-by-design-in-hollow-knight/)
- [Critic] Super Metroid's item dots signal that something significant is in a room without listing every secret. *(snippet only)* — [Metroid Wiki](https://metroid.fandom.com/wiki/Map)
- [Critic, designer] Yang on the failure case: with no map and no late-game beacons, a player can miss a quarter of the world and cannot tell a fatal boundary from ordinary unexplored dark. He asks for fallback pointers to unexplored places and for incomplete in-world maps. — [Radiator blog](https://www.blog.radiator.debacle.us/2018/02/mapping-sea-floors-of-subnautica.html)
- [Designer] Outer Wilds marks unresolved leads with question marks in rumour mode (colour changed to orange in a patch, implying they needed to stand out more). *(snippet only)* — [Mobius Digital news](https://www.mobiusdigitalgames.com/news/category/all/4)

### Inferences
- A defensible spoiler rule drawn from these: the map may show that something exists and roughly where, never what it is, and only on evidence the player has actually encountered (seen an exit, read a rumour, bought the chart). Hollow Knight, Super Metroid's dots and Outer Wilds' question marks all follow it.
- "Seen but not reached" deserves its own visual state. Outer Wilds' question mark and Breath of the Wild's incomplete icon are the precedents; for a text adventure the obvious instance is an exit listed in a room description that has not been taken.
- Complete blankness is a risk both ways: it removes the invitation (Stamen's point) and removes the safety net (Yang's point). Some residue under the fog is better than none.
- Hints work better as destinations the player must still locate (Elden Ring's sketch maps) than as lines to follow.
- A late or optional "you have unexplored leads here" nudge is supported by a designer (Mobius) and requested by a critic (Yang); making it opt-in or delayed respects players who want to be lost.

### Gaps
- No direct designer quote found on the specific practice of drawing exit stubs for untaken doors (Metroid-style openings in room walls); what I have is critic description only.
- No sourced discussion of the spoiler problem peculiar to revealing room names or counts ("12 of 40 rooms found") before the player has earned them.
- Team Cherry's own words on concealment were not retrievable.
- Unverified background: Outer Wilds' "there's more to explore here" tag on a log card is widely praised as a spoiler-free completeness signal; Metroid Dread blinks unexplored openings; Symphony of the Night's percentage counter tells players a second castle exists by exceeding 100. Not sourced this session.

---

## 5. Which games let the player annotate the map, and which do automatic annotation well? What is the argument for making the player do some of the mapping themselves?

### Takeaway
Manual mapping is defended by its designers as an activity worth doing for its own sake (calm, studious, or tense and skilful) that gives meaning to travel; automatic annotation is praised when it records what the player did or learned in the character's voice. The successful middle ground is automatic geometry with player-owned meaning: free-meaning stamps and short-lived pins on top of a self-drawing map.

### Cited Findings
- [Designer] Niinou frames drawing the map as the foundation of Etrian Odyssey and as a deliberate "study style" pleasure, drawing a precise map and "writing memos on it"; he limited the stylus to mapping so that it would not become tiresome. — [Director's Diary](https://www.dampfkraft.com/games/etrian-odyssey-directors-diary.html); [Engadget](https://www.engadget.com/2008-06-05-etrian-odyssey-ii-director-speaks-about-the-unexpected-hit-serie.html)
- [Critic] Etrian Odyssey V is described as a tribute to the practice of mapping games by hand. *(title and snippet only)* — [Retronauts](https://retronauts.com/article/611/etrian-odyssey-v-a-diy-tribute-to-the-art-of-game-mapping)
- [Designer] Johnson: Miasmata's player-made map exists "to add significance to the simple act of traveling"; skipping it is like "playing Call of Duty without pulling the trigger". The brothers also chose a simpler mapping system than first proposed. — [Low Cutoff](https://lowcutoff.blogspot.com/2013/06/bonus-feature-interview-with-miasmata.html)
- [Critic] Breath of the Wild: towers give terrain, the player supplies points of interest by spotting them and pinning; stamps carry no label so meaning is the player's; pins can be converted to stamps. — [Stamen](https://stamen.com/cartographers-play-video-games-a-review-of-the-map-in-the-legend-of-zelda-breath-of-the-wild/); [GMTK Substack](https://gmtk.substack.com/p/how-nintendo-solved-zeldas-open-world)
- [Critic] Elden Ring offers ten stamp symbols and a hundred markers and almost no automatic points of interest beyond rest sites, which Kowalsky says preserves the satisfaction of finding things. — [Stamen](https://stamen.com/cartographers-play-video-games-a-review-of-the-map-in-elden-ring/)
- [Critic] Automatic, in-character annotation: Firewatch's Henry draws routes and circles places as events happen, and what appears depends on how the player played. *(second half snippet only)* — [Game Developer blog](https://gamedeveloper.com/design/how-firewatch-s-ui-enhances-player-immersion); [review](https://mikepennisi.com/blog/2022/review-firewatch/)
- [Critic] Automatic with a delay: Hollow Knight records explored rooms only on resting or dying, and only once the quill is owned. — [SUPERJUMP](https://www.superjumpmagazine.com/getting-lost-by-design-in-hollow-knight/)
- [Designer] Automatic knowledge annotation: Outer Wilds' detective-board computer records what the player has learned and was validated in playtests as a help for stuck players. — [Mobius Digital blog](https://www.mobiusdigitalgames.com/news/a-look-into-playtests)
- [Designer] Sorcery! annotates automatically by drawing the journey line, which doubles as proof that choices had consequences. — [Game Developer](https://gamedeveloper.com/business/postmortem-i-steve-jackson-s-sorcery-i-series-by-inkle)
- [Players] The cost of manual systems: Miasmata players reported not understanding why their sightings did or did not reveal map. *(snippet only)* — [GOG forum](https://gog.com/forum/miasmata/please_explain_the_mapping_to_an_idiot)
- [Critic, designer] Yang's suggested compromise for a map-less game is letting players photograph partial in-world maps, i.e. player-collected rather than player-drawn. — [Radiator blog](https://www.blog.radiator.debacle.us/2018/02/mapping-sea-floors-of-subnautica.html)

### Inferences
- The arguments for player effort, as designers state them: it is a pleasant activity in itself (Niinou); it makes ordinary movement matter (Johnson); self-made discoveries are remembered and used (Shouldice, said of mechanics, plausibly true of maps). The argument against, from the same sources: it must be optional in degree and its rules must be transparent, or it reads as chore or bug.
- Both designers who built manual mapping also bounded it: Niinou confined the stylus to one job; the Johnsons simplified. Effort should be concentrated where it carries meaning.
- For a phone text-adventure map, geometry and room names are the tedious part and the part an automap does reliably; the meaningful part is interpretation ("come back with a light", "troll wants payment"). The Breath of the Wild split, machine draws terrain and player marks significance, fits that division.
- Unlabeled stamps avoid a keyboard on a phone; an optional short note covers the cases symbols cannot.
- Firewatch suggests a voice for automatic notes: written as the character's or a companion's handwriting, tied to what actually happened, so two players' maps differ.

### Gaps
- No controlled or designer-reported evidence found on whether manual mapping improves recall or enjoyment; the case rests on designer intent and critic testimony.
- Could not confirm from a primary source how Etrian Odyssey's optional auto-mapping divides work between game and player.
- Nothing sourced on La-Mulana's or Animal Well's annotation tools, or on Metroid Dread's or Hollow Knight: Silksong's pin systems.
- Unverified background: several recent games (Animal Well, Tunic's manual scribbles, Chants of Sennaar's notebook) hand the player a pencil or free-text notes; The Legend of Zelda: Phantom Hourglass let players write on the map with a stylus and built puzzles around it. Not sourced this session.

---

## 6. Which games make the map a physical or in-world object, and what does that add?

### Takeaway
Held or authored maps (Far Cry 2, Firewatch, Hollow Knight, Elden Ring, Tunic, Miasmata, Obra Dinn) add three things the sources name: belief that the map belongs to the world, a cost and ritual to consulting it, and an author with a voice and limits, so that imprecision and gaps read as character instead of missing features.

### Cited Findings
- [Critic] Far Cry 2's map is a physical sheet in the character's hand, readable while driving, alongside a handheld GPS. *(snippet only)* — [bit-tech](https://bit-tech.net/features/gaming/pc/10-years-on-far-cry-2/1/); [HandWiki](https://handwiki.org/wiki/Software:Far_Cry_2)
- [Critic] Firewatch's map is "as close to a real paper map as a game can get"; orientation requires a separate compass; Iyer argues the friction is the point and notes Far Cry 2 did it first. A period justification is also offered: map and compass were the real tools in a 1980s forest. — [Game Developer blog](https://gamedeveloper.com/design/how-firewatch-s-ui-enhances-player-immersion)
- [Designer action] Campo Santo released the map as a printable, including a clean copy for marking up while playing. *(snippet only)* — [DualShockers](https://www.dualshockers.com/campo-santo-releases-printable-firewatch-maps-for-cooperative-playthrough-or-your-wall/)
- [Critic] Hollow Knight's map has an in-world author and seller (Cornifer, Iselda), is found by following his paper trail and humming, and is a carried item. — [SUPERJUMP](https://www.superjumpmagazine.com/getting-lost-by-design-in-hollow-knight/)
- [Critic] Elden Ring's map reads as "an in-game artifact"; wax-stamp markers add to this; Thorn compares the feeling to the paper map boxed with Morrowind. Elements drawn in a different style (the brown contour-like lines) "work against the immersive nature". — [Stamen](https://stamen.com/cartographers-play-video-games-a-review-of-the-map-in-elden-ring/)
- [Critic] Breath of the Wild's map is styled as the output of in-world technology; its tile-like grid over unknown areas fits that fiction. — [Stamen](https://stamen.com/cartographers-play-video-games-a-review-of-the-map-in-the-legend-of-zelda-breath-of-the-wild/)
- [Designer] Tunic's manual is an object made "for someone else"; Shouldice chose it over pop-ups because studying a page preserves wonder. Its maps are fixed drawings the player must match to the world. — [Game Developer](https://www.gamedeveloper.com/road-to-igf-2023/how-tunic-weaves-wondrous-unknowable-worlds-inspired-by-inscrutable-nes-manuals); [GameSpot](https://gamespot.com/articles/tunic-challenges-you-to-find-all-its-secrets-and-i-cant-wait-to-try/1100-6493563/)
- [Critic] Obra Dinn's deck plan sits inside the insurance adjuster's book and gives the ship "a sense of presence". *(snippet only)* — [Kill Screen](https://killscreen.com/articles/return-obra-dinns-ghostly-journey-starting-really-come-together)
- [Designer] Niinou evokes the physical ritual directly: desk, coffee, careful drawing. — [Director's Diary](https://www.dampfkraft.com/games/etrian-odyssey-directors-diary.html)
- [Designer] Sorcery!'s map began from the maps printed at the front of fantasy novels: "What if you could visit any of them?" — [Game Developer](https://gamedeveloper.com/business/postmortem-i-steve-jackson-s-sorcery-i-series-by-inkle)

### Inferences
- An authored map excuses incompleteness. Cornifer's maps are unfinished because he is still surveying; a manual page is torn; a parchment has not been joined yet. A phone app can borrow this: the map is "your notebook", so it knows only what you have seen.
- Material consistency matters: Stamen's complaint about one off-style line set suggests every layer, including player stamps and system hints, should look made by the same hand.
- The object framing supplies natural gestures for a touch screen: unfold to zoom out, stamp, pencil, turn a page between regions or levels.
- A static map with no "you are here" (Tunic, Firewatch by default as reported by critics) turns reading the map into a small puzzle. For a main screen used every turn that is probably too costly as a default, but it is a credible optional mode.
- Text adventures often shipped with paper maps and props in the box; an in-world-object map is in the tradition of the medium, though that claim belongs to the researcher covering text adventures.

### Gaps
- No Clint Hocking or Campo Santo primary statement on the held map was retrieved; the Firewatch claim that Jake Rodkin championed the diegetic map appeared only on an unreliable fan page and is not used above.
- No sourced detail on Miasmata's map as a journal object, or La-Mulana's in-fiction laptop.
- Unverified background: Metro 2033's clipboard and lighter, Sea of Thieves' held treasure maps that other players can read over your shoulder, and Pentiment's and Return of the Obra Dinn's book interfaces are frequently cited alongside these. Not sourced this session.

---

## 7. Games where the map is the primary interface through which the player acts

### Takeaway
inkle's Sorcery! and 80 Days are the best-documented cases, and directly relevant because they are text-driven mobile games: the designer credits the map with turning a gamebook into a game by making it the progress meter, the visible record of consequence, the strategy layer, and the undo control at once.

### Cited Findings
- [Designer] Jon Ingold: the map was "the single best design decision" and "the map is what turned Sorcery! from a gamebook into a game". It began from front-of-novel fantasy maps. The player draws a path to a destination, a counter moves along it, and prose appears on arrival. — [Game Developer postmortem](https://gamedeveloper.com/business/postmortem-i-steve-jackson-s-sorcery-i-series-by-inkle)
- [Designer] Roles Ingold lists for the map: progress meter; proof of consequence without a "the game will remember that" notice (take the river and you visibly bypass the village); strategic layer for choosing destinations and risks; checkpointing, since "you simply rewind along the line of your journey"; and an end-of-game record of the whole trip. — [Game Developer postmortem](https://gamedeveloper.com/business/postmortem-i-steve-jackson-s-sorcery-i-series-by-inkle)
- [Designer] Over the series inkle moved more choices out of the prose and onto the map, added interior maps so buildings are explored room by room, made part 3 open-world, and let the player alter the landscape from the map. — [Game Developer postmortem](https://gamedeveloper.com/business/postmortem-i-steve-jackson-s-sorcery-i-series-by-inkle)
- [Critic] 80 Days frames its interface around a globe, with the major choices being which routes to take and places to visit. *(snippet only)* — [50 Years of Text Games](https://if50.substack.com/p/2014-80-days)
- [Designer] Joe Humfrey (inkle art director): spinning the globe or flipping through suitcases has "got to feel as nice as possible". *(snippet only)* — [VICE](https://vice.com/en/article/the-makers-of-mobile-hit-80-days-on-the-importance-of-amazing-ui-852)
- [Critic] Breath of the Wild's map is also an action surface in a lighter sense: fast travel is chosen from it and pins set on it steer play. — [Stamen](https://stamen.com/cartographers-play-video-games-a-review-of-the-map-in-the-legend-of-zelda-breath-of-the-wild/)
- [Designer] Outer Wilds shows a non-spatial board (cards and links) can be the planning interface from which the player decides where to go next. — [Mobius Digital blog](https://www.mobiusdigitalgames.com/news/a-look-into-playtests)

### Inferences
- Ingold's list is close to a requirements list for a map-as-main-screen in a text game: show progress, show the road not taken, let the player pick a destination, and let the journey line serve as undo.
- Sorcery!'s interior maps are a precedent for room-by-room exploration presented on a map with prose arriving per location, which is the same shape as a parser game's room graph.
- "Consequence without a notification" generalises: when the game state changes, changing the map visibly is a quieter and more trusted signal than a message.
- Tap-to-travel across already-explored rooms is the natural mobile action; the sourced examples suggest travel should be shown as movement along the drawn route so the player keeps their sense of distance and can be interrupted.
- 80 Days hides most of the route network until the player learns of routes; revelation by rumour or purchase is a self-revealing map driven by knowledge, not by footsteps (my reading of the game's reputation; see Gaps).

### Gaps
- Ingold's separate 80 Days design write-up ("Threats, luggage and mild peril", Game Developer) was found but not read; details of how routes are revealed on the globe are therefore unsourced.
- Nothing found on Heaven's Vault's map as interface.
- Unverified background: other map-first interfaces often cited are FTL's sector map, Slay the Spire's branching path map, Sunless Sea's chart that fills in as you sail, Citizen Sleeper's station view, and board-game-like node maps in Disco Elysium's fast travel. None was researched this session.
