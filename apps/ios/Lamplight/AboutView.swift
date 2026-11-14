import SwiftUI

/// The app's name, what it is, and the bundled story's source and license as
/// the repository records them.
struct AboutView: View {
    /// The game, for starting over.
    let game: Game
    @Environment(\.dismiss) private var dismiss
    @State private var asking = false

    private var license: String {
        guard let url = Bundle.main.url(forResource: "story-license", withExtension: "txt"),
            let text = try? String(contentsOf: url, encoding: .utf8)
        else { return "" }
        return Self.reflowed(text)
    }

    /// The notice with each paragraph as one run of text. Its own line ends
    /// are for a wide page, and on a phone every line would wrap twice. The
    /// words and the paragraphs are unchanged.
    static func reflowed(_ notice: String) -> String {
        notice
            .replacingOccurrences(of: "\r\n", with: "\n")
            .components(separatedBy: "\n\n")
            .map { paragraph in
                paragraph
                    .split(separator: "\n")
                    .map { $0.trimmingCharacters(in: .whitespaces) }
                    .joined(separator: " ")
            }
            .filter { !$0.isEmpty }
            .joined(separator: "\n\n")
    }

    private func startOver() {
        Task {
            await game.newGame()
            dismiss()
        }
    }

    var body: some View {
        NavigationStack {
            ScrollView {
                VStack(alignment: .leading, spacing: 16) {
                    Text(verbatim: "Lamplight")
                        .font(.largeTitle.bold())
                    Text("A demonstration: a classic text adventure with a map that draws itself as you walk.")
                        .accessibilityIdentifier("about.demonstration")
                    // Giving up a game in progress is asked about first, in
                    // the layer's words. Where the layer has no question
                    // there is nothing to give up.
                    Button("Start over", role: .destructive) {
                        if game.showing?.startOver == nil {
                            startOver()
                        } else {
                            asking = true
                        }
                    }
                    .disabled(game.showing == nil || game.busy)
                    .accessibilityIdentifier("button.startOver")
                    .confirmationDialog(
                        Text(verbatim: game.showing?.startOver ?? ""),
                        isPresented: $asking, titleVisibility: .visible
                    ) {
                        Button("Start over", role: .destructive) { startOver() }
                            .accessibilityIdentifier("button.startOverConfirm")
                        Button("Cancel", role: .cancel) {}
                    }
                    Divider()
                    Text("The bundled story")
                        .font(.headline)
                    Text(verbatim: license)
                        .font(.footnote)
                        .accessibilityIdentifier("about.license")
                }
                .frame(maxWidth: .infinity, alignment: .leading)
                .padding()
            }
            .toolbar {
                ToolbarItem(placement: .confirmationAction) {
                    Button("Done") { dismiss() }
                        .accessibilityIdentifier("button.aboutDone")
                }
            }
        }
    }
}
