import Foundation

/// What the experience layer says the screen should show. The fields are the
/// layer's (specs/004-first-demo/data-model.md); the app adds nothing to them.
struct Snapshot: Decodable, Sendable, Equatable {
    let generation: UInt64
    let turnCount: Int
    let waiting: Waiting
    let message: String?
    let status: Status?
    let turns: [Turn]
    let inventory: Strip
    let light: Light
    /// What a screen reader is to speak for the newest turn: the layer's
    /// text, whole. Nil where there is nothing to say.
    let announcement: String?
    /// The layer's words to show once: that an earlier game could not be
    /// brought back, or that this one is not being kept.
    let notice: String?
    /// The layer's question before this game is given up for a new one; nil
    /// where nothing would be given up.
    let startOver: String?
    /// The sheet of the map the player is on; nil where the layer has no map
    /// for the game (specs/012-map-first-slice/contracts/map.md, part S).
    let map: MapSheet?

    enum CodingKeys: String, CodingKey {
        case generation, waiting, message, status, turns, inventory, light
        case announcement, notice, map
        case turnCount = "turn_count"
        case startOver = "start_over"
    }
}

enum Waiting: String, Decodable, Sendable {
    case command, answer, ended, fault
}

struct Status: Decodable, Sendable, Equatable {
    let place: String
    let score: Int?
    let moves: Int?
    let hours: Int?
    let minutes: Int?
}

struct Turn: Decodable, Sendable, Equatable, Identifiable {
    let number: Int
    let command: String?
    let source: String?
    let text: String

    var id: Int { number }
}

/// Whether the player is in the dark, and the layer's words for it. `dark` is
/// nil where the layer does not know.
struct Light: Decodable, Sendable, Equatable {
    let dark: Bool?
    let note: String?
}

/// What to show of what the player carries: the layer's words throughout.
struct Strip: Decodable, Sendable, Equatable {
    let known: Bool
    let note: String?
    let spoken: String
    let items: [StripItem]
}

struct StripItem: Decodable, Sendable, Equatable {
    let name: String
    let state: ItemState
    let contents: [StripItem]
}

enum ItemState: String, Decodable, Sendable {
    case plain, open, closed
}

/// One sheet of the map, as the layer gives it. Every position, every state
/// and every word is the layer's; the app draws them and works nothing out.
struct MapSheet: Decodable, Sendable, Equatable {
    /// The layer's words where there is something to say in place of the map.
    let note: String?
    /// The layer's summary, for a screen reader.
    let spoken: String
    let sheet: Int
    let sheets: Int
    /// The place the player is in; nil where the layer cannot say.
    let here: Int?
    let lamp: String
    let places: [MapPlace]
    let lines: [MapLine]
    let leads: [MapLead]
    let stubs: [MapStub]
    let ways: [MapWay]
}

struct MapPlace: Decodable, Sendable, Equatable, Identifiable {
    let id: Int
    let x: Int
    let y: Int
    /// Drawn small, half a step right of and below its position.
    let inset: Bool
    /// Nil until the player has seen the place lit.
    let name: String?
}

struct MapLine: Decodable, Sendable, Equatable {
    let a: Int
    let b: Int
    let out: String
    let back: String?
    let kind: String
    let label: String?
}

struct MapLead: Decodable, Sendable, Equatable {
    let from: Int
    let way: String
    let label: String
    let to: String?
    let sheet: Int
}

struct MapStub: Decodable, Sendable, Equatable {
    let of: Int
    let way: String
    let state: String
    let darkAhead: Bool

    enum CodingKeys: String, CodingKey {
        case of, way, state
        case darkAhead = "dark_ahead"
    }
}

/// A way of the place the player is in.
struct MapWay: Decodable, Sendable, Equatable {
    let way: String
    let label: String
    let state: String
    /// Present when the way can be sent now.
    let command: String?
    let darkAhead: Bool
    let refused: Bool
    let spoken: String

    enum CodingKeys: String, CodingKey {
        case way, label, state, command, refused, spoken
        case darkAhead = "dark_ahead"
    }
}
