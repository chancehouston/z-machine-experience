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
    let compass: Compass
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

    enum CodingKeys: String, CodingKey {
        case generation, waiting, message, status, turns, compass, inventory, light
        case announcement, notice
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

struct Compass: Decodable, Sendable, Equatable {
    let offered: Bool
    let known: Bool
    let note: String?
    let places: [CompassPlace]
    let extras: [CompassExtra]
}

enum PlaceState: String, Decodable, Sendable {
    case open, maybe, door, closed, unknown, idle

    /// Whether a tap on a control in this state is sent.
    var sends: Bool { self == .open || self == .maybe || self == .door }
}

struct CompassPlace: Decodable, Sendable, Equatable {
    let place: String
    let label: String
    let state: PlaceState
    let command: String?
    let leadsTo: String?
    /// Whether the way leads into darkness, as far as the layer knows.
    let darkAhead: Bool
    let spoken: String

    enum CodingKeys: String, CodingKey {
        case place, label, state, command, spoken
        case leadsTo = "leads_to"
        case darkAhead = "dark_ahead"
    }
}

struct CompassExtra: Decodable, Sendable, Equatable {
    let label: String
    let state: PlaceState
    let command: String?
    let leadsTo: String?
    let darkAhead: Bool
    let spoken: String

    enum CodingKeys: String, CodingKey {
        case label, state, command, spoken
        case leadsTo = "leads_to"
        case darkAhead = "dark_ahead"
    }
}
