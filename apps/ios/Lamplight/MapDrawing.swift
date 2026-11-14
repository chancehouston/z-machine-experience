import SwiftUI

/// The hand the map is drawn in: outlines and lines that are a little
/// uneven, as pencil on paper is, and always uneven in the same way. Every
/// unevenness is worked out from a number the snapshot gave (a place's id),
/// so a place looks the same each time it is drawn (contract A§10).
enum MapHand {
    /// `-showMapHand` adds elements that tests read: each place's outline,
    /// and whether the map moves.
    static let shown = ProcessInfo.processInfo.arguments.contains("-showMapHand")
    /// `-reduceMotion` draws the map as the system's Reduce Motion does.
    static let stillAsked = ProcessInfo.processInfo.arguments.contains("-reduceMotion")

    /// Numbers between -1 and 1 that follow from a seed alone.
    struct Run {
        private var state: UInt64

        init(seed: Int) {
            state = UInt64(bitPattern: Int64(seed)) &* 0x9E37_79B9_7F4A_7C15 &+ 0xD1B5_4A32_D192_ED03
        }

        mutating func next() -> CGFloat {
            state &+= 0x9E37_79B9_7F4A_7C15
            var z = state
            z = (z ^ (z >> 30)) &* 0xBF58_476D_1CE4_E5B9
            z = (z ^ (z >> 27)) &* 0x94D0_49BB_1331_11EB
            z ^= z >> 31
            return CGFloat(z >> 11) / CGFloat(UInt64(1) << 53) * 2 - 1
        }
    }

    /// What a place's outline is drawn from: how far each of its four
    /// corners is moved, across and down, and how far each side bows.
    static func wobble(seed: Int) -> [CGFloat] {
        var run = Run(seed: seed)
        return (0..<12).map { _ in run.next() }
    }

    /// The outline's numbers as text, for tests.
    static func outline(seed: Int) -> String {
        wobble(seed: seed).map { String(Int(($0 * 100).rounded())) }.joined(separator: " ")
    }

    /// A line from one point to another that bows a little to one side.
    static func line(from: CGPoint, to: CGPoint, seed: Int, bow: CGFloat = 2.2) -> Path {
        var run = Run(seed: seed)
        let length = max(hypot(to.x - from.x, to.y - from.y), 1)
        let across = CGPoint(x: -(to.y - from.y) / length, y: (to.x - from.x) / length)
        let along = 0.5 + run.next() * 0.15
        let aside = run.next() * bow
        var path = Path()
        path.move(to: from)
        path.addQuadCurve(
            to: to,
            control: CGPoint(
                x: from.x + (to.x - from.x) * along + across.x * aside,
                y: from.y + (to.y - from.y) * along + across.y * aside))
        return path
    }
}

/// A place's outline: a box whose corners are not quite square and whose
/// sides are not quite straight.
struct PencilBox: Shape {
    let seed: Int

    func path(in rect: CGRect) -> Path {
        let wobble = MapHand.wobble(seed: seed)
        let reach: CGFloat = 1.6
        let inner = rect.insetBy(dx: 2.2, dy: 2.2)
        let square = [
            CGPoint(x: inner.minX, y: inner.minY), CGPoint(x: inner.maxX, y: inner.minY),
            CGPoint(x: inner.maxX, y: inner.maxY), CGPoint(x: inner.minX, y: inner.maxY),
        ]
        let corners = square.indices.map { index in
            CGPoint(
                x: square[index].x + wobble[index * 2] * reach,
                y: square[index].y + wobble[index * 2 + 1] * reach)
        }
        var path = Path()
        path.move(to: corners[0])
        for index in corners.indices {
            let from = corners[index]
            let to = corners[(index + 1) % corners.count]
            let length = max(hypot(to.x - from.x, to.y - from.y), 1)
            let bow = wobble[8 + index] * 1.4
            path.addQuadCurve(
                to: to,
                control: CGPoint(
                    x: (from.x + to.x) / 2 - (to.y - from.y) / length * bow,
                    y: (from.y + to.y) / 2 + (to.x - from.x) / length * bow))
        }
        path.closeSubpath()
        return path
    }
}

/// A line that is drawn in from its start when it first appears, over a
/// quarter of a second; at once where the map is to be still (FR-029).
private struct DrawnIn: View {
    let path: Path
    let color: Color
    let style: StrokeStyle
    let still: Bool

    @State private var drawn = false

