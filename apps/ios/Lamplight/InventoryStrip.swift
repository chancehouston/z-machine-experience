import SwiftUI

/// Draws what the snapshot says the player carries, in one row. Every word in
/// it is the layer's; nothing here sends anything to the game.
struct InventoryStrip: View {
    let strip: Strip
    /// False when the game is not waiting for a command: the row is shown as
    /// the compass is at such times.
    let current: Bool

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
                            StripGroup(item: strip.items[index])
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
                // One element for a screen reader, which says the layer's words.
                .accessibilityElement(children: .ignore)
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

    var body: some View {
        if item.contents.isEmpty {
            StripChip(item: item)
        } else {
            HStack(spacing: 6) {
                StripChip(item: item)
                ForEach(item.contents.indices, id: \.self) { index in
                    StripGroup(item: item.contents[index])
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

    var body: some View {
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
    }
}
