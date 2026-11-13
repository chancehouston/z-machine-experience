import Foundation
import Observation

/// What the screen is showing: the turns so far and the newest snapshot.
/// It holds what the layer reported and decides nothing about the game.
@MainActor
@Observable
final class Game {
    /// Every turn of the game on screen, oldest first.
    private(set) var turns: [Turn] = []
    /// The newest snapshot; its stamp goes with every command.
    private(set) var showing: Snapshot?
    /// Set when no game could be started: the layer's message.
    private(set) var startFailure: String?
    /// A command is being worked out; nothing else is sent meanwhile.
    private(set) var busy = false
    /// Milliseconds from the start of the process to the first snapshot's
    /// being handed to the screen.
    private(set) var launchMillis: Int?
    /// What was last asked to be spoken, and how many times anything was:
    /// shown to tests only (`-speakAlways`).
    private(set) var lastSpoken: String?
    private(set) var timesSpoken = 0

    private var engine: Engine?

    /// Reads the story from the bundle and starts a game. `-seed N` fixes the
    /// game's randomness and `-story NAME` names another resource.
    func start() async {
        #if DEBUG
            // For tests of how a snapshot is drawn: a hand-made one, in place of
            // a game. Nothing can be sent, as there is no game to send it to.
            if let json = ProcessInfo.processInfo.environment["LAMPLIGHT_FIXTURE_SNAPSHOT"] {
                show(try? JSONDecoder().decode(Snapshot.self, from: Data(json.utf8)))
                return
            }
        #endif
        let defaults = UserDefaults.standard
        let name = defaults.string(forKey: "story") ?? "story"
        let seed = defaults.string(forKey: "seed").flatMap { UInt32($0) }
        guard let url = Bundle.main.url(forResource: name, withExtension: "z3"),
            let story = try? Data(contentsOf: url)
        else {
            startFailure = "The story file is missing."
            return
        }
        do {
            // The story's game pack, if the bundle holds one beside it. Its
            // text is handed over as it is; the app reads none of it.
            let pack = Bundle.main.url(forResource: name, withExtension: "pack")
                .flatMap { try? String(contentsOf: $0, encoding: .utf8) }
            // The game in progress, kept from the last launch. `-fresh` gives
            // it up first, so that a test starts from a new game.
            let keptFile = KeptFile(story: name)
            let arguments = ProcessInfo.processInfo.arguments
            if arguments.contains("-fresh") {
                keptFile?.remove()
            }
            #if DEBUG
                // For a test of a kept game that cannot be brought back.
                if arguments.contains("-damageKept") {
                    try? keptFile?.write(Data("damaged".utf8))
                }
            #endif
            let engine = try Engine(story: story, pack: pack, seed: seed, keptFile: keptFile)
            self.engine = engine
            show(await kept(engine, await engine.snapshot(since: 0), since: 0))
        } catch {
            startFailure = error.message
        }
    }

    func send(typed line: String) async {
        await command { engine, showing in await engine.send(typed: line, showing: showing) }
    }

    func tap(place: String) async {
        await command { engine, showing in await engine.tap(place: place, showing: showing) }
    }

    func tap(extra index: Int) async {
        await command { engine, showing in await engine.tap(extra: index, showing: showing) }
    }

    func newGame() async {
        await command { engine, _ in await engine.newGame() }
    }

    private func command(_ act: (Engine, Snapshot) async -> Snapshot?) async {
        guard !busy, let engine, let showing else { return }
        busy = true
        let snapshot = await act(engine, showing)
        // A new game's snapshot holds all its turns; a command's, the new ones.
        let since = snapshot?.generation == showing.generation ? showing.turnCount : 0
        show(await kept(engine, snapshot, since: since))
        busy = false
    }

    /// Has the game stored before its snapshot is shown, so that nothing the
    /// player has seen answered can be lost, and no other command is taken
    /// meanwhile. If it could not be stored, the snapshot shown is the one
    /// in which the layer says so: asked for again, with the turns from
    /// `since` as the first had them.
    private func kept(_ engine: Engine, _ snapshot: Snapshot?, since: Int) async -> Snapshot? {
        guard let snapshot, await !engine.keep() else { return snapshot }
        return await engine.snapshot(since: since) ?? snapshot
    }

    private func show(_ snapshot: Snapshot?) {
        guard let snapshot else { return }
        let before = turns.count
        let newGame = snapshot.generation != showing?.generation
        if newGame {
            turns = snapshot.turns
        } else {
            turns.append(contentsOf: snapshot.turns.filter { $0.number >= turns.count })
        }
        showing = snapshot
        // A new turn, or a new game, is spoken: the layer's text, as it is.
        if newGame || turns.count > before, let text = snapshot.announcement, Announcer.speaking {
            Announcer.post(text)
            lastSpoken = text
            timesSpoken += 1
        }
        if launchMillis == nil {
            launchMillis = LaunchClock.millisSinceProcessStart()
        }
    }
}
