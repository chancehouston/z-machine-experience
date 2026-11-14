import SwiftUI

/// The card for something the player carries, shown over the map: a token,
/// the game's name for the item, what the game says of it (or the layer's
/// words where it could not be had), and a button for each thing to do with it
/// (specs/014-inventory-card/contracts/card.md, part A). Every word on it is
/// the game's or the layer's; a touch is handed to the closures it is given.
///
/// Closing is what a touch does unless it is on an action: on the close
/// control, anywhere else on the card, or on the band beside it. An action is
/// small and must be aimed at (found by the owner on a phone: with a large
/// button and a small close control, an item was dropped by mistake). The
/// close control is at the card's bottom left, with the actions to its right,
/// as the owner asked.
struct ItemCard: View {
    let card: Card
    /// The item's place among the shown items: its token is drawn from it.
    let index: Int
    /// False while a command is being worked out.
    let ready: Bool
    let tapAction: (Int) -> Void
    let close: () -> Void

    /// Whether there is text below what is shown.
    @State private var more = false

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack(alignment: .center, spacing: 10) {
                ItemToken(
                    seed: index, kind: card.kind, state: card.state, lit: card.lit,
                    probe: "card.token")
                name
                Spacer(minLength: 8)
            }
            .padding(.top, 10)
            .padding(.leading, 8)
            ScrollView {
                text
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .padding(.horizontal, 8)
            }
            .onScrollGeometryChange(for: Bool.self) { geometry in
                geometry.contentOffset.y + geometry.containerSize.height
                    < geometry.contentSize.height - 4
            } action: { _, below in
                more = below
            }
            // A description taller than its room has the mark the transcript
            // has: there is more below.
            .overlay(alignment: .bottom) {
                if more {
                    Image(systemName: "chevron.compact.down")
                        .font(.title3.weight(.semibold))
                        .foregroundStyle(.secondary)
                        .padding(.horizontal, 10)
                        .background(Capsule().fill(Color(.systemBackground).opacity(0.85)))
                        .accessibilityIdentifier("card.more")
                        .allowsHitTesting(false)
                }
            }
            controls
        }
        .padding(.horizontal, 6)
        .padding(.bottom, 4)
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        // The card stands out from the band: paper, with an edge drawn in
        // the map's hand and a little shadow.
        .background {
            PencilBox(seed: index &+ 31)
                .fill(Color(.systemBackground))
                .shadow(color: .black.opacity(0.18), radius: 5, y: 2)
        }
        .overlay {
            PencilBox(seed: index &+ 31)
                .stroke(
                    Color.primary.opacity(0.82),
                    style: StrokeStyle(lineWidth: 1.6, lineCap: .round, lineJoin: .round)
                )
                .allowsHitTesting(false)
        }
        .contentShape(Rectangle())
        .onTapGesture(perform: close)
        .accessibilityElement(children: .contain)
        .accessibilityIdentifier("card")
        .padding(.horizontal, 10)
        .padding(.vertical, 8)
        // The band about the card: the map is covered, and a touch there
        // closes the card too.
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background {
            Color(.secondarySystemBackground)
                .contentShape(Rectangle())
                .onTapGesture(perform: close)
        }
    }

    private var name: some View {
        Text(verbatim: card.name)
            .font(.headline)
            .lineLimit(2)
            .minimumScaleFactor(0.7)
            .accessibilityIdentifier("card.name")
    }

    /// The game's reply, or the layer's words where there is none.
    private var text: some View {
        Text(verbatim: card.description ?? card.note ?? "")
            .font(.callout)
            .foregroundStyle(card.description == nil ? .secondary : .primary)
            .fixedSize(horizontal: false, vertical: true)
            .accessibilityIdentifier("card.text")
    }

    /// The card's controls, in one row at its foot: the close control first,
    /// at the bottom left where the thumb is, then clear space, then each
    /// action in the layer's order. The actions scroll sideways where there
    /// are more than the card has room for; the close control stays put.
    private var controls: some View {
        HStack(spacing: 0) {
            Button(action: close) {
                Image(systemName: "xmark")
                    .font(.system(size: 13, weight: .semibold))
                    .frame(width: 30, height: 30)
                    .background(Circle().fill(Color(.secondarySystemBackground)))
                    .frame(width: MapLayout.control, height: MapLayout.control)
                    .contentShape(Rectangle())
            }
            .accessibilityLabel("Close")
            .accessibilityIdentifier("card.close")
            // Clear space before the first action: a thumb reaching across
            // the phone for the close control does not land on an action
            // (the owner's trial).
            .padding(.trailing, MapLayout.control)
            ScrollView(.horizontal, showsIndicators: false) {
                actions
            }
        }
    }

    private var actions: some View {
        HStack(spacing: 4) {
            ForEach(card.actions.indices, id: \.self) { index in
                let action = card.actions[index]
                Button {
                    tapAction(index)
                } label: {
                    // Small to look at; as tall to touch as any control.
                    Text(verbatim: action.label)
                        .font(.footnote.weight(.medium))
                        .lineLimit(1)
                        .padding(.horizontal, 12)
                        .padding(.vertical, 5)
                        .overlay(Capsule().strokeBorder(.tint, lineWidth: 1.2))
                        .frame(minWidth: MapLayout.control, minHeight: MapLayout.control)
                        .contentShape(Rectangle())
                }
                .buttonStyle(.plain)
                .foregroundStyle(.tint)
                .disabled(!ready)
                .accessibilityLabel(Text(verbatim: action.spoken))
                .accessibilityIdentifier("card.action.\(index)")
            }
        }
    }
}
