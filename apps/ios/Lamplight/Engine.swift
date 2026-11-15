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

    /// Where the game is kept between launches, if the device gives a place.
    private let keptFile: KeptFile?

    /// Loads the story and runs it to its first stop. With no seed, the game's
    /// randomness comes from the system's generator. `pack` is the text of the
    /// story's game pack, if it has one: the layer decides whether it applies.
    /// `presentation` is the text of its presentation file, likewise.
    /// If `keptFile` holds a game, the layer brings it back, or says in its
    /// first snapshot that it could not.
    init(
        story: Data, pack: String?, presentation: String?, seed: UInt32?, keptFile: KeptFile?
    ) throws(StartFailure) {
        var generator = SystemRandomNumberGenerator()
        let entropy = UInt32.random(in: .min ... .max, using: &generator)
        var error: UnsafeMutablePointer<CChar>?
        let open = { (kept: UnsafePointer<UInt8>?, length: Int) in
            story.withUnsafeBytes { bytes in
                zmx_session_open(
                    bytes.bindMemory(to: UInt8.self).baseAddress, bytes.count,
                    seed ?? 0, seed == nil ? 0 : 1, entropy, pack, presentation, kept, length,
                    nil, &error)
            }
        }
        let made: OpaquePointer?
        if let kept = keptFile?.read() {
            // An empty file is still a kept game that cannot be brought back:
            // the layer is given a place to read no bytes from.
            made = kept.withUnsafeBytes { bytes in
                var none: UInt8 = 0
                return withUnsafePointer(to: &none) { nothing in
                    open(bytes.bindMemory(to: UInt8.self).baseAddress ?? nothing, bytes.count)
                }
            }
        } else {
            made = open(nil, 0)
        }
        guard let made else {
            let message = error.map { String(cString: $0) } ?? ""
            zmx_string_free(error)
            throw StartFailure(message: message)
        }
        session = made
        self.keptFile = keptFile
    }

    /// Stores the game as it stands, so that it can be brought back. If it
    /// cannot be stored the layer is told, and its next snapshot says so:
    /// the answer is then false.
    func keep() -> Bool {
        guard let keptFile else { return true }
        // No text: the session has stopped unexpectedly, and what was stored
        // before is left as it is.
        guard let text = zmx_session_kept(session) else { return true }
        defer { zmx_string_free(text) }
        do {
            try keptFile.write(Data(bytes: text, count: strlen(text)))
            return true
        } catch {
            zmx_session_keep_failed(session)
            return false
        }
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

    /// Sends a touch on a way of the map: one of the snapshot's ways of the
    /// place the player is in, by its place in that list.
    func tap(way index: Int, showing: Snapshot) -> Snapshot? {
        _ = zmx_session_tap_way(session, showing.generation, showing.turnCount, index)
        return snapshot(since: showing.turnCount)
    }

    /// Sends a touch on the place the player is in: the snapshot's look.
    func tapPlace(showing: Snapshot) -> Snapshot? {
        _ = zmx_session_tap_place(session, showing.generation, showing.turnCount)
        return snapshot(since: showing.turnCount)
    }

    /// Has the player's sheet of the map laid out again around them, then
    /// reports what the screen should show. The game is sent nothing.
    func redrawMap(showing: Snapshot) -> Snapshot? {
        _ = zmx_session_redraw_map(session, showing.generation, showing.turnCount)
        return snapshot(since: showing.turnCount)
    }

    /// A sheet of the map by its number. Nothing is sent to the game.
    func sheet(number: Int, showing: Snapshot) -> MapSheet? {
        guard let json = zmx_session_sheet(session, showing.generation, showing.turnCount, number)
        else { return nil }
        defer { zmx_string_free(json) }
        let data = Data(bytes: json, count: strlen(json))
        return try? JSONDecoder().decode(MapSheet.self, from: data)
    }

    /// The card for a carried item, by its place among the shown items.
    /// Nothing is sent to the game.
    func card(index: Int, showing: Snapshot) -> Card? {
        guard let json = zmx_session_card(session, showing.generation, showing.turnCount, index)
        else { return nil }
        defer { zmx_string_free(json) }
        let data = Data(bytes: json, count: strlen(json))
        return try? JSONDecoder().decode(Card.self, from: data)
    }

    /// Sends an action chosen on an item's card.
    func tap(action: Int, item index: Int, showing: Snapshot) -> Snapshot? {
        _ = zmx_session_tap_action(
            session, showing.generation, showing.turnCount, index, action)
        return snapshot(since: showing.turnCount)
    }

    /// Abandons the game and starts another.
    func newGame() -> Snapshot? {
        var generator = SystemRandomNumberGenerator()
        zmx_session_new_game(session, UInt32.random(in: .min ... .max, using: &generator))
        return snapshot(since: 0)
    }
}
