import SwiftUI

/// Draws what the snapshot says the player carries, in one row. Every word in
/// it is the layer's; nothing here sends anything to the game.
struct InventoryStrip: View {
    let strip: Strip
    /// False when the game is not waiting for a command: the row is shown as
    /// the compass is at such times.
    let current: Bool
    /// A touch on an item that can be touched: its place among the items.
    let tap: (Int) -> Void

    var body: some View {
        Group {
            if let note = strip.note {
                Text(verbatim: note)
                    .font(.footnote)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity)
                    .accessibilityIdentifier("inventory.note")
            } else {
                ScrollView(.horizontal, showsIndicators: false) {
                    HStack(spacing: 8) {
                        ForEach(strip.items.indices, id: \.self) { index in
                            StripGroup(item: strip.items[index], current: current, tap: tap)
                        }
                    }
                    .padding(.horizontal, 16)
                }
                // Where the row runs past the edge, it fades: there is more.
                .mask(
                    LinearGradient(
                        stops: [
                            .init(color: .black, location: 0),
                            .init(color: .black, location: 0.92),
                            .init(color: .clear, location: 1),
                        ],
                        startPoint: .leading, endPoint: .trailing)
                )
                // For a screen reader the strip says the layer's words, and
                // holds each item that can be touched.
                .accessibilityElement(children: .contain)
                .accessibilityLabel(Text(verbatim: strip.spoken))
                .accessibilityIdentifier("inventory.strip")
            }
        }
        .frame(height: 40)
        .opacity(current ? 1 : 0.45)
    }
}

/// One item, and after it what is shown inside it, drawn as one group.
private struct StripGroup: View {
    let item: StripItem
    let current: Bool
    let tap: (Int) -> Void

    var body: some View {
        if item.contents.isEmpty {
            StripChip(item: item, current: current, tap: tap)
        } else {
            HStack(spacing: 6) {
                StripChip(item: item, current: current, tap: tap)
                ForEach(item.contents.indices, id: \.self) { index in
                    StripGroup(item: item.contents[index], current: current, tap: tap)
                }
            }
            .padding(.trailing, 6)
            .background(Capsule().fill(Color.secondary.opacity(0.12)))
        }
    }
}

/// An item's name, with the mark of a closed way when it is closed.
private struct StripChip: View {
    let item: StripItem
    let current: Bool
    let tap: (Int) -> Void

    var body: some View {
        // Where the snapshot says the item has a card now, it is a button.
        if let index = item.index, item.card == true, current {
            Button {
                tap(index)
            } label: {
                chip
            }
            .buttonStyle(.plain)
            .accessibilityLabel(Text(verbatim: item.name))
            .accessibilityIdentifier("item.\(index)")
        } else {
            chip
        }
    }

    private var chip: some View {
        HStack(spacing: 5) {
            if item.state == .closed {
                RoundedRectangle(cornerRadius: 2)
                    .strokeBorder(Color.secondary, lineWidth: 1.5)
                    .frame(width: 9, height: 12)
            }
            Text(verbatim: item.name)
                .font(.footnote)
                .lineLimit(1)
                .fixedSize()
        }
        .padding(.horizontal, 10)
        .padding(.vertical, 6)
        .overlay(Capsule().strokeBorder(Color.secondary.opacity(0.5), lineWidth: 1))
        .contentShape(Capsule())
    }
}
