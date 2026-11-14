import SwiftUI

/// An item's token: a small mark for the kind of thing it is, drawn in the
/// map's hand. The kind is the layer's word for it, and this is the one file
/// that knows those words; what makes a thing a lamp or a container is the
/// layer's to say (specs/014-inventory-card/contracts/card.md, A§9, A§10).
struct ItemToken: View {
    let seed: Int
    let kind: String?
    let state: ItemState
    /// For a lamp: whether it is lit, where the layer knows.
    let lit: Bool?
    /// With `-showMapHand`, the identifier of an element tests read the
    /// token's kind from, with `+` or `-` after it for a lamp lit or not.
    var probe: String?

    /// The layer's word for a thing of no particular kind.
    static let plain = "plain"

    private let pencil = Color.primary.opacity(0.82)
    private var stroke: StrokeStyle {
        StrokeStyle(lineWidth: 1.4, lineCap: .round, lineJoin: .round)
    }

    var body: some View {
        ZStack {
            switch kind {
            case "lamp": lamp
            case "container": container
            case "page": page
            case "food": food
            case "clothing": clothing
            default: PencilBox(seed: seed &+ 7).stroke(pencil, style: stroke)
            }
        }
        .frame(width: 30, height: 26)
        .accessibilityHidden(true)
        .overlay {
            if let probe, MapHand.shown {
                Color.clear
                    .frame(width: 1, height: 1)
                    .accessibilityElement()
                    .accessibilityValue(
                        Text(verbatim: (kind ?? "") + (lit.map { $0 ? "+" : "-" } ?? "")))
                    .accessibilityIdentifier(probe)
            }
        }
    }

    private func line(_ from: CGPoint, _ to: CGPoint, bow: CGFloat = 1) -> some View {
        MapHand.line(from: from, to: to, seed: seed &+ Int(from.x * 7 + to.y), bow: bow)
            .stroke(pencil, style: stroke)
    }

    /// A lantern: a body under a handle; with rays, and filled, where it is
    /// lit.
    private var lamp: some View {
        ZStack {
            PencilBox(seed: seed &+ 3)
                .fill(lit == true ? Color.orange.opacity(0.75) : Color.clear)
                .frame(width: 14, height: 15)
                .offset(y: 4)
            PencilBox(seed: seed &+ 3)
                .stroke(pencil, style: stroke)
                .frame(width: 14, height: 15)
                .offset(y: 4)
            Path { path in
                path.addArc(
                    center: CGPoint(x: 15, y: 9), radius: 5,
                    startAngle: .degrees(180), endAngle: .degrees(0), clockwise: false)
            }
            .stroke(pencil, style: stroke)
            if lit == true {
                line(CGPoint(x: 3, y: 14), CGPoint(x: 6, y: 15), bow: 0)
                line(CGPoint(x: 27, y: 14), CGPoint(x: 24, y: 15), bow: 0)
                line(CGPoint(x: 4, y: 6), CGPoint(x: 7, y: 9), bow: 0)
                line(CGPoint(x: 26, y: 6), CGPoint(x: 23, y: 9), bow: 0)
            }
        }
    }

    /// A box, with its lid down where the layer says it is closed and lifted
    /// where it says it is open.
    private var container: some View {
        ZStack {
            PencilBox(seed: seed &+ 7)
                .stroke(pencil, style: stroke)
                .frame(width: 26, height: 18)
                .offset(y: 4)
            switch state {
            case .open: line(CGPoint(x: 3, y: 5), CGPoint(x: 23, y: -2))
            case .closed: line(CGPoint(x: 2, y: 8), CGPoint(x: 28, y: 8))
            case .plain: line(CGPoint(x: 11, y: 4), CGPoint(x: 19, y: 4), bow: 2.5)
            }
        }
    }

    /// A sheet with lines of writing.
    private var page: some View {
        ZStack {
            PencilBox(seed: seed &+ 11)
                .stroke(pencil, style: stroke)
                .frame(width: 20, height: 26)
            line(CGPoint(x: 9, y: 8), CGPoint(x: 21, y: 8), bow: 0.6)
            line(CGPoint(x: 9, y: 13), CGPoint(x: 21, y: 13), bow: 0.6)
            line(CGPoint(x: 9, y: 18), CGPoint(x: 17, y: 18), bow: 0.6)
        }
    }

    /// Something round, with a bite out of it.
    private var food: some View {
        Path { path in
            path.addArc(
                center: CGPoint(x: 14, y: 14), radius: 10,
                startAngle: .degrees(-25), endAngle: .degrees(-95), clockwise: false)
            path.addArc(
                center: CGPoint(x: 21, y: 6), radius: 5,
                startAngle: .degrees(165), endAngle: .degrees(75), clockwise: true)
        }
        .stroke(pencil, style: stroke)
    }

    /// A shirt.
    private var clothing: some View {
        Path { path in
            path.move(to: CGPoint(x: 10, y: 3))
            path.addLine(to: CGPoint(x: 3, y: 8))
            path.addLine(to: CGPoint(x: 6, y: 12))
            path.addLine(to: CGPoint(x: 9, y: 10))
            path.addLine(to: CGPoint(x: 9, y: 24))
            path.addLine(to: CGPoint(x: 21, y: 24))
            path.addLine(to: CGPoint(x: 21, y: 10))
            path.addLine(to: CGPoint(x: 24, y: 12))
            path.addLine(to: CGPoint(x: 27, y: 8))
            path.addLine(to: CGPoint(x: 20, y: 3))
            path.addQuadCurve(to: CGPoint(x: 10, y: 3), control: CGPoint(x: 15, y: 8))
        }
        .stroke(pencil, style: stroke)
    }
}
