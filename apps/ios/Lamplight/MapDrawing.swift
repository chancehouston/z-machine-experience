import SwiftUI

/// Draws a sheet of the map as the snapshot gives it: places at their
/// positions, the lines between them, the ways seen and not walked, and the
/// ways that lead off the sheet. It decides nothing: which places there are,
/// where each is, and which line is straight are the layer's.
struct MapDrawing: View {
    let map: MapSheet

    private func place(_ id: Int) -> MapPlace? {
        map.places.first { $0.id == id }
    }

    var body: some View {
        ZStack(alignment: .topLeading) {
            ForEach(map.lines.indices, id: \.self) { index in
                line(map.lines[index])
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
    /// and is filled in.
    private func box(_ place: MapPlace) -> some View {
        let size = MapLayout.size(place)
        let here = place.id == map.here
        return RoundedRectangle(cornerRadius: 4)
            .fill(place.name == nil ? Color.secondary.opacity(0.35) : Color(.systemBackground))
            .overlay {
                RoundedRectangle(cornerRadius: 4)
                    .strokeBorder(here ? Color.accentColor : Color.primary, lineWidth: here ? 2.5 : 1.2)
            }
            .overlay {
                Text(verbatim: place.name ?? "")
                    .font(.system(size: place.inset ? 10 : 11, weight: here ? .semibold : .regular))
                    .multilineTextAlignment(.center)
                    .lineLimit(2)
                    .minimumScaleFactor(0.7)
                    .padding(.horizontal, 3)
            }
            .frame(width: size.width, height: size.height)
            .accessibilityElement(children: .ignore)
            .accessibilityLabel(Text(verbatim: place.name ?? ""))
            .accessibilityIdentifier(here ? "map.here" : "map.place.\(place.id)")
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
            .position(MapLayout.point(place))
    }

    /// A line between two places of the sheet.
    @ViewBuilder
    private func line(_ line: MapLine) -> some View {
        if let a = place(line.a), let b = place(line.b) {
            let from = MapLayout.point(a)
            let to = MapLayout.point(b)
            switch line.kind {
            case "straight":
                Path { path in
                    path.move(to: from)
                    path.addLine(to: to)
                }
                .stroke(Color.primary, lineWidth: 1.6)
            case "ladder":
                // Rungs: a mark that cannot be taken for a compass line.
                Path { path in
                    path.move(to: from)
                    path.addLine(to: to)
                }
                .stroke(Color.primary, style: StrokeStyle(lineWidth: 7, dash: [1.5, 4]))
            case "bent":
                // It leaves each end the way it was walked.
                Path { path in
                    path.move(to: from)
                    path.addCurve(
                        to: to,
                        control1: lean(from, by: line.out, toward: to),
                        control2: lean(to, by: line.back, toward: from))
                }
                .stroke(Color.primary, style: StrokeStyle(lineWidth: 1.4, lineCap: .round))
            default:
                Path { path in
                    path.move(to: from)
                    path.addLine(to: to)
                }
                .stroke(Color.secondary, style: StrokeStyle(lineWidth: 1.4, lineCap: .round, dash: [2, 5]))
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