    var body: some View {
        path
            .trim(from: 0, to: drawn || still ? 1 : 0)
            .stroke(color, style: style)
            .onAppear {
                if still {
                    drawn = true
                } else {
                    withAnimation(.easeOut(duration: 0.25)) { drawn = true }
                }
            }
    }
}

/// Draws a sheet of the map as the snapshot gives it: places at their
/// positions, the lines between them, the ways seen and not walked, and the
/// ways that lead off the sheet. It decides nothing: which places there are,
/// where each is, and which line is straight are the layer's.
struct MapDrawing: View {
    let map: MapSheet
    /// Whether nothing is to be drawn in: Reduce Motion.
    let still: Bool
    /// False while a command is being worked out.
    let ready: Bool
    /// A touch on the place the player is in.
    let tapPlace: () -> Void

    /// The pencil's colour.
    private let pencil = Color.primary.opacity(0.82)

    private func place(_ id: Int) -> MapPlace? {
        map.places.first { $0.id == id }
    }

    var body: some View {
        ZStack(alignment: .topLeading) {
            ForEach(map.lines, id: \.key) { line in
                self.line(line)
            }
            ForEach(map.stubs.indices, id: \.self) { index in
                stub(map.stubs[index])
            }
            ForEach(map.leads.indices, id: \.self) { index in
                lead(map.leads[index])
            }
            ForEach(map.places) { place in
                box(place)
            }
        }
    }

    /// A place: a box with its name. One seen only in the dark has no name,
    /// and is filled in. The place the player is in carries the lamp.
    private func box(_ place: MapPlace) -> some View {
        let size = MapLayout.size(place)
        let here = place.id == map.here
        return PencilBox(seed: place.id)
            .fill(place.name == nil ? Color.secondary.opacity(0.35) : Color(.systemBackground))
            .overlay {
                PencilBox(seed: place.id)
                    .stroke(
                        here ? Color.accentColor : pencil,
                        style: StrokeStyle(lineWidth: here ? 2.4 : 1.3, lineCap: .round, lineJoin: .round))
            }
            .overlay {
                Text(verbatim: place.name ?? "")
                    .font(.system(size: place.inset ? 10 : 11, weight: here ? .semibold : .regular))
                    .multilineTextAlignment(.center)
                    .lineLimit(2)
                    .minimumScaleFactor(0.7)
                    .padding(.horizontal, 5)
            }
            .frame(width: size.width, height: size.height)
            // The place the player is in is touched in a taller area than
            // its box, and one no wider than leaves the ways' controls clear.
            .frame(
                width: here ? min(size.width, MapLayout.placeTouch.width) : size.width,
                height: here ? max(size.height, MapLayout.placeTouch.height) : size.height
            )
            .contentShape(Rectangle())
            .accessibilityElement(children: .ignore)
            .accessibilityLabel(Text(verbatim: place.name ?? ""))
            .accessibilityIdentifier(here ? "map.here" : "map.place.\(place.id)")
            .modifier(Looking(look: here && ready ? map.look : nil, action: tapPlace))
            // The place the player is in is counted with the others by tests.
            .overlay {
                if here {
                    Color.clear
                        .frame(width: 1, height: 1)
                        .accessibilityElement()
                        .accessibilityLabel(Text(verbatim: place.name ?? ""))
                        .accessibilityValue(Text(verbatim: map.lamp))
                        .accessibilityIdentifier("map.place.\(place.id)")
                }
            }
            // The marker: a lamp at the corner of the place the player is
            // in, lit or out as the layer says (S§2).
            .overlay(alignment: .topLeading) {
                if here {
                    Image(systemName: MapLayout.lampSymbol(for: map.lamp))
                        .font(.system(size: 11, weight: .semibold))
                        .foregroundStyle(MapLayout.lampIsLit(map.lamp) ? Color.orange : Color.secondary)
                        .frame(width: 20, height: 20)
                        .background(Circle().fill(Color(.systemBackground)))
                        .overlay(Circle().strokeBorder(Color.accentColor, lineWidth: 1.2))
                        .accessibilityElement()
                        .accessibilityLabel(Text(verbatim: map.lamp))
                        .accessibilityValue(Text(verbatim: map.lamp))
                        .accessibilityIdentifier("map.lamp")
                        .offset(x: -9, y: -9)
                }
            }
            .overlay(alignment: .bottomTrailing) {
                if MapHand.shown {
                    Color.clear
                        .frame(width: 1, height: 1)
                        .accessibilityElement()
                        .accessibilityValue(Text(verbatim: MapHand.outline(seed: place.id)))
                        .accessibilityIdentifier("map.outline.\(place.id)")
                }
            }
            .modifier(AppearingIn(still: still))
            .position(MapLayout.point(place))
    }

