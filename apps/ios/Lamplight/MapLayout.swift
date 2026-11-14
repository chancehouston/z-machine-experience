import CoreGraphics

/// Where things go in the map's drawing: how far apart positions are, where a
/// way's control sits beside the place the player is in, and the symbol that
/// stands for each way. This is all the app knows of directions: which ways
/// there are, their states and what they send are the snapshot's.
enum MapLayout {
    /// The distance between two positions of a sheet, across and down.
    static let step = CGSize(width: 128, height: 84)
    /// The box a place is drawn in. An inset place is drawn smaller.
    static let box = CGSize(width: 82, height: 34)
    static let insetScale: CGFloat = 0.8
    /// The side of the square a way is touched in. It does not grow with the
    /// text size.
    static let control: CGFloat = 44
    /// How tall the band is while the keyboard is up.
    static let band: CGFloat = 200

    /// Where a place is drawn, from the sheet's own origin.
    static func point(_ place: MapPlace) -> CGPoint {
        let half: CGFloat = place.inset ? 0.5 : 0
        return CGPoint(
            x: (CGFloat(place.x) + half) * step.width,
            y: (CGFloat(place.y) + half) * step.height)
    }

    static func size(_ place: MapPlace) -> CGSize {
        let scale = place.inset ? insetScale : 1
        return CGSize(width: box.width * scale, height: box.height * scale)
    }

    /// The step on the page for the eight ways of the compass: x to the
    /// right, y downward. Nil for every other way.
    static func step(for way: String) -> (x: CGFloat, y: CGFloat)? {
        switch way {
        case "north": (0, -1)
        case "northeast": (1, -1)
        case "east": (1, 0)
        case "southeast": (1, 1)
        case "south": (0, 1)
        case "southwest": (-1, 1)
        case "west": (-1, 0)
        case "northwest": (-1, -1)
        default: nil
        }
    }

    /// Where a way's control sits, from the middle of the place the player is
    /// in: the eight in a ring round it, and up, down, in and out at the four
    /// corners beyond, in the gaps between the rows of places. Nil for a way
    /// that is none of the twelve.
    static func controlOffset(for way: String) -> CGSize? {
        let ring = CGSize(width: 62, height: 48)
        if let step = step(for: way) {
            return CGSize(width: step.x * ring.width, height: step.y * ring.height)
        }
        let corner = CGSize(width: 120, height: 48)
        switch way {
        case "up": return CGSize(width: corner.width, height: -corner.height)
        case "down": return CGSize(width: -corner.width, height: corner.height)
        case "in": return CGSize(width: -corner.width, height: -corner.height)
        case "out": return CGSize(width: corner.width, height: corner.height)
        default: return nil
        }
    }

    /// Where a way that is not of the compass is drawn from at a place's
    /// edge: up and out to the right, down and in to the left, as their
    /// controls are; anything else, beneath.
    static func cornerStep(for way: String) -> (x: CGFloat, y: CGFloat) {
        switch way {
        case "up": (1, -1)
        case "down": (-1, 1)
        case "in": (-1, -1)
        case "out": (1, 1)
        default: (0, 1)
        }
    }

    /// The symbol for a way and how far it is turned, in degrees.
    static func symbol(for way: String) -> (name: String, turned: Double) {
        switch way {
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

    /// Whether the way's name is written under its symbol: the four that are
    /// not points of the compass.
    static func isCaptioned(_ way: String) -> Bool {
        ["up", "down", "in", "out"].contains(way)
    }
}
