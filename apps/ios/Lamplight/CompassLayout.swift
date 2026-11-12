/// Where each of the twelve places goes on the screen, and the symbol that
/// stands for it. This is all the app knows of directions: what a place is
/// called, whether it is open and what it sends are the snapshot's.
enum CompassLayout {
    /// The eight points, in rows from the top; the centre is empty.
    static let ring: [[String?]] = [
        ["northwest", "north", "northeast"],
        ["west", nil, "east"],
        ["southwest", "south", "southeast"],
    ]
    /// The column to the left of the ring, and the column to the right.
    static let left = ["up", "down"]
    static let right = ["in", "out"]

    /// The symbol for a place and how far it is turned, in degrees.
    static func symbol(for place: String) -> (name: String, turned: Double) {
        switch place {
        case "north": ("arrow.up", 0)
        case "northeast": ("arrow.up", 45)
        case "east": ("arrow.up", 90)
        case "southeast": ("arrow.up", 135)
        case "south": ("arrow.up", 180)
        case "southwest": ("arrow.up", 225)
        case "west": ("arrow.up", 270)
        case "northwest": ("arrow.up", 315)
        case "up": ("arrow.up.to.line", 0)
        case "down": ("arrow.down.to.line", 0)
        case "in": ("arrow.right.to.line", 0)
        case "out": ("arrow.left.to.line", 0)
        default: ("circle", 0)
        }
    }

    /// Whether the place's name is written under its symbol: the four that are
    /// not points of the compass.
    static func isCaptioned(_ place: String) -> Bool {
        left.contains(place) || right.contains(place)
    }
}
