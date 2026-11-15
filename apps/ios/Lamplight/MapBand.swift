import SwiftUI

/// The map: the sheet the player is on, with a control for each way out of
/// the place they are in. While the keyboard is up it is a band above the
/// game's reply, moved so that their place is in the middle. Opened out, it
/// has the transcript's room as well and can be dragged about
/// (specs/012-map-first-slice/contracts/map.md, part A).
struct MapBand: View {
    let map: MapSheet
    /// A place to put in the middle, where the sheet was turned to by a way
    /// that leads to it.
    let centre: Int?
    /// False while a command is being worked out.
    let ready: Bool
    /// Whether the sheet is opened out.
    let open: Bool
    let setOpen: (Bool) -> Void
    /// Asks for the sheet to be laid out again around the player.
    let redraw: () -> Void
    /// Turns the opened map to another sheet, with a place of it to centre.
    let turnSheet: (Int, Int?) -> Void
    let tapWay: (Int) -> Void
    /// A touch on the place the player is in.
    let tapPlace: () -> Void

    @Environment(\.accessibilityReduceMotion) private var reduceMotion
    /// How far the opened sheet has been dragged from having the player's
    /// place in the middle.
    @State private var pan: CGSize = .zero
    @GestureState private var drag: CGSize = .zero

    private var still: Bool { reduceMotion || MapHand.stillAsked }
    private var glide: Animation? { still ? nil : .easeInOut(duration: 0.25) }

    /// What is put in the middle: the place the sheet was turned to for;
    /// else the place the player is in; else the first place found on the
    /// sheet.
    private var focus: CGPoint {
        let wanted = [centre, map.here, map.first].compactMap { $0 }
        let place =
            wanted.lazy.compactMap { id in map.places.first { $0.id == id } }.first
            ?? map.places.first
        return place.map(MapLayout.point) ?? .zero
    }

    /// A drag kept within the sheet: the point of the sheet at the middle of
    /// the view stays among the places drawn.
    private func limited(_ shift: CGSize) -> CGSize {
        let points = map.places.map(MapLayout.point)
        guard let left = points.map(\.x).min(), let right = points.map(\.x).max(),
            let top = points.map(\.y).min(), let bottom = points.map(\.y).max()
        else { return .zero }
        return CGSize(
            width: min(max(shift.width, focus.x - right), focus.x - left),
            height: min(max(shift.height, focus.y - bottom), focus.y - top))
    }

