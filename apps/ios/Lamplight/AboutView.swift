import SwiftUI

/// The app's name, what it is, and the bundled story's source and license as
/// the repository records them.
struct AboutView: View {
    @Environment(\.dismiss) private var dismiss

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
