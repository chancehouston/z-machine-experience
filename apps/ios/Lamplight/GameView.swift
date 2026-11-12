import SwiftUI

/// The one screen: status, transcript, compass and field
/// (specs/004-first-demo/contracts/screen.md).
struct GameView: View {
    @State private var game = Game()
    @State private var line = ""
    @State private var showingAbout = false
    @State private var typing = false
    /// Lines typed and not yet sent: one is sent at a time, in order.
    @State private var waitingLines: [String] = []
    @State private var sending = false

    /// `-showLaunchTime` adds the time the launch took to the screen.
    private let showsLaunchTime = ProcessInfo.processInfo.arguments.contains("-showLaunchTime")

    var body: some View {
        Group {
            if let failure = game.startFailure {
                VStack(spacing: 12) {
                    Text("The game could not be started")
                        .font(.headline)
                    Text(verbatim: failure)
                        .foregroundStyle(.secondary)
                }
                .multilineTextAlignment(.center)
                .padding()
            } else if let showing = game.showing {
                VStack(spacing: 0) {
                    StatusBar(status: showing.status) { showingAbout = true }
                    if showsLaunchTime, let millis = game.launchMillis {
                        Text(verbatim: String(millis))
                            .font(.caption.monospacedDigit())
                            .accessibilityIdentifier("debug.launchMillis")
                    }
                    Divider()
                    TranscriptView(turns: game.turns, keyboardUp: typing)
                    Divider()
                    // With the keyboard showing, the compass and the strip of
                    // what is carried give their room to the transcript and the
                    // field.
                    if !typing {
                        InventoryStrip(
                            strip: showing.inventory, current: showing.waiting == .command)
                        Divider()
                        CompassView(
                            compass: showing.compass, ready: !game.busy,
                            tapPlace: { place in Task { await game.tap(place: place) } },
                            tapExtra: { index in Task { await game.tap(extra: index) } })
                        Divider()
                    }
                    bottomBar(showing)
                }
            }
        }
        .task { await game.start() }
        .sheet(isPresented: $showingAbout) { AboutView() }
    }

    @ViewBuilder
    private func bottomBar(_ showing: Snapshot) -> some View {
        switch showing.waiting {
        case .command, .answer:
            HStack(spacing: 8) {
                // Beside the field rather than on the keyboard's own bar: there
                // the system floats it over the send button.
                if typing {
                    Button {
                        typing = false
                    } label: {
                        Image(systemName: "keyboard.chevron.compact.down")
                            .font(.title3)
                    }
                    .accessibilityLabel("Show compass")
                    .accessibilityIdentifier("button.compass")
                }
                CommandField(
                    placeholder: showing.waiting == .command
                        ? "Type a command" : "Type your answer",
                    text: $line,
                    focused: $typing,
                    onReturn: send
                )
                .accessibilityIdentifier("field.command")
                .onChange(of: game.busy) { sendNextLine() }
                // When the game ends the field goes, and the keyboard with it:
                // a new game starts with the compass showing.
                .onDisappear { typing = false }
                Button {
                    let typed = line
                    line = ""
                    send(typed)
                } label: {
                    Image(systemName: "arrow.up.circle.fill")
                        .font(.title)
                }
                .disabled(line.isEmpty || game.busy)
                .accessibilityLabel("Send")
                .accessibilityIdentifier("button.send")
            }
            .padding(.horizontal)
            .padding(.vertical, 8)
        case .ended, .fault:
            VStack(spacing: 8) {
                if showing.waiting == .ended {
                    Text("The game has ended.")
                } else {
                    Text(verbatim: showing.message ?? "")
                }
                Button("New game") {
                    Task { await game.newGame() }
                }
                .buttonStyle(.borderedProminent)
                .disabled(game.busy)
                .accessibilityIdentifier("button.newGame")
            }
            .padding()
        }
    }

    /// The one place a typed line is sent from: the field's text, as it is.
    /// Sending does not put the keyboard away.
    private func send(_ typed: String) {
        typing = true
        guard !typed.isEmpty else { return }
        waitingLines.append(typed)
        sendNextLine()
    }

    /// Sends the oldest waiting line, once the command before it has been
    /// answered. Lines still waiting when the game ends are dropped.
    private func sendNextLine() {
        guard !sending, !game.busy, !waitingLines.isEmpty else { return }
        guard let waiting = game.showing?.waiting, waiting == .command || waiting == .answer else {
            waitingLines.removeAll()
            return
        }
        let typed = waitingLines.removeFirst()
        sending = true
        Task {
            await game.send(typed: typed)
            sending = false
            sendNextLine()
        }
    }
}