    var body: some View {
        GeometryReader { room in
            let shift =
                open
                ? limited(CGSize(width: pan.width + drag.width, height: pan.height + drag.height))
                : .zero
            let middle = CGPoint(
                x: room.size.width / 2 + shift.width, y: room.size.height / 2 + shift.height)
            ZStack(alignment: .topLeading) {
                MapDrawing(
                    map: map, still: still, ready: ready, tapPlace: tapPlace,
                    turnSheet: open ? turnSheet : nil)
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
                        .position(x: room.size.width / 2, y: room.size.height - 14)
                }
                toggle
                    .position(x: room.size.width - 26, y: 26)
                // Redraws the sheet around the player, where the layer
                // offers it: its words are the layer's.
                if let words = map.redraw {
                    Button(action: redraw) {
                        Image(systemName: "arrow.triangle.2.circlepath")
                            .font(.system(size: 13, weight: .semibold))
                            .frame(width: 30, height: 30)
                            .background(Circle().fill(Color(.systemBackground).opacity(0.9)))
                            .frame(width: MapLayout.control, height: MapLayout.control)
                            .contentShape(Rectangle())
                    }
                    .disabled(!ready)
                    .accessibilityLabel(Text(verbatim: words))
                    .accessibilityIdentifier("button.map.redraw")
                    .position(x: 26, y: room.size.height - 26)
                }
                if open {
                    sheets.position(x: room.size.width / 2, y: 26)
                    if let back = map.back {
                        turner(to: back, symbol: "location.fill", identifier: "button.sheet.return")
                            .position(x: room.size.width - 26, y: room.size.height - 26)
                    }
                }
                // For tests: whether the map is drawn in and glided, or still.
                if MapHand.shown {
                    Color.clear
                        .frame(width: 1, height: 1)
                        .accessibilityElement()
                        .accessibilityValue(Text(verbatim: still ? "reduced" : "full"))
                        .accessibilityIdentifier("map.motion")
                }
            }
            .frame(width: room.size.width, height: room.size.height, alignment: .topLeading)
            .contentShape(Rectangle())
            .clipped()
            .gesture(dragging)
            // The view glides to the place the player has walked to.
            .animation(glide, value: map.here)
        }
        .frame(height: open ? nil : MapLayout.band)
        .frame(maxHeight: open ? .infinity : nil)
        .background(Color(.secondarySystemBackground))
        // The map's own frame, for tests: the element that holds the
        // drawing is as large as what is drawn in it.
        .background {
            Color.clear
                .accessibilityElement()
                .accessibilityIdentifier("map.frame")
        }
        .accessibilityElement(children: .contain)
        .accessibilityLabel(Text(verbatim: map.spoken))
        .accessibilityIdentifier("map")
        // A move, another sheet, or opening and closing puts the player's
        // place back in the middle.
        .onChange(of: map.here) {
            withAnimation(glide) { pan = .zero }
        }
        .onChange(of: map.sheet) { pan = .zero }
        .onChange(of: open) { pan = .zero }
    }

    /// Opened out, a drag moves the sheet. As a band, pulling it down opens
    /// it out, once the finger is lifted.
    private var dragging: some Gesture {
        DragGesture(minimumDistance: 10)
            .updating($drag) { value, state, _ in
                if open {
                    state = value.translation
                }
            }
            .onEnded { value in
                if open {
                    pan = limited(
                        CGSize(
                            width: pan.width + value.translation.width,
                            height: pan.height + value.translation.height))
                } else if value.translation.height > 40,
                    value.translation.height > abs(value.translation.width)
                {
                    setOpen(true)
                }
            }
    }

    /// Which sheet is shown, in the layer's words, between the controls that
    /// turn to the sheet before and the sheet after (contract A§2).
    private var sheets: some View {
        HStack(spacing: 2) {
            if let before = map.before {
                turner(to: before, symbol: "chevron.left", identifier: "button.sheet.before")
            }
            if let label = map.label {
                Text(verbatim: label)
                    .font(.footnote.weight(.medium))
                    .lineLimit(1)
                    .minimumScaleFactor(0.7)
                    .padding(.horizontal, 8)
                    .padding(.vertical, 4)
                    .background(Capsule().fill(Color(.systemBackground).opacity(0.9)))
                    .accessibilityIdentifier("map.sheet.label")
            }
            if let after = map.after {
                turner(to: after, symbol: "chevron.right", identifier: "button.sheet.after")
            }
        }
    }

    /// A control that turns to another sheet; a screen reader is given the
    /// layer's words for that sheet.
    private func turner(to sheet: SheetRef, symbol: String, identifier: String) -> some View {
        Button {
            turnSheet(sheet.number, nil)
        } label: {
            Image(systemName: symbol)
                .font(.system(size: 14, weight: .semibold))
                .frame(width: 30, height: 30)
                .background(Circle().fill(Color(.systemBackground).opacity(0.9)))
                .frame(width: MapLayout.control, height: MapLayout.control)
                .contentShape(Rectangle())
        }
        .accessibilityLabel(Text(verbatim: sheet.label))
        .accessibilityIdentifier(identifier)
    }

    /// Opens the sheet out, or brings the band back.
    private var toggle: some View {
        Button {
            setOpen(!open)
        } label: {
            Image(
                systemName: open
                    ? "arrow.down.right.and.arrow.up.left" : "arrow.up.left.and.arrow.down.right"
            )
            .font(.system(size: 14, weight: .semibold))
            .frame(width: 30, height: 30)
            .background(Circle().fill(Color(.systemBackground).opacity(0.9)))
            .frame(width: MapLayout.control, height: MapLayout.control)
            .contentShape(Rectangle())
        }
        .accessibilityLabel(open ? "Close map" : "Open map")
        .accessibilityIdentifier(open ? "button.closeMap" : "button.openMap")
    }

    /// The twelve ways, each at its place round the place the player is in.
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

    /// Ways that are none of the twelve: a row of buttons at the map's top.
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