    /// A line between two places of the sheet.
    @ViewBuilder
    private func line(_ line: MapLine) -> some View {
        if let a = place(line.a), let b = place(line.b) {
            let from = MapLayout.point(a)
            let to = MapLayout.point(b)
            let seed = line.a &* 8191 &+ line.b
            switch line.kind {
            case "straight":
                DrawnIn(
                    path: MapHand.line(from: from, to: to, seed: seed),
                    color: pencil,
                    style: StrokeStyle(lineWidth: 1.6, lineCap: .round),
                    still: still)
            case "ladder":
                // Rungs: a mark that cannot be taken for a compass line.
                DrawnIn(
                    path: MapHand.line(from: from, to: to, seed: seed, bow: 0.8),
                    color: pencil,
                    style: StrokeStyle(lineWidth: 7, dash: [1.5, 4]),
                    still: still)
            case "bent":
                // It leaves each end the way it was walked.
                DrawnIn(
                    path: Path { path in
                        path.move(to: from)
                        path.addCurve(
                            to: to,
                            control1: lean(from, by: line.out, toward: to),
                            control2: lean(to, by: line.back, toward: from))
                    },
                    color: pencil,
                    style: StrokeStyle(lineWidth: 1.4, lineCap: .round),
                    still: still)
            default:
                DrawnIn(
                    path: MapHand.line(from: from, to: to, seed: seed),
                    color: .secondary,
                    style: StrokeStyle(lineWidth: 1.4, lineCap: .round, dash: [2, 5]),
                    still: still)
            }
        }
    }

    /// A point out from `from` in the direction of `way`, or toward `other`
    /// where the way is not one of the compass's.
    private func lean(_ from: CGPoint, by way: String?, toward other: CGPoint) -> CGPoint {
        if let way, let step = MapLayout.step(for: way) {
            return CGPoint(
                x: from.x + step.x * MapLayout.step.width * 0.6,
                y: from.y + step.y * MapLayout.step.height * 0.8)
        }
        return CGPoint(x: (from.x + other.x) / 2, y: (from.y + other.y) / 2)
    }

    /// The edge of a place's box in the direction of a way, and a point a
    /// little beyond it.
    private func edge(of place: MapPlace, toward way: String, reach: CGFloat) -> (CGPoint, CGPoint) {
        let centre = MapLayout.point(place)
        let size = MapLayout.size(place)
        let step = MapLayout.step(for: way) ?? MapLayout.cornerStep(for: way)
        let start = CGPoint(x: centre.x + step.x * size.width / 2, y: centre.y + step.y * size.height / 2)
        return (start, CGPoint(x: start.x + step.x * reach, y: start.y + step.y * reach))
    }

    /// A way seen and not walked, of a place the player is not in: a short
    /// line, dashed where the game decides, crossed where it is a closed door.
    private func stub(_ stub: MapStub) -> some View {
        Group {
            if let place = place(stub.of) {
                let (start, end) = edge(of: place, toward: stub.way, reach: 10)
                Path { path in
                    path.move(to: start)
                    path.addLine(to: end)
                    if stub.state == "door" {
                        let across = CGPoint(x: -(end.y - start.y) * 0.4, y: (end.x - start.x) * 0.4)
                        path.move(to: CGPoint(x: end.x + across.x, y: end.y + across.y))
                        path.addLine(to: CGPoint(x: end.x - across.x, y: end.y - across.y))
                    }
                }
                .stroke(
                    Color.secondary,
                    style: StrokeStyle(
                        lineWidth: 1.4, lineCap: .round, dash: stub.state == "maybe" ? [2, 3] : []))
            }
        }
    }

    /// A walked way that leaves the sheet: a short line, and the layer's
    /// words for the way and for where it leads.
    private func lead(_ lead: MapLead) -> some View {
        Group {
            if let place = place(lead.from) {
                let (start, end) = edge(of: place, toward: lead.way, reach: 12)
                Path { path in
                    path.move(to: start)
                    path.addLine(to: end)
                }
                .stroke(Color.accentColor, lineWidth: 1.4)
                Text(verbatim: [lead.label, lead.to].compactMap { $0 }.joined(separator: " → "))
                    .font(.system(size: 9))
                    .foregroundStyle(.tint)
                    .fixedSize()
                    .position(x: end.x, y: end.y + (end.y >= start.y ? 8 : -8))
            }
        }
    }
}

