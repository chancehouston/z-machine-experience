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

    enum CodingKeys: String, CodingKey {
        case generation, waiting, message, status, turns, compass
        case turnCount = "turn_count"
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

struct Compass: Decodable, Sendable, Equatable {
    let offered: Bool
    let known: Bool
    let note: String?
    let places: [CompassPlace]
    let extras: [CompassExtra]
}

enum PlaceState: String, Decodable, Sendable {
    case open, maybe, closed, unknown, idle

    /// Whether a tap on a control in this state is sent.
    var sends: Bool { self == .open || self == .maybe }
}

struct CompassPlace: Decodable, Sendable, Equatable {
    let place: String
    let label: String
    let state: PlaceState
    let command: String?
    let leadsTo: String?
    let spoken: String

    enum CodingKeys: String, CodingKey {
        case place, label, state, command, spoken
        case leadsTo = "leads_to"
    }
}

struct CompassExtra: Decodable, Sendable, Equatable {
    let label: String
    let state: PlaceState
    let command: String?
    let leadsTo: String?
    let spoken: String

    enum CodingKeys: String, CodingKey {
        case label, state, command, spoken
        case leadsTo = "leads_to"
    }
}
