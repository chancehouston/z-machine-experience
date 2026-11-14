# Automatic and in-game maps in text adventures and MUDs: prior art and lessons

Reading notes for the report writer:

- "Fetched" means I read the page itself (through a summarising fetch tool). "Snippet" means I only saw a search-result excerpt, so treat it as lower confidence.
- Facts and opinions are labelled. Opinions name the person.
- Several tools named in the brief (GUEmap, Zoom, Spatterlight, Gargoyle, Lectrote, Parchment, Frotz variants, TinTin++, MUSHclient) were not researched to a citable standard. They are listed under Gaps, not described from memory.

## 1. Which commercial text adventures shipped an on-screen map, and how did it look and behave?

### Takeaway
Four commercial lineages put a self-revealing map on screen between 1987 and 1993: Beyond Zork (character-graphics map permanently beside the text), Zork Zero (a separate MAP screen), Legend's games (map swapped in for the picture), and Wonderland (a map window with one-click travel). Every one showed only visited rooms, most allowed click-to-move, and even sympathetic critics say none fully replaced a hand-drawn map.

### Cited Findings

**Beyond Zork (Infocom, 1987)**
- Fact (fetched): the screen is split; the upper left is a window showing either the room description or the inventory, and the upper right holds a map that "draws itself and updates as you walk around." — [Digital Antiquarian, "Beyond Zork"](https://www.filfre.net/2015/11/beyond-zork/)
- Fact (fetched): rooms are boxes and connections are lines. The current room is highlighted and others are outlined. Opening a door adds a line to the next room. Dark rooms show a question mark. It shows the current room's exits and those of adjoining visited rooms, not unvisited rooms. — [Digital Antiquarian](https://www.filfre.net/2015/11/beyond-zork/)
- Fact (fetched): on machines with a mouse, the player can move by clicking the map. — [Digital Antiquarian](https://www.filfre.net/2015/11/beyond-zork/)
- Fact (fetched): typing MODE returns to a conventional text layout, but play is then "a bit more confusing than usual" because room descriptions no longer list exits. The prose had been written to lean on the map. — [Digital Antiquarian](https://www.filfre.net/2015/11/beyond-zork/)
- Fact (fetched): it was built on the version 5 Z-machine using character graphics modelled on the DEC VT-220's downloadable character set (lines, boxes, frames). The interpreter queried each machine's abilities and degraded: on the Apple II the map lost colour and fell back to rough ASCII. — [Digital Antiquarian](https://www.filfre.net/2015/11/beyond-zork/)
- Fact (snippet): the manual says the on-screen map shows only part of the geography and recommends still drawing your own; exits appear only once seen. — [Beyond Zork manual](https://www.mocagh.org/infocom/beyondzork-manual.pdf)
- Fact (snippet): the game reserved a large area at the top of the screen for the room description and local map. — [IFWiki, Beyond Zork](https://www.ifwiki.org/Beyond_Zork)
- Opinion (Jimmy Maher): the interface is "smart and playable and hugely convenient" and "a natural, elegant evolution"; he wishes later games had built on it. His complaints about the game are about puzzles and RPG mechanics, not the map. — [Digital Antiquarian](https://www.filfre.net/2015/11/beyond-zork/)
- Fact (fetched): parts of the game's geography are randomised on first entering an area, so the map could not be a fixed picture. — [Digital Antiquarian, comments](https://www.filfre.net/2015/11/beyond-zork/)
- Opinion (snippet, one retrospective writer): Beyond Zork's map was "crude" next to Zork Zero's. — [search excerpt of a Zork Zero review, everygamegoing.com](https://everygamegoing.com/larticle/Zork-Zero-000/43754)

**Zork Zero (Infocom, 1988)**
- Fact (snippet): a MAP command opens a separate map screen showing only locations already visited and only the general vicinity, not the whole game; a keypress returns to the story. With a mouse the player can move around from the map. — [Zork Zero manual](https://www.mocagh.org/infocom/zorkzero-manual.pdf)
- Opinion (snippet, 1989 C&VG review): the map is "a particularly good, mouse-sensitive example of its kind." — [everygamegoing.com review archive](https://everygamegoing.com/larticle/Zork-Zero-000/43754)
- Opinion (snippet, via Wikipedia's summary): Dave Arneson in Computer Gaming World praised the in-game map and help system. — [Wikiwand, Zork Zero](https://wikiwand.com/en/articles/Zork_Zero)

**Legend Entertainment (Spellcasting 101, 1990, onward; same engine for Timequest, Gateway, Eric the Unready)**
- Fact (fetched): three screen modes. The default has a verb menu of about 120 verbs, a noun list, a compass rose and an illustration. A second mode drops the menus for more text but keeps the compass rose and picture. A third is full-screen text. — [Digital Antiquarian, "A Time of Beginnings: Legend Entertainment"](https://www.filfre.net/2017/01/a-time-of-beginnings-legend-entertainment-or-bob-and-mikes-excellent-adventure-game-company/)
- Fact (fetched): the map is shown in place of the picture, not alongside it. Function keys also swap the picture for the room description or inventory without using a turn. — [Digital Antiquarian](https://www.filfre.net/2017/01/a-time-of-beginnings-legend-entertainment-or-bob-and-mikes-excellent-adventure-game-company/)
- Opinion (Maher): the automap is "handy" but "not always quite enough to replace pen and paper (or, today, Trizbort)." The default layout is "elaborate — not to say cluttered" and makes the games look "cluttered and ugly." He suspects most players used the menu-less mode. — [Digital Antiquarian](https://www.filfre.net/2017/01/a-time-of-beginnings-legend-entertainment-or-bob-and-mikes-excellent-adventure-game-company/)
- Opinion (Maher, in comments): on-screen maps help with large geographies, and inventory and room-description windows preserve conversational flow. — [Digital Antiquarian](https://www.filfre.net/2017/01/a-time-of-beginnings-legend-entertainment-or-bob-and-mikes-excellent-adventure-game-company/)
- Opinion (Steve Meretzky): the menus "were giveaways because they gave you a list of all possible verbs and all visible objects." This is about menus, but it is the same spoiler objection later raised against maps. — [Digital Antiquarian](https://www.filfre.net/2017/01/a-time-of-beginnings-legend-entertainment-or-bob-and-mikes-excellent-adventure-game-company/)
- Fact (snippet): Gateway and Eric the Unready use the same engine; the compass rose is clickable for movement. Later reviewers split between calling the interface excellent and calling it archaic for 1993. — [HardcoreGaming101, Gateway](https://www.hardcoregaming101.net/gateway/); [MobyGames reviews, Eric the Unready](https://www.mobygames.com/game/431/eric-the-unready/reviews/)

**Wonderland (Magnetic Scrolls, 1990)**
- Fact (fetched): the "Magnetic Windows" interface had separate, movable, resizable windows for text, picture, inventory icons, compass rose, hints, and "an auto-map (complete with one-click navigation to any location in the game's world)." — [Digital Antiquarian, "A Time of Endings, Part 4: Magnetic Scrolls"](https://www.filfre.net/2017/01/a-time-of-endings-part-4-magnetic-scrolls/)
- Fact (fetched): it ran responsively only on a high-end 386 and "ran like a dog" on an Amiga 500. — [Digital Antiquarian](https://www.filfre.net/2017/01/a-time-of-endings-part-4-magnetic-scrolls/)
- Fact (snippet, contemporary review): on an Atari 520ST only the text and graphics windows were available, so the map was missing on low-end machines. — [everygamegoing.com, Wonderland review](https://www.everygamegoing.com/larticle/wonderland-000/48975)
- Opinion (snippet, same contemporary review): the map and click-to-travel cut out "much tedious typing," but players could get absorbed in arranging windows instead of playing. — [everygamegoing.com](https://www.everygamegoing.com/larticle/wonderland-000/48975)
- Opinion (Maher): "very impressive the first time you fired it up," then "a little superfluous"; it "did very little to truly improve the playability of text adventures"; "lipstick on the same old textual swine." — [Digital Antiquarian](https://www.filfre.net/2017/01/a-time-of-endings-part-4-magnetic-scrolls/)
- Fact (snippet): contemporary scores were high (CU Amiga 96%, Zero 95/100); a later retrospective scored it 44 and said it was not special "except for the fancy interface." — [Wikipedia, Wonderland](https://en.wikipedia.org/wiki/Wonderland_(video_game)); [The Adventure Gamer, Missed Classics 6](https://advgamer.blogspot.ca/2015/02/missed-classics-6-wonderland.html)

**Later commercial and near-commercial examples**
- Fact (snippet): Hadean Lands (Andrew Plotkin, 2014) on iOS lists "a built-in game map" that lets the player tap to go directly to any explored room. One App Store reviewer said the map works, which they had not expected; another said the interface does not cross over well to phones. — [App Store, Hadean Lands](https://apps.apple.com/us/app/hadean-lands/id918958300)
- Fact (snippet, attributed to Emily Short's own notes on the game): Counterfeit Monkey has a compass rose that shades exits differently depending on whether the room in that direction has been visited, and it is hyperlinked so the player can click instead of typing. — [Emily Short, "Counterfeit Monkey"](https://emshort.blog/2012/12/31/counterfeit-monkey/); [Counterfeit Monkey source](https://tass.neocities.org/source/source_3.html)
- Opinion (HanonO, forum): Counterfeit Monkey's map is "wonderful" and a model for a map that appears as the player explores. — [intfiction.org, "What compassless games are missing"](https://intfiction.org/t/what-compassless-games-are-missing/13372?page=2)
- Fact as reported by forum posters: ADRIFT and Quest have built-in automaps; ADRIFT's lets the player click a room to go there. — [intfiction.org, compassless thread](https://intfiction.org/t/what-compassless-games-are-missing/13372?page=2); [intfiction.org, "Coders, how would you implement a tool to automap"](https://intfiction.org/t/coders-how-would-you-implement-a-tool-to-automap-as-you-play-i-f/48090)
- Opinion (reader comment): the Strand Games remake of The Pawn is praised for its automap and compass rose. — [Digital Antiquarian comments](https://www.filfre.net/2017/01/a-time-of-beginnings-legend-entertainment-or-bob-and-mikes-excellent-adventure-game-company/)

### Inferences
- Every commercial map got its data from the game itself, written by the author. None inferred the map from text. That is why they were reliable, and why they only ever existed per game.
- The designs converge on the same five rules: reveal only what has been visited; show exits as stubs before the far room is known; mark the current room strongly; mark dark rooms as unknown, not absent; let a tap or click on a known room travel there.
- Nobody made the map the main thing on screen. Beyond Zork gave it roughly a quarter of the screen; Zork Zero and Legend made it a mode that hid something else; Wonderland made it one window of many. A map-first phone layout has no direct commercial precedent in this genre.
- Beyond Zork shows a cost of leaning on the map: once the prose stops listing exits, the text alone is worse. A map-first app playing unmodified story files does not have this problem, since the prose is untouched, but it does mean exits the prose never mentions cannot be assumed.
- Both Infocom maps showed only the "general vicinity." Local views sidestep the hardest layout problem (fitting a whole irregular world on one plane).
- The Wonderland and Legend criticism is about clutter and speed, not about the map. The map is the one part of those interfaces that critics consistently call handy.

### Gaps
- Arthur, Shogun and Journey (Infocom's other graphical games) were not checked for map features.
- Legend's map was not described in detail in any source I could read (the Eric the Unready manual is a scanned PDF with no text layer). I do not know whether it was click-to-travel or how it handled up/down.
- I found no contemporary review of Beyond Zork that discusses the map at length; the COMPUTE! review exists ([atarimagazines.com](https://atarimagazines.com/compute/issue95/021_1_REVIEWS_BEYOND_ZORK.php)) but I did not read it.
- How Hadean Lands' and Counterfeit Monkey's maps are drawn (hand-drawn art with reveal versus generated) was not confirmed from a primary source.
- Other candidates not researched: Level 9, Synapse, Telarium, Thaumistry (2017).

## 2. Which interpreters and tools draw a map automatically, where do they get their information, and how reliable is it?

### Takeaway
There are four information sources, in rising order of reliability: scraping the transcript (Trizbort, IFMapper, scr2ifm), reading the story's memory for the location variable (nitfol), having the game emit structured data each turn (Strand Games IFI, MUD protocols), and having the author ship the map (in-game maps, proposed Blorb map resources). Transcript scraping works on any game but its own authors call it imprecise; memory reading is exact about "which room" but needs per-game setup; author data is exact but only exists for games written for it.

### Cited Findings

**Transcript scrapers**
- Fact: Trizbort builds its map from a transcript file the interpreter writes (the player types SCRIPT and points Trizbort at the file), so it works with most interpreters. — [intfiction.org, "Is there an automapper for playing Z5 games?"](https://intfiction.org/t/is-there-an-automapper-for-playing-z5-z-code-games/1894); [intfiction.org, coders thread](https://intfiction.org/t/coders-how-would-you-implement-a-tool-to-automap-as-you-play-i-f/48090)
- Opinion from the tool's author (Genstein): "Automapping isn't a precise science so automapping tools can make mistakes"; tools "have trouble with mazes"; Trizbort "can't map certain 'unconventional' games." — [intfiction.org](https://intfiction.org/t/is-there-an-automapper-for-playing-z5-z-code-games/1894)
- User reports: Denk found Trizbort's automap on a complicated map needed constant manual adjustment. A user reported (snippet) that it struggles when a room name is in a non-standard format, and that capitalisation alone can cause trouble. Another reported that it did not pick up new transcript text live and had to be restarted. — [intfiction.org, coders thread](https://intfiction.org/t/coders-how-would-you-implement-a-tool-to-automap-as-you-play-i-f/48090); [intfiction.org, "Trizbort is now open source"](https://intfiction.org/t/trizbort-is-now-open-source/5147/17); [intfiction.org, "Trizbort v1.5.9.6"](https://intfiction.org/t/trizbort-v1-5-9-6/10991)
- Opinion (Peter Piers): Trizbort is "much friendlier in disambiguating between rooms" than IFMapper, which also has a built-in automapper and is "almost as good." Opinion (Stian): IFMapper is buggy and unmaintained. — [intfiction.org](https://intfiction.org/t/is-there-an-automapper-for-playing-z5-z-code-games/1894); [intfiction.org, coders thread](https://intfiction.org/t/coders-how-would-you-implement-a-tool-to-automap-as-you-play-i-f/48090)
- Fact: trizbort.io (the JavaScript version) has no transcript parsing. Trizbort itself is C# and Windows-only, run elsewhere through Wine. — [intfiction.org, coders thread](https://intfiction.org/t/coders-how-would-you-implement-a-tool-to-automap-as-you-play-i-f/48090)
- Fact (scr2ifm, the best-documented scraper): commands are found by the `>` prompt. A room change is a short title-like line after a command: lines with characters that never appear in titles are rejected, then word count and capitalisation are checked, with a default maximum of 8 words. — [IFM manual, scr2ifm](https://ifm.readthedocs.io/en/latest/scr2ifm.html)
- Fact (scr2ifm): deciding whether a room was seen before is described as the hardest step. It tries an exact description match, then a substring match, then the first few words; with no description it falls back to name only, which "isn't as reliable." The manual therefore tells the player to use VERBOSE mode. — [IFM manual](https://ifm.readthedocs.io/en/latest/scr2ifm.html)
- Fact (scr2ifm limits): rooms with the same name and description "will simply appear as a single room"; one-way links "aren't detected, so you'll have to add the appropriate IFM attribute yourself"; teleport words such as XYZZY are not detected and produce a bad link that must be hidden by hand; walking into the dark makes "Darkness" appear as a room; only one level of UNDO is assumed; a bend in a passage is only known if the player walks back. Fixes go in a separate command file. — [IFM manual](https://ifm.readthedocs.io/en/latest/scr2ifm.html)
- Opinion (mawg): scraping "go north" is easy, but movement can happen in "limitless ways," the location name may not come first, and locations can be renamed mid-game. — [intfiction.org, coders thread](https://intfiction.org/t/coders-how-would-you-implement-a-tool-to-automap-as-you-play-i-f/48090)

**Memory readers**
- Fact: nitfol (a Z-machine interpreter, unmaintained since 2008) draws an ASCII map in its own window. The user must give it an expression for the current location, for example `/automap (global) 15`. In Z-machine versions 1 to 3 the location is always global 0. In later versions the user must hunt for it with debugger commands. — [nitfol manual](https://0v9xf534pt.unbox.ifarchive.org/0v9xf534pt/nitfol.html); [IFWiki, Nitfol](https://www.ifwiki.org/Nitfol)
- Fact: nitfol discovers exits by silently trying every direction from the current room, recording where each leads, and undoing each move, with output switched off. — [nitfol manual](https://0v9xf534pt.unbox.ifarchive.org/0v9xf534pt/nitfol.html)
- Fact: nitfol's own manual lists its failures: random destinations (the forest in Advent) are handled poorly; several places coded as one room (the long road in Enchanter) are not handled; exits that go nowhere but change game state are not shown; teleports confuse it; a move that kills the player is treated as leading to the reincarnation room; some games keep the location in several globals and the choice changes behaviour in the dark; it is slow and "still very buggy"; and "the end user must do a lot of setup work." — [nitfol manual](https://0v9xf534pt.unbox.ifarchive.org/0v9xf534pt/nitfol.html)
- User report (Peter Piers): "never got it to work." — [intfiction.org](https://intfiction.org/t/is-there-an-automapper-for-playing-z5-z-code-games/1894)
- Proposal (Andrew Plotkin, "zarf"): read the game's `location` variable at a known memory address. The compiler's debug file "includes the memory addresses of all variables and objects," so this works for Inform 6 and 7 games on Z-code and Glulx when that file exists. — [intfiction.org, coders thread p.2](https://intfiction.org/t/coders-how-would-you-implement-a-tool-to-automap-as-you-play-i-f/48090?page=2)
- Fact (zarf): interpreters cannot simply hook "object moved" events, because Z-code and Glulx have no standard routines for that at the level the interpreter sees; "each game implements them separately." — [intfiction.org, coders thread](https://intfiction.org/t/coders-how-would-you-implement-a-tool-to-automap-as-you-play-i-f/48090)

**Game emits data each turn**
- Fact: Strand Games' IFI front end draws its map from "json messages sent after every turn"; the game engine must be built to send them. — [intfiction.org, coders thread](https://intfiction.org/t/coders-how-would-you-implement-a-tool-to-automap-as-you-play-i-f/48090)

**Author ships the map**
- Proposal (zarf): a Blorb resource holding a simple map plus a table from location value to room ID and display name, because "many rooms could be named 'Maze'." He would keep it deliberately simple: "I would not go down the road of adding features to the format to support every possible game concept." — [intfiction.org, coders thread p.2](https://intfiction.org/t/coders-how-would-you-implement-a-tool-to-automap-as-you-play-i-f/48090?page=2)
- Proposal (Denk): an author-exported map file with grid positions and connections, combined with play to show only visited rooms; awkward links (XYZZY) are handled by the author leaving them off. Denk also thinks zarf's simple format "will be too simple for most games." — [intfiction.org, coders thread](https://intfiction.org/t/coders-how-would-you-implement-a-tool-to-automap-as-you-play-i-f/48090?page=2)
- Fact as reported by its builder (Mike_G): a partial Z-machine implementation with a map image and metadata in custom Blorb chunks; it reveals rooms and connections including secret and one-way ones, marks the current room, and saves reveal state in a custom chunk of the save file. — [intfiction.org, coders thread p.2](https://intfiction.org/t/coders-how-would-you-implement-a-tool-to-automap-as-you-play-i-f/48090?page=2)
- Fact: Mark Tilford's "Automap" extension for Inform 7 is compiled into the game, so it reads the game's own room connections. A "map view" command lists visited rooms with coordinates relative to the current room. It can draw with the Beyond Zork character set, which some interpreters do not support. — [intfiction.org, "Automap by Mark Tilford"](https://intfiction.org/t/automap-by-mark-tilford/9201)
- User reports on that extension: it broke across Inform versions; one author saw only the start room mapped; another saw diagonal lines "where there shouldn't be any lines at all"; Peter Piers had earlier found it "invaluable." — [intfiction.org](https://intfiction.org/t/automap-by-mark-tilford/9201)
- Fact: Inform 7's own map (the Index "World" tab) is an authoring aid, not available to players. — [intfiction.org](https://intfiction.org/t/is-there-an-automapper-for-playing-z5-z-code-games/1894)

**The summary judgement from inside the community**
- Opinion (zarf): "Interpreters should do mapping, and also it's impossible for interpreters to do mapping." — [intfiction.org, coders thread p.2](https://intfiction.org/t/coders-how-would-you-implement-a-tool-to-automap-as-you-play-i-f/48090?page=2)

### Inferences
- An interpreter that owns the Z-machine has an advantage none of the transcript tools have: it can read the location variable directly, exactly, every turn. This turns "is this the same room?" from a text-matching guess into an integer comparison, which removes the largest single source of error in Trizbort and scr2ifm (identical names, changed names, darkness, brief mode).
- The remaining cost is finding that variable. Nitfol left it to the user and that is the stated reason almost nobody used it. Versions 1 to 3 are free (global 0). For later versions a per-game table (the "game packs" idea) or automatic detection is needed; nitfol's author wanted auto-detection and never built it.
- Knowing the room identity does not give layout. Where to draw a room still comes from the direction the player typed, and that is where non-compass moves, teleports and non-reciprocal exits still bite.
- Nitfol's probe-every-exit-and-undo trick gives complete exit knowledge without the player walking, but it reveals exits the player has not found, costs time, and misfires on random exits, deaths and state-changing moves. For a self-revealing map it is more useful as an offline step for building game packs than as live behaviour.
- Every scraper ends up needing a manual correction layer (scr2ifm's command file, Trizbort's editor). A design with no way to fix or override the map is betting on perfect inference, which nobody has achieved.
- Status-line scraping was not discussed in any source I read. It is a plausible fallback source for the room name but I have no evidence about anyone relying on it.

### Gaps
- GUEmap, Zoom, Spatterlight, Gargoyle, Lectrote, Parchment and the Frotz variants: I found no evidence in this pass that any of them automaps, and did not find a source saying they do not. Needs a direct check before the report asserts either.
- Trizbort's actual matching rules (how it uses names versus descriptions, how it handles regions) were not read from its documentation or source; only its author's and users' statements are cited. Repository: [JasonLautzenheiser/trizbort](https://github.com/JasonLautzenheiser/trizbort).
- IFMapper's automap method was not read from a primary source.
- jszip was mentioned by one poster as having automap support; not verified.
- Research frameworks that read location from the Z-machine object tree for AI agents (for example Jericho) are relevant precedent for memory reading but were not researched here.
- How the Hadean Lands map is fed (the app is an interpreter bundled with one game) was not confirmed.

## 3. How do MUD clients do automapping, and what is considered solved and unsolved?

### Takeaway
MUD mapping became dependable only where the server sends structured room data (room ID, name, exits) out of band; clients that infer rooms from text have the same failures as IF transcript scrapers and need per-game trigger tuning. Pathfinding, click-to-walk, areas, special exits and manual editing are treated as solved; text-only room identification, mazes and automatic layout are not.

### Cited Findings
- Fact: Mudlet's mapper is two parts: a display, storage and pathfinding layer, and a game-specific script that tracks location, which the manual calls "very game-specific." — [Mudlet manual, Mapper](https://wiki.mudlet.org/w/Manual:Mapper)
- Fact: a room is drawn only once it has a unique integer ID, an area and integer x, y, z coordinates. Areas are separate planes; a city that is one room on a wilderness map can be its own area. — [Mudlet manual](https://wiki.mudlet.org/w/Manual:Mapper)
- Fact: "special exits" are stored as command strings (for example "jump cliff") with no spatial direction; they are not drawn unless the user adds custom exit lines, which are "purely visual tools." Stub exits draw a direction before the far room is known. Rooms and exits carry weights and locks for pathfinding. There is a one-way link symbol. — [Mudlet manual](https://wiki.mudlet.org/w/Manual:Mapper)
- Fact: darkness and blindness can break room matching; Mudlet provides a vision-fail event so the script can move the marker anyway. — [Mudlet manual](https://wiki.mudlet.org/w/Manual:Mapper)
- Fact (snippet): Mudlet's generic mapper script works from triggers on text and needs two things, the room name and the exits; the IRE mapper script uses GMCP data from the server instead. — [Mudlet forums](https://forums.mudlet.org/viewtopic.php?p=45628); [Mudlet wiki, Generic mapper package](https://cnwiki.mudlet.org/w/Generic_mapper_package)
- User reports (snippet): one project, "after weeks of struggling with text-based detection," switched to GMCP room IDs, which stopped duplicate rooms being created. A MUD developer described the "tricky things that come with only being able to work with room names" and planned to send coordinates. — [Mudlet forums](https://forums.mudlet.org/viewtopic.php?p=46753); [Mudlet forums](https://forums.mudlet.org/viewtopic.php?p=46767)
- Fact (snippet): server room IDs that are hashes or very large numbers must be remapped to small sequential IDs for pathfinding to work. — [Mudlet wiki](https://wiki.mudlet.org/w/Manual:Mapper)
- Fact (snippet, vendor documentation): zMUD decides whether a room was visited before from the room text the MUD sends, and "occasionally" puts a second room on top of an existing one; the help says most MUD areas are designed to be hard to map, so the automapper will not always be right. — [zMUD help, "Creating maps"](https://zuggsoft.com/zmud/help6/Crea0191.htm)
- Fact (fetched): zMUD keeps a "Direction Queue" of moves sent but not yet confirmed. A failed move must be removed by a trigger (`#NODIR` on "The door is closed") or the mapper creates a room that is not there. `#OK` confirms a move when no room description arrives, as in darkness. `#NOMAP` hides lines that confuse it. — [zMUD help, "Advanced Mapping"](https://zuggsoft.com/zmud/help6/Adva0177.htm)
- Fact (fetched): zMUD handles non-compass exits by letting the user type `>enter`, and `>enter>e` to say "draw the new room to the east." For mazes it recommends editing the link table directly and giving look-alike rooms unique labels, such as an object dropped there. — [zMUD help, "Advanced Mapping"](https://zuggsoft.com/zmud/help6/Adva0177.htm)
- User reports (snippet): on IRE games zMUD mapped reliably only when room names had a unique colour to match on, or with server protocols (MXP, ATCP) under CMUD; "getting everything reliable is still a hassle." — [Lusternia forums](https://www.lusternia.com/archive/forums/thread/28313); [Lusternia forums](https://lusternia.com/archive/forums/thread/13239)

### Inferences
- The MUD world's answer to "which room am I in" is a room ID from the server. The single-player equivalent is the location variable in the story's memory. The MUD record suggests that once this exists, most of the remaining work is drawing and editing, not detection.
- Solved in practice: storage of rooms with stable IDs, stub exits, areas as separate planes, special exits as labelled commands, weighted pathfinding and click-to-walk, and user editing of everything.
- Unsolved in practice: identifying rooms from text alone; mazes; automatic placement that does not overlap (both Mudlet and zMUD place the new room one step in the direction moved and leave collisions to the user); one-way detection.
- zMUD's direction queue is a direct lesson: a map must only commit a move after confirming that the location actually changed. With memory access this is trivial (compare the location before and after the turn) and it also handles failed moves, darkness and delayed descriptions.
- Mudlet's model of a non-spatial exit (a command string that is in the graph but not drawn as a direction) is the cleanest existing treatment of teleports, magic words and "enter"/"jump" moves.
- Areas are the MUD answer to "does not fit on a grid": do not try to fit the whole world in one plane.

### Gaps
- TinTin++ `#map` and MUSHclient mappers were not researched; no findings.
- I found no single community statement of "what is solved and unsolved"; the split above is my inference from manuals and forum complaints.
- Much of the MUD forum material is 10 to 20 years old.

## 4. What are the classic hard problems, and how has each been handled?

### Takeaway
Nearly every hard case is one of two problems: identity (is this the same room?) and layout (where do I draw it?). Reading room identity from the game solves the first almost completely; the second has no automatic solution in any tool and has been handled by local views, separate areas, labelled non-spatial links, stretched or bent lines, and manual correction.

### Cited Findings
- Mazes with identical names: scr2ifm "will assume there's just a single room"; Genstein says automappers "have trouble with mazes"; zMUD recommends manual link editing and unique labels such as a dropped object. The Colossal Cave maze, whose rooms have distinct descriptions, maps fine in scr2ifm. — [IFM manual](https://ifm.readthedocs.io/en/latest/scr2ifm.html); [intfiction.org](https://intfiction.org/t/is-there-an-automapper-for-playing-z5-z-code-games/1894); [zMUD help](https://zuggsoft.com/zmud/help6/Adva0177.htm)
- Mazes, design view: Garry Francis (Warrigal) says ADVENT's mazes could not be drawn automatically; Denk says authors may want maze connections kept hidden because working them out is the puzzle. — [intfiction.org, coders thread p.2](https://intfiction.org/t/coders-how-would-you-implement-a-tool-to-automap-as-you-play-i-f/48090?page=2)
- Same name, different rooms: zarf's answer is to key rooms by ID and treat the name as display text only. — [intfiction.org, coders thread p.2](https://intfiction.org/t/coders-how-would-you-implement-a-tool-to-automap-as-you-play-i-f/48090?page=2)
- Same room, different names, and renaming: mawg notes locations can be renamed mid-game; zarf mentions rooms whose names change and several locations held in one object; nitfol fails on several places coded as one room (Enchanter's long road). — [intfiction.org, coders thread](https://intfiction.org/t/coders-how-would-you-implement-a-tool-to-automap-as-you-play-i-f/48090?page=2); [nitfol manual](https://0v9xf534pt.unbox.ifarchive.org/0v9xf534pt/nitfol.html)
- Non-reciprocal exits and bends: scr2ifm only learns a bend if the player walks the return route. Nitfol merges multiple exits between two rooms into one two-way passage where it can, and planned but never built "extra bends." — [IFM manual](https://ifm.readthedocs.io/en/latest/scr2ifm.html); [nitfol manual](https://0v9xf534pt.unbox.ifarchive.org/0v9xf534pt/nitfol.html)
- One-way passages: not detected by scr2ifm. Nitfol detects them because it probes the return direction, and draws arrowheads. Mudlet has a one-way symbol. tundish's example: jumping off a waterfall into a pool with no way back. — [IFM manual](https://ifm.readthedocs.io/en/latest/scr2ifm.html); [nitfol manual](https://0v9xf534pt.unbox.ifarchive.org/0v9xf534pt/nitfol.html); [Mudlet manual](https://wiki.mudlet.org/w/Manual:Mapper); [intfiction.org, coders thread p.2](https://intfiction.org/t/coders-how-would-you-implement-a-tool-to-automap-as-you-play-i-f/48090?page=2)
- Up/down and in/out: scr2ifm assigns such a move a free compass direction on the page and records the real command as a label. Nitfol draws only the current floor and marks stairs with u, d or b in the room. Mudlet gives rooms a z coordinate. zMUD lets the user say which way to draw a non-compass exit. — [IFM manual](https://ifm.readthedocs.io/en/latest/scr2ifm.html); [nitfol manual](https://0v9xf534pt.unbox.ifarchive.org/0v9xf534pt/nitfol.html); [Mudlet manual](https://wiki.mudlet.org/w/Manual:Mapper); [zMUD help](https://zuggsoft.com/zmud/help6/Adva0177.htm)
- Unusual direction sets: Sea Coral mixes nautical directions, compass points, enter/exit/board and up/down; mawg's proposal is to label each connection with the verb actually used ("Plugh, xyzzy, y2"). — [intfiction.org, coders thread p.2](https://intfiction.org/t/coders-how-would-you-implement-a-tool-to-automap-as-you-play-i-f/48090?page=2)
- Rooms that cannot sit on a grid: nitfol lengthens passages by one unit, then two, to remove overlaps, then gives up; the manual says this "isn't perfect." Mudlet splits the world into areas and uses custom lines. scr2ifm starts a new map section and "joins" sections. Beyond Zork and Zork Zero show only the local vicinity. — [nitfol manual](https://0v9xf534pt.unbox.ifarchive.org/0v9xf534pt/nitfol.html); [Mudlet manual](https://wiki.mudlet.org/w/Manual:Mapper); [IFM manual](https://ifm.readthedocs.io/en/latest/scr2ifm.html); [Zork Zero manual](https://www.mocagh.org/infocom/zorkzero-manual.pdf)
- Teleports: confuse nitfol; produce a false link in scr2ifm that must be hidden by hand; Mudlet stores them as special exits that are not drawn spatially; RESTART and RESTORE are treated as teleports by scr2ifm. — [nitfol manual](https://0v9xf534pt.unbox.ifarchive.org/0v9xf534pt/nitfol.html); [IFM manual](https://ifm.readthedocs.io/en/latest/scr2ifm.html); [Mudlet manual](https://wiki.mudlet.org/w/Manual:Mapper)
- UNDO, RESTORE, death: scr2ifm assumes one level of undo; nitfol treats a fatal move as leading to the reincarnation room. — [IFM manual](https://ifm.readthedocs.io/en/latest/scr2ifm.html); [nitfol manual](https://0v9xf534pt.unbox.ifarchive.org/0v9xf534pt/nitfol.html)
- Rooms and links that change: mawg notes an automapper cannot know a bridge collapsed behind the player and suggests user annotations; tundish calls exits "directional and mutable"; Beyond Zork adds a line when a door is opened and randomises some layouts on first entry; nitfol does badly with random destinations. — [intfiction.org, coders thread p.2](https://intfiction.org/t/coders-how-would-you-implement-a-tool-to-automap-as-you-play-i-f/48090?page=2); [Digital Antiquarian](https://www.filfre.net/2015/11/beyond-zork/); [nitfol manual](https://0v9xf534pt.unbox.ifarchive.org/0v9xf534pt/nitfol.html)
- Darkness: Beyond Zork draws a question mark. scr2ifm records a false room called "Darkness." zMUD needs a trigger to confirm the move. Mudlet has a vision-fail event. In nitfol, which global is chosen as the location changes what happens in the dark. — [Digital Antiquarian](https://www.filfre.net/2015/11/beyond-zork/); [IFM manual](https://ifm.readthedocs.io/en/latest/scr2ifm.html); [zMUD help](https://zuggsoft.com/zmud/help6/Adva0177.htm); [Mudlet manual](https://wiki.mudlet.org/w/Manual:Mapper); [nitfol manual](https://0v9xf534pt.unbox.ifarchive.org/0v9xf534pt/nitfol.html)
- Failed moves: zMUD creates phantom rooms unless a trigger cancels the queued direction. — [zMUD help](https://zuggsoft.com/zmud/help6/Adva0177.htm)
- Brief mode: name-only matching is less reliable, so scr2ifm requires VERBOSE. — [IFM manual](https://ifm.readthedocs.io/en/latest/scr2ifm.html)

### Inferences
- With the location read from memory: identical maze rooms are distinct objects and so distinct on the map; renamed rooms keep their identity; failed moves are simply "location unchanged"; teleports and magic words are "location changed after a non-direction command" and can be drawn as a labelled, non-spatial link in the Mudlet style.
- Memory reading has its own identity failures, both documented by nitfol: one object standing for many places, and darkness. In Inform games the location variable points at a special darkness object while the player is in the dark, so the map should show "somewhere dark" (Beyond Zork's question mark) and resolve it when light arrives, not create a room. This last point is my inference from the nitfol note about globals and darkness; I did not find it stated outright.
- Drawing every maze room distinctly solves the maze for the player. That is a design decision with a spoiler cost (see section 5), not only a technical win.
- The most reusable layout rule across tools: place a new room one step in the direction moved; if the cell is taken, do not silently overlap (the zMUD failure). Lengthen the link, bend it, or start a new area.
- Up/down has three known treatments (separate floors, a free compass slot with a label, an in-room marker). On a phone, floors with an in-room stair marker (nitfol) keep each plane simple.
- A local view centred on the player, as Infocom used, avoids ever having to make the whole world consistent on one plane, and suits a small screen.

### Gaps
- Rooms that physically move (vehicles, lifts, ships) were not discussed in any source read.
- I found no worked solution anywhere for good automatic layout of an irregular IF map; nitfol's passage-stretching is the only algorithm described, and its author calls it imperfect.
- I did not find Emily Short's or Jimmy Maher's writing specifically on maze mapping in this pass, though Short's geography article exists: [emshort.blog](https://emshort.blog/how-to-play/writing-if/my-articles/geography/).

## 5. What do players and authors say for and against automaps?

### Takeaway
Opinion is split along familiar lines, with more people wanting a built-in map than opposing one; the objections are that drawing the map is part of playing, that a map can spoil discovery and mazes, and that it cannot cope with deliberately strange geography. The one point of near-agreement is that a map should show only what the player has found.

### Cited Findings

**For**
- Opinion (John Phoenix): wants a built-in mapper so he need not "pause to go to the map software" and enter rooms by hand; external tools are "nice" but "don't really work as I want them to." — [intfiction.org](https://intfiction.org/t/is-there-an-automapper-for-playing-z5-z-code-games/1894)
- Opinion (zarf): people now play on phones and couches, where a paper map is impractical; hence "Interpreters should do mapping." — [intfiction.org, coders thread p.2](https://intfiction.org/t/coders-how-would-you-implement-a-tool-to-automap-as-you-play-i-f/48090?page=2)
- Opinion (Maher): Beyond Zork's map is "hugely convenient"; on-screen maps help with large geographies. — [Digital Antiquarian](https://www.filfre.net/2015/11/beyond-zork/); [Digital Antiquarian](https://www.filfre.net/2017/01/a-time-of-beginnings-legend-entertainment-or-bob-and-mikes-excellent-adventure-game-company/)
- Opinion (J. J. Guest): "We all have a mental map of the places we visit"; he found a compassless game hard because he could not form one. He suggests click-to-move maps and GO TO commands. — [intfiction.org, compassless thread](https://intfiction.org/t/what-compassless-games-are-missing/13372?page=2)
- Opinion (Pseudavid): compass directions hurt accessibility and should be a last resort; proposes a "go to" that walks the player along the path. — [intfiction.org, compassless thread](https://intfiction.org/t/what-compassless-games-are-missing/13372?page=2)
- Opinion (contemporary Wonderland review, snippet): the map with click-to-travel cuts out "much tedious typing." — [everygamegoing.com](https://www.everygamegoing.com/larticle/wonderland-000/48975)

**Against, or with reservations**
- Opinion (Garry Francis, "Warrigal"): he would not bother with an automapper; drawing maps is "part of the adventure experience." He also notes that many game maps are very hard to draw, and that tools would push authors toward maps that "make sense." — [intfiction.org, coders thread](https://intfiction.org/t/coders-how-would-you-implement-a-tool-to-automap-as-you-play-i-f/48090)
- Opinion (inventor200): some authors enjoy "really messing with room and connection models," which a standard map cannot represent. — [intfiction.org, coders thread](https://intfiction.org/t/coders-how-would-you-implement-a-tool-to-automap-as-you-play-i-f/48090)
- Opinion (Ruber Eaglenest): a full map can spoil discovery where the protagonist is learning the place; a map suits games where the protagonist already knows it. He suggests fog of war, incomplete floor plans and automaps as middle paths, and says dream-like works may need no orientation at all. — [intfiction.org, compassless thread](https://intfiction.org/t/what-compassless-games-are-missing/13372?page=2)
- Opinion (Denk): authors may want maze connections hidden as part of the puzzle. — [intfiction.org, coders thread p.2](https://intfiction.org/t/coders-how-would-you-implement-a-tool-to-automap-as-you-play-i-f/48090?page=2)
- Opinion (Maher): even a good automap is "not always quite enough to replace pen and paper"; and of Wonderland's whole apparatus, "a little superfluous." — [Digital Antiquarian](https://www.filfre.net/2017/01/a-time-of-beginnings-legend-entertainment-or-bob-and-mikes-excellent-adventure-game-company/); [Digital Antiquarian](https://www.filfre.net/2017/01/a-time-of-endings-part-4-magnetic-scrolls/)
- Fact: Infocom's own Beyond Zork manual still told players to draw a map (snippet). — [Beyond Zork manual](https://www.mocagh.org/infocom/beyondzork-manual.pdf)
- Opinion (ramstrong): simple maps suit small games, not big ones; ASCII maps are "not screen reader friendly"; parser games on phones are "painful." — [intfiction.org, coders thread p.2](https://intfiction.org/t/coders-how-would-you-implement-a-tool-to-automap-as-you-play-i-f/48090?page=2)

**On how much to reveal**
- Opinion (mawg): an automapper should "only reveal what the player has already discovered." zarf: showing the full map "will be a player preference." Mike_G: trying to prevent full reveal is futile, since players will get around it. Denk: visited-only should be the default. — [intfiction.org, coders thread p.2](https://intfiction.org/t/coders-how-would-you-implement-a-tool-to-automap-as-you-play-i-f/48090?page=2)

### Inferences
- The "mapping is part of the game" view is real but was a minority in the threads read. Those threads are small and self-selected (people who opened a thread about automappers), so this is weak evidence about players in general.
- The objections point at specific design answers, not at dropping the map: reveal on visit only; do not show exits the player has not been told about; consider treating mazes specially (for example leaving maze links undrawn or letting the player turn the map off) so the map does not solve the puzzle; let the player annotate and correct.
- Accessibility matters for this project in particular, since it already ships screen reader support: a map that is the main thing on screen needs a non-visual equivalent (a room list, GO TO by name), which several posters propose independently.
- A map that carries exits is implicitly a hint system about where the player has not yet been (stub exits). The historical maps all did this and nobody in the sources objects to it.

### Gaps
- I did not find the rec.arts.int-fiction archive discussions on mapping, nor a piece by Emily Short arguing for or against automaps. A search aimed at intfiction.org for "draw maps by hand" returned nothing relevant.
- No survey data on how many players map by hand, use Trizbort, or use nothing.
- No source read discusses player reaction to a map that reveals maze structure automatically; the spoiler concern is asserted, not measured.
- Jimmy Maher's views on mazes and hand-mapping in his Zork and Adventure articles were not retrieved.