/// The place, and the score and moves or the time, as the game reports them.
private struct StatusBar: View {
    let status: Status?
    let about: () -> Void

    @Environment(\.dynamicTypeSize) private var typeSize

    var body: some View {
        Group {
            if typeSize.isAccessibilitySize {
                // At the largest sizes the figures go under the place.
                VStack(alignment: .leading, spacing: 4) {
                    HStack(alignment: .top, spacing: 12) {
                        place.lineLimit(2)
                        Spacer(minLength: 8)
                        aboutButton
                    }
                    HStack(spacing: 16) { figures }
                }
            } else {
                HStack(spacing: 12) {
                    place.lineLimit(1)
                    Spacer(minLength: 8)
                    figures
                    aboutButton
                }
            }
        }
        .padding(.horizontal)
        .padding(.vertical, 8)
    }

    private var place: some View {
        Text(verbatim: status?.place ?? "")
            .font(.headline)
            .accessibilityIdentifier("status.place")
    }

    @ViewBuilder
    private var figures: some View {
        if let score = status?.score {
            figure("Score", String(score), "status.score")
        }
        if let moves = status?.moves {
            figure("Moves", String(moves), "status.moves")
        }
        if let hours = status?.hours, let minutes = status?.minutes {
            Text(verbatim: String(hours) + ":" + (minutes < 10 ? "0" : "") + String(minutes))
                .font(.subheadline.monospacedDigit())
                .lineLimit(1)
                .fixedSize()
                .accessibilityIdentifier("status.time")
        }
    }

    private var aboutButton: some View {
        Button(action: about) {
            Image(systemName: "info.circle")
        }
        .accessibilityLabel("About")
        .accessibilityIdentifier("button.about")
    }

    private func figure(_ name: LocalizedStringKey, _ number: String, _ identifier: String) -> some View {
        HStack(spacing: 4) {
            Text(name).foregroundStyle(.secondary)
            Text(verbatim: number).monospacedDigit()
        }
        .font(.subheadline)
        .lineLimit(1)
        .fixedSize()
        .accessibilityElement(children: .ignore)
        .accessibilityLabel(Text(name))
        .accessibilityValue(Text(verbatim: number))
        .accessibilityIdentifier(identifier)
    }
}

/// Every turn in order. A turn is the game's text as printed; the command that
/// led to the next turn is written straight after the prompt that ends it, as
/// on a terminal, and set apart as the player's.
private struct TranscriptView: View {
    let turns: [Turn]
    let keyboardUp: Bool

    private let end = "transcript.end"

    var body: some View {
        GeometryReader { room in
            ScrollViewReader { proxy in
                ScrollView {
                    VStack(alignment: .leading, spacing: 0) {
                        ForEach(turns) { turn in
                            let next = turn.number + 1 < turns.count ? turns[turn.number + 1].command : nil
                            Text(drawn(turn, then: next))
                                .frame(maxWidth: .infinity, alignment: .leading)
                                .accessibilityLabel(Text(verbatim: turn.text))
                                .accessibilityIdentifier("turn.\(turn.number).text")
                                .overlay(alignment: .bottomTrailing) {
                                    if let next {
                                        Color.clear
                                            .frame(width: 1, height: 1)
                                            .accessibilityElement()
                                            .accessibilityLabel(Text(verbatim: next))
                                            .accessibilityIdentifier("turn.\(turn.number + 1).command")
                                    }
                                }
                        }
                        Color.clear.frame(height: 1).id(end)
                    }
                    .padding(.horizontal)
                    .padding(.vertical, 8)
                    // A transcript shorter than its room sits at the bottom, next
                    // to the compass and the field, whenever it is drawn.
                    .frame(minHeight: room.size.height, alignment: .bottom)
                }
                .defaultScrollAnchor(.bottom)
                .accessibilityIdentifier("transcript")
                .onChange(of: turns.count) {
                    withAnimation { proxy.scrollTo(end, anchor: .bottom) }
                }
                .onChange(of: keyboardUp) {
                    proxy.scrollTo(end, anchor: .bottom)
                }
            }
        }
    }

    private func drawn(_ turn: Turn, then command: String?) -> AttributedString {
        var text = AttributedString(turn.text)
        if let command {
            var typed = AttributedString(command)
            typed.inlinePresentationIntent = .stronglyEmphasized
            typed.foregroundColor = .accentColor
            text.append(typed)
        }
        return text
    }
}