/// The place the player is in, where the map says it can be touched: a touch
/// is handed on, and a screen reader is told it is a button and, in the
/// layer's words, what it does (specs/013-room-look/contracts/look.md, A§1).
/// It answers a tap and is not a button of its own, so that a drag beginning
/// on it still moves the sheet.
private struct Looking: ViewModifier {
    let look: MapLook?
    let action: () -> Void

    func body(content: Content) -> some View {
        if let look {
            content
                .onTapGesture(perform: action)
                .accessibilityAddTraits(.isButton)
                .accessibilityHint(Text(verbatim: look.spoken))
        } else {
            content
        }
    }
}

/// A place that comes into view when it is first drawn, over a quarter of a
/// second; at once where the map is to be still.
private struct AppearingIn: ViewModifier {
    let still: Bool

    @State private var shown = false

    func body(content: Content) -> some View {
        content
            .opacity(shown || still ? 1 : 0)
            .scaleEffect(shown || still ? 1 : 0.92)
            .onAppear {
                if still {
                    shown = true
                } else {
                    withAnimation(.easeOut(duration: 0.25)) { shown = true }
                }
            }
    }
}

/// The control for one way of the place the player is in, drawn from its
/// state as the compass drew it: open, a filled disc with the arrow, and a
/// small black badge where the way leads into darkness; maybe, an outlined
/// disc with the arrow and a question mark; door (a closed door), an outlined
/// upright panel with the arrow; refused, a faint barred ring.
struct WayControl: View {
    let way: MapWay
    let identifier: String
    let ready: Bool
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            VStack(spacing: 0) {
                mark.frame(width: 28, height: 28)
                if MapLayout.isCaptioned(way.way) {
                    Text(verbatim: way.label)
                        .font(.system(size: 10, weight: .medium))
                        .lineLimit(1)
                        .fixedSize()
                }
            }
            .frame(width: MapLayout.control, height: MapLayout.control)
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .disabled(way.command == nil || !ready)
        .accessibilityLabel(Text(verbatim: way.spoken))
        .accessibilityValue(Text(verbatim: way.state))
        .accessibilityIdentifier(identifier)
    }

    @ViewBuilder
    private var mark: some View {
        let symbol = MapLayout.symbol(for: way.way)
        switch way.state {
        case "open":
            Circle().fill(.tint)
                .overlay {
                    Image(systemName: symbol.name)
                        .font(.system(size: 14, weight: .bold))
                        .rotationEffect(.degrees(symbol.turned))
                        .foregroundStyle(.white)
                }
                .overlay(alignment: .topTrailing) {
                    if way.darkAhead {
                        Image(systemName: "moon.fill")
                            .font(.system(size: 7, weight: .heavy))
                            .foregroundStyle(.white)
                            .frame(width: 14, height: 14)
                            .background(Circle().fill(.black))
                            .overlay(Circle().strokeBorder(.white, lineWidth: 1))
                            .offset(x: 4, y: -4)
                    }
                }
        case "maybe":
            Circle().fill(Color(.systemBackground))
                .overlay { Circle().strokeBorder(.tint, lineWidth: 2) }
                .overlay {
                    Image(systemName: symbol.name)
                        .font(.system(size: 12, weight: .semibold))
                        .rotationEffect(.degrees(symbol.turned))
                        .foregroundStyle(.tint)
                }
                .overlay(alignment: .topTrailing) {
                    Text(verbatim: "?")
                        .font(.system(size: 11, weight: .heavy))
                        .foregroundStyle(.white)
                        .frame(width: 14, height: 14)
                        .background(Circle().fill(.tint))
                        .offset(x: 4, y: -4)
                }
        case "door":
            RoundedRectangle(cornerRadius: 4)
                .fill(Color(.systemBackground))
                .overlay { RoundedRectangle(cornerRadius: 4).strokeBorder(.tint, lineWidth: 2) }
                .frame(width: 22, height: 27)
                .overlay {
                    Image(systemName: symbol.name)
                        .font(.system(size: 11, weight: .semibold))
                        .rotationEffect(.degrees(symbol.turned))
                        .foregroundStyle(.tint)
                }
        default:
            // Tried, and refused.
            Circle()
                .strokeBorder(.secondary.opacity(0.7), lineWidth: 1.5)
                .frame(width: 18, height: 18)
                .overlay {
                    Rectangle().fill(.secondary.opacity(0.7)).frame(width: 18, height: 1.5)
                        .rotationEffect(.degrees(45))
                }
        }
    }
}
