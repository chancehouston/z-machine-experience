import SwiftUI

/// The app's name, what it is, and the bundled story's source and license as
/// the repository records them.
struct AboutView: View {
    @Environment(\.dismiss) private var dismiss

    private var license: String {
        guard let url = Bundle.main.url(forResource: "story-license", withExtension: "txt"),
            let text = try? String(contentsOf: url, encoding: .utf8)
        else { return "" }
        return text
    }

    var body: some View {
        NavigationStack {
            ScrollView {
                VStack(alignment: .leading, spacing: 16) {
                    Text(verbatim: "Lamplight")
                        .font(.largeTitle.bold())
                    Text("A demonstration: a classic text adventure with a compass that shows where you can go.")
                        .accessibilityIdentifier("about.demonstration")
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
