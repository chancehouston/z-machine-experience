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
            let engine = try Engine(story: story, pack: pack, seed: seed)
            self.engine = engine
            show(await engine.snapshot(since: 0))
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
        show(await act(engine, showing))
        busy = false
    }

    private func show(_ snapshot: Snapshot?) {
        guard let snapshot else { return }
        if snapshot.generation != showing?.generation {
            turns = snapshot.turns
        } else {
            turns.append(contentsOf: snapshot.turns.filter { $0.number >= turns.count })
        }
        showing = snapshot
        if launchMillis == nil {
            launchMillis = LaunchClock.millisSinceProcessStart()
        }
    }
}
