import SwiftUI

struct GameView: View {
    @State private var game = Game()

    var body: some View {
        ScrollView {
            LazyVStack(alignment: .leading) {
                ForEach(game.turns) { turn in
                    Text(verbatim: turn.text)
                        .accessibilityIdentifier("turn.\(turn.number).text")
                }
            }
        }
        .task { await game.start() }
    }
}
