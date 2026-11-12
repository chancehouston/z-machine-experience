import Foundation
import ZMX

/// Why a game could not be started, in the layer's words.
struct StartFailure: Error {
    let message: String
}

/// The one owner of the session. Everything that crosses into the experience
/// layer crosses here, and only one call at a time: it is an actor.
actor Engine {
    private let session: OpaquePointer

    /// Loads the story and runs it to its first stop. With no seed, the game's
    /// randomness comes from the system's generator.
    init(story: Data, seed: UInt32?) throws(StartFailure) {
        var generator = SystemRandomNumberGenerator()
        let entropy = UInt32.random(in: .min ... .max, using: &generator)
        var error: UnsafeMutablePointer<CChar>?
        let made = story.withUnsafeBytes { bytes in
            zmx_session_new(
                bytes.bindMemory(to: UInt8.self).baseAddress, bytes.count,
                seed ?? 0, seed == nil ? 0 : 1, entropy, &error)
        }
        guard let made else {
            let message = error.map { String(cString: $0) } ?? ""
            zmx_string_free(error)
            throw StartFailure(message: message)
        }
        session = made
    }

    isolated deinit {
        zmx_session_free(session)
    }

    /// The snapshot, with the turns from `since` onward.
    func snapshot(since: Int) -> Snapshot? {
        guard let json = zmx_session_snapshot(session, since) else { return nil }
        defer { zmx_string_free(json) }
        let data = Data(bytes: json, count: strlen(json))
        return try? JSONDecoder().decode(Snapshot.self, from: data)
    }

    /// Sends what the player typed, then reports what the screen should show.
    func send(typed: String, showing: Snapshot) -> Snapshot? {
        _ = zmx_session_send_line(session, showing.generation, showing.turnCount, typed)
        return snapshot(since: showing.turnCount)
    }

    /// Sends a tap on one of the twelve places.
    func tap(place: String, showing: Snapshot) -> Snapshot? {
        _ = zmx_session_tap(session, showing.generation, showing.turnCount, place)
        return snapshot(since: showing.turnCount)
    }

    /// Sends a tap on one of the extra ways out.
    func tap(extra index: Int, showing: Snapshot) -> Snapshot? {
        _ = zmx_session_tap_extra(session, showing.generation, showing.turnCount, index)
        return snapshot(since: showing.turnCount)
    }

    /// Abandons the game and starts another.
    func newGame() -> Snapshot? {
        var generator = SystemRandomNumberGenerator()
        zmx_session_new_game(session, UInt32.random(in: .min ... .max, using: &generator))
        return snapshot(since: 0)
    }
}
