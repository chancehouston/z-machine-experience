import SwiftUI

/// The one screen: status, the map's band, the transcript and the field, with
/// the keyboard up (specs/004-first-demo/contracts/screen.md, and for the band
/// and the keyboard specs/012-map-first-slice/contracts/map.md, part A).
struct GameView: View {
    @State private var game = Game()
    @State private var line = ""
    @State private var showingAbout = false
    /// Whether the command field has the keyboard. It has it whenever the game
    /// waits for a line, and gives it up only when the player scrolls back
    /// through the transcript; touching the field or sending takes it again.
    @State private var typing = true
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
                    // For tests of what is asked to be spoken: there is nothing
                    // to hear in a simulator.
                    if Announcer.always {
                        Color.clear
                            .frame(width: 1, height: 1)
                            .accessibilityElement()
                            .accessibilityLabel(Text(verbatim: game.lastSpoken ?? ""))
                            .accessibilityValue(Text(verbatim: String(game.timesSpoken)))
                            .accessibilityIdentifier("announcement.last")
                    }
                    Divider()
                    // The map: the sheet the player is on, about their place,
                    // with a control for each way out. Where the layer has no
                    // map for the game there is no band.
                    if let map = showing.map {
                        MapBand(map: map, ready: !game.busy) { index in
                            Task { await game.tap(way: index) }
                        }
                    }
                    // At the band's lower edge, whether or not the keyboard
                    // is up: what is carried, and the layer's words where the
                    // player is in the dark.
                    HStack(spacing: 8) {
                        if let note = showing.light.note {
                            Label {
                                Text(verbatim: note)
                            } icon: {
                                Image(systemName: "moon.fill")
                            }
                            .font(.footnote.weight(.semibold))
                            .fixedSize()
                            .accessibilityElement(children: .combine)
                            .accessibilityIdentifier("light.note")
                        }
                        InventoryStrip(
                            strip: showing.inventory, current: showing.waiting == .command)
                    }
                    .padding(.leading, showing.light.note == nil ? 0 : 12)
                    Divider()
                    TranscriptView(turns: game.turns, keyboardUp: typing)
                    Divider()
                    // The layer's words to show once: an earlier game that
                    // could not be brought back, or a place that is not kept.
                    if let notice = showing.notice {
                        Text(verbatim: notice)
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                            .multilineTextAlignment(.center)
                            .padding(.horizontal)
                            .padding(.top, 8)
                            .accessibilityIdentifier("notice")
                    }
                    bottomBar(showing)
                }
            }
        }
        .task { await game.start() }
        .sheet(isPresented: $showingAbout) { AboutView(game: game) }
    }

    @ViewBuilder
    private func bottomBar(_ showing: Snapshot) -> some View {
        switch showing.waiting {
        case .command, .answer:
            HStack(spacing: 8) {
                CommandField(
                    placeholder: showing.waiting == .command
                        ? "Type a command" : "Type your answer",
                    text: $line,
                    focused: $typing,
                    onReturn: send
                )
                .accessibilityIdentifier("field.command")
                .onChange(of: game.busy) { sendNextLine() }
                // The field has the keyboard from the moment it is there: at
                // launch, and again in a new game after one has ended.
                .onAppear { typing = true }
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
                // The layer's words: for a game that has ended, or a fault.
                Text(verbatim: showing.message ?? "")
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
///
/// The newest turn is shown from its first line: a reply longer than the room
/// is read from its start, with a sign that there is more below.
private struct TranscriptView: View {
    let turns: [Turn]
    /// Whether the command field has the keyboard. When it comes back, the
    /// newest text is brought back into view.
    let keyboardUp: Bool

    /// Where the transcript is scrolled to: the newest turn's first line,
    /// until the player moves it.
    @State private var position = ScrollPosition(edge: .top)
    /// How tall the room for the transcript is now.
    @State private var roomHeight: CGFloat = 0
    /// The moving of the transcript to the newest turn, while it is going on.
    @State private var moving: Task<Void, Never>?
    /// Whether there is text below what is shown.
    @State private var more = false
    /// Where the newest turn starts, and how tall it is.
    @State private var newest: NewestTurn.Start?

    var body: some View {
        GeometryReader { room in
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
                            // The newest turn says where it starts, so that
                            // the transcript can be moved there once it has
                            // been laid out.
                            .background {
                                if turn.number == turns.count - 1 {
                                    GeometryReader { spot in
                                        Color.clear.preference(
                                            key: NewestTurn.self,
                                            value: NewestTurn.Start(
                                                turn: turn.number,
                                                y: spot.frame(in: .named("transcript.content")).minY,
                                                height: spot.size.height))
                                    }
                                }
                            }
                    }
                }
                .coordinateSpace(.named("transcript.content"))
                .padding(.horizontal)
                .padding(.vertical, 8)
                // A transcript shorter than its room sits at the bottom, next
                // to the field, whenever it is drawn.
                .frame(minHeight: room.size.height, alignment: .bottom)
            }
            .scrollPosition($position)
            // Scrolling back lowers the keyboard, and the system does it:
            // the finger draws the text down into the keyboard, which goes
            // with it. Reading on through a long reply moves the finger the
            // other way and leaves the keyboard where it is. (A first fix
            // lowered it from here, by giving up the field's focus while the
            // finger was still down. The keyboard went and the screen kept
            // its room for it: found by the owner on a phone, 2026-11-14.)
            .scrollDismissesKeyboard(.interactively)
            .onScrollGeometryChange(for: Bool.self) { geometry in
                geometry.contentOffset.y + geometry.containerSize.height
                    < geometry.contentSize.height - 6
            } action: { _, below in
                more = below
            }
            // When the keyboard comes back the room is shorter: the newest
            // text is brought back into it.
            .onChange(of: keyboardUp) {
                if keyboardUp {
                    showNewest()
                }
            }
            // A new turn brings the newest text into view, wherever the
            // player had scrolled to: its first line where the reply is
            // taller than the room, and otherwise the whole of it, at the
            // end of the transcript. Its place is known only once it has
            // been laid out.
            .onPreferenceChange(NewestTurn.self) { start in
                guard let start else { return }
                let arrived = start.turn != newest?.turn
                newest = start
                // The opening of a new game is shown from the top as it is.
                if arrived, start.turn > 0 || turns.count > 1 {
                    showNewest(roomHeight: room.size.height)
                }
            }
            .background {
                // The room's height, for the moves asked for outside it.
                Color.clear.onChange(of: room.size.height, initial: true) {
                    roomHeight = room.size.height
                }
            }
            .accessibilityIdentifier("transcript")
            .overlay(alignment: .bottom) {
                if more {
                    Image(systemName: "chevron.compact.down")
                        .font(.title3.weight(.semibold))
                        .foregroundStyle(.secondary)
                        .padding(.horizontal, 10)
                        .background(Capsule().fill(Color(.systemBackground).opacity(0.85)))
                        .accessibilityIdentifier("transcript.more")
                        .allowsHitTesting(false)
                }
            }
        }
    }

    /// Brings the newest turn into view: its first line where it is taller
    /// than the room, and otherwise the whole of it, at the transcript's end.
    private func showNewest(roomHeight given: CGFloat? = nil) {
        guard let newest else { return }
        moving?.cancel()
        moving = Task { @MainActor in
            // The move is asked for three times over most of a second: one
            // asked for while the room is changing size (the keyboard
            // arriving) is lost.
            for attempt in 0..<3 {
                if attempt > 0 {
                    try? await Task.sleep(for: .milliseconds(350))
                }
                if Task.isCancelled { return }
                let tall = newest.height > (given ?? roomHeight)
                withAnimation(.easeOut(duration: 0.2)) {
                    if tall {
                        position.scrollTo(y: max(0, newest.y))
                    } else {
                        position.scrollTo(edge: .bottom)
                    }
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

/// Where the newest turn starts within the transcript's content.
private struct NewestTurn: PreferenceKey {
    struct Start: Equatable {
        let turn: Int
        let y: CGFloat
        let height: CGFloat
    }

    static let defaultValue: Start? = nil

    static func reduce(value: inout Start?, nextValue: () -> Start?) {
        value = nextValue() ?? value
    }
}
