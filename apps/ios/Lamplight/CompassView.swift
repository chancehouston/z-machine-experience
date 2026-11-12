import SwiftUI

/// Draws the compass the snapshot reports. A control sends only in the states
/// the layer marks as sending; nothing is worked out here.
struct CompassView: View {
    let compass: Compass
    /// False while a command is being worked out.
    let ready: Bool
    let tapPlace: (String) -> Void
    let tapExtra: (Int) -> Void

    var body: some View {
        VStack(spacing: 8) {
            HStack(spacing: 20) {
                column(CompassLayout.left)
                Grid(horizontalSpacing: 2, verticalSpacing: 2) {
                    ForEach(CompassLayout.ring.indices, id: \.self) { row in
                        GridRow {
                            ForEach(CompassLayout.ring[row].indices, id: \.self) { index in
                                if let name = CompassLayout.ring[row][index] {
                                    control(name)
                                } else {
                                    Color.clear.frame(width: CompassControl.side, height: CompassControl.side)
                                }
                            }
                        }
                    }
                }
                column(CompassLayout.right)
            }
            .opacity(compass.offered ? 1 : 0.45)
            if !compass.extras.isEmpty {
                FlowLayout(spacing: 8) {
                    ForEach(compass.extras.indices, id: \.self) { index in
                        let extra = compass.extras[index]
                        Button {
                            tapExtra(index)
                        } label: {
                            Text(verbatim: extra.label)
                        }
                        .buttonStyle(.bordered)
                        .disabled(!extra.state.sends || !ready)
                        .accessibilityLabel(Text(verbatim: extra.spoken))
                        .accessibilityValue(Text(verbatim: extra.state.rawValue))
                        .accessibilityIdentifier("compass.extra.\(index)")
                    }
                }
            }
            if let note = compass.note {
                Text(verbatim: note)
                    .font(.footnote)
                    .foregroundStyle(.secondary)
                    .multilineTextAlignment(.center)
                    .accessibilityIdentifier("compass.note")
            }
        }
        .padding(.vertical, 8)
        .frame(maxWidth: .infinity)
    }

    private func column(_ names: [String]) -> some View {
        VStack(spacing: 2) {
            ForEach(names, id: \.self) { control($0) }
        }
    }

    @ViewBuilder
    private func control(_ name: String) -> some View {
        if let place = compass.places.first(where: { $0.place == name }) {
            CompassControl(place: place, ready: ready) { tapPlace(name) }
        } else {
            Color.clear.frame(width: CompassControl.side, height: CompassControl.side)
        }
    }
}

/// One of the twelve controls, drawn from its state:
/// open, a filled disc with the arrow; maybe, an outlined disc with the arrow
/// and a question mark; door (a closed door), a solid upright panel with the
/// arrow and a knob; closed and idle, a faint dot; unknown, a dashed ring.
struct CompassControl: View {
    /// The side of the square a control is tapped in. It does not grow with the
    /// text size.
    static let side: CGFloat = 48

    let place: CompassPlace
    let ready: Bool
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            VStack(spacing: 0) {
                mark.frame(width: 32, height: 32)
                if CompassLayout.isCaptioned(place.place) {
                    Text(verbatim: place.label)
                        .font(.system(size: 11, weight: .medium))
                        .foregroundStyle(place.state.sends ? Color.primary : Color.secondary)
                        .lineLimit(1)
                        .fixedSize()
                }
            }
            .frame(width: Self.side, height: Self.side)
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .disabled(!place.state.sends || !ready)
        .accessibilityLabel(Text(verbatim: place.spoken))
        .accessibilityValue(Text(verbatim: place.state.rawValue))
        .accessibilityIdentifier("compass.\(place.place)")
    }

    @ViewBuilder
    private var mark: some View {
        let symbol = CompassLayout.symbol(for: place.place)
        switch place.state {
        case .open:
            Circle().fill(.tint)
                .overlay {
                    Image(systemName: symbol.name)
                        .font(.system(size: 16, weight: .bold))
                        .rotationEffect(.degrees(symbol.turned))
                        .foregroundStyle(.white)
                }
        case .maybe:
            Circle().strokeBorder(.tint, lineWidth: 2)
                .overlay {
                    Image(systemName: symbol.name)
                        .font(.system(size: 14, weight: .semibold))
                        .rotationEffect(.degrees(symbol.turned))
                        .foregroundStyle(.tint)
                }
                .overlay(alignment: .topTrailing) {
                    Text(verbatim: "?")
                        .font(.system(size: 12, weight: .heavy))
                        .foregroundStyle(.white)
                        .frame(width: 15, height: 15)
                        .background(Circle().fill(.tint))
                        .offset(x: 4, y: -4)
                }
        case .door:
            // A closed door: a solid upright panel with a knob, not a disc, so
            // that its shape tells it from the other states without colour.
            RoundedRectangle(cornerRadius: 4)
                .fill(.tint)
                .frame(width: 24, height: 32)
                .overlay {
                    Image(systemName: symbol.name)
                        .font(.system(size: 12, weight: .bold))
                        .rotationEffect(.degrees(symbol.turned))
                        .foregroundStyle(.white)
                        .offset(y: -3)
                }
                .overlay(alignment: .bottomTrailing) {
                    Circle().fill(.white).frame(width: 3.5, height: 3.5).offset(x: -4, y: -5)
                }
        case .closed, .idle:
            Circle().fill(.secondary.opacity(0.4)).frame(width: 7, height: 7)
        case .unknown:
            Circle()
                .strokeBorder(.secondary.opacity(0.6), style: StrokeStyle(lineWidth: 1.5, dash: [4, 4]))
                .frame(width: 26, height: 26)
        }
    }
}

/// Lays its children out in rows, wrapping to a new row when one is full.
struct FlowLayout: Layout {
    var spacing: CGFloat = 8

    func sizeThatFits(proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) -> CGSize {
        let rows = arrange(subviews, within: proposal.width ?? .infinity)
        return CGSize(width: rows.width, height: rows.height)
    }

    func placeSubviews(in bounds: CGRect, proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) {
        let rows = arrange(subviews, within: bounds.width)
        for (index, origin) in rows.origins.enumerated() {
            // Each row is centred.
            let shift = (bounds.width - rows.rowWidths[rows.rowOf[index]]) / 2
            subviews[index].place(
                at: CGPoint(x: bounds.minX + origin.x + shift, y: bounds.minY + origin.y),
                proposal: ProposedViewSize(width: bounds.width, height: nil))
        }
    }

    private func arrange(_ subviews: Subviews, within width: CGFloat)
        -> (origins: [CGPoint], rowOf: [Int], rowWidths: [CGFloat], width: CGFloat, height: CGFloat)
    {
        var origins: [CGPoint] = []
        var rowOf: [Int] = []
        var rowWidths: [CGFloat] = []
        var x: CGFloat = 0
        var y: CGFloat = 0
        var rowHeight: CGFloat = 0
        for subview in subviews {
            let size = subview.sizeThatFits(ProposedViewSize(width: width, height: nil))
            if x > 0, x + size.width > width {
                rowWidths.append(x - spacing)
                x = 0
                y += rowHeight + spacing
                rowHeight = 0
            }
            origins.append(CGPoint(x: x, y: y))
            rowOf.append(rowWidths.count)
            x += size.width + spacing
            rowHeight = max(rowHeight, size.height)
        }
        rowWidths.append(max(0, x - spacing))
        return (origins, rowOf, rowWidths, rowWidths.max() ?? 0, y + rowHeight)
    }
}
