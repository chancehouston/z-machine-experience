import SwiftUI

/// The map as it is shown above the game's reply while the keyboard is up: the
/// sheet the player is on, moved so that their place is in the middle, with a
/// control for each way out of it
/// (specs/012-map-first-slice/contracts/map.md, part A).
struct MapBand: View {
    let map: MapSheet
    /// False while a command is being worked out.
    let ready: Bool
    let tapWay: (Int) -> Void

    /// The middle of the place the player is in; or, where that is not
    /// known, of the first place of the sheet.
    private var focus: CGPoint {
        let place = map.places.first { $0.id == map.here } ?? map.places.first
        return place.map(MapLayout.point) ?? .zero
    }

    var body: some View {
        GeometryReader { room in
            let middle = CGPoint(x: room.size.width / 2, y: room.size.height / 2)
            ZStack(alignment: .topLeading) {
                MapDrawing(map: map)
                    .offset(x: middle.x - focus.x, y: middle.y - focus.y)
                if map.here != nil {
                    ways(around: middle)
                }
                extras
                if let note = map.note {
                    Text(verbatim: note)
                        .font(.footnote)
                        .foregroundStyle(.secondary)
                        .padding(.horizontal, 8)
                        .padding(.vertical, 2)
                        .background(Capsule().fill(Color(.systemBackground).opacity(0.9)))
                        .accessibilityIdentifier("map.note")
                        .position(x: middle.x, y: room.size.height - 14)
                }
            }
            .frame(width: room.size.width, height: room.size.height, alignment: .topLeading)
            .clipped()
        }
        .frame(height: MapLayout.band)
        .background(Color(.secondarySystemBackground))
        // The band's own frame, for tests: the element that holds the
        // drawing is as large as what is drawn in it.
        .background {
            Color.clear
                .accessibilityElement()
                .accessibilityIdentifier("map.frame")
        }
        .accessibilityElement(children: .contain)
        .accessibilityLabel(Text(verbatim: map.spoken))
        .accessibilityIdentifier("map")
    }

    /// The twelve ways, each at its place round the middle.
    private func ways(around middle: CGPoint) -> some View {
        ForEach(map.ways.indices, id: \.self) { index in
            let way = map.ways[index]
            if let offset = MapLayout.controlOffset(for: way.way) {
                WayControl(way: way, identifier: "compass.\(way.way)", ready: ready) {
                    tapWay(index)
                }
                .position(x: middle.x + offset.width, y: middle.y + offset.height)
            }
        }
    }

    /// Ways that are none of the twelve: a row of buttons at the band's top.
    private var extras: some View {
        let indices = map.ways.indices.filter { MapLayout.controlOffset(for: map.ways[$0].way) == nil }
        return HStack(spacing: 6) {
            ForEach(Array(indices.enumerated()), id: \.element) { number, index in
                let way = map.ways[index]
                Button {
                    tapWay(index)
                } label: {
                    Text(verbatim: way.label).font(.footnote)
                }
                .buttonStyle(.bordered)
                .disabled(way.command == nil || !ready)
                .accessibilityLabel(Text(verbatim: way.spoken))
                .accessibilityValue(Text(verbatim: way.state))
                .accessibilityIdentifier("compass.extra.\(number)")
            }
        }
        .padding(6)
    }
}
