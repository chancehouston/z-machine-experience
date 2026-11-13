import XCTest

/// Tests of the screen, against specs/004-first-demo/contracts/screen.md.
/// Every game is started with a fixed seed so that it is the same game each time.
final class LamplightUITests: XCTestCase {
    override func setUp() {
        continueAfterFailure = false
    }

    @MainActor
    private func launch(_ arguments: [String] = ["-seed", "11"]) -> XCUIApplication {
        let app = XCUIApplication()
        app.launchArguments = arguments
        app.launch()
        return app
    }

    /// How long to wait for something that is expected to appear: five seconds.
    /// TEST_RUNNER_WAIT_SECONDS gives another, for a machine that is not the
    /// measure of the app: a shared CI runner once took longer to show one turn.
    private let patience: TimeInterval = {
        let given = ProcessInfo.processInfo.environment["WAIT_SECONDS"]
        return given.flatMap { TimeInterval($0) } ?? 5
    }()

    /// The element with this accessibility identifier, of whatever kind.
    @MainActor
    private func element(_ app: XCUIApplication, _ identifier: String) -> XCUIElement {
        app.descendants(matching: .any).matching(identifier: identifier).firstMatch
    }

    // S§ "Starting", "For automated tests"; FR-001, FR-002; US1 scenario 1.
    @MainActor
    func testOpeningIsShown() {
        let app = launch()
        let opening = element(app, "turn.0.text")
        XCTAssertTrue(opening.waitForExistence(timeout: patience), "the opening text is shown")
        XCTAssertTrue(opening.label.contains("West of House"), opening.label)
        XCTAssertTrue(opening.label.hasSuffix(">"), "the text ends with the game's prompt")
    }

    private let places = [
        "north", "northeast", "east", "southeast", "south", "southwest", "west", "northwest",
        "up", "down", "in", "out",
    ]

    /// Types a line in the field and sends it.
    @MainActor
    private func type(_ app: XCUIApplication, _ line: String) {
        let field = element(app, "field.command")
        XCTAssertTrue(field.waitForExistence(timeout: patience))
        field.tap()
        field.typeText(line)
        element(app, "button.send").tap()
    }

    /// Waits for turn `number` to be on screen and returns its text.
    @MainActor
    private func turn(_ app: XCUIApplication, _ number: Int) -> String {
        let text = element(app, "turn.\(number).text")
        XCTAssertTrue(text.waitForExistence(timeout: patience), "turn \(number) is shown")
        return text.label
    }

    @MainActor
    private func value(_ app: XCUIApplication, _ identifier: String) -> String {
        let found = element(app, identifier)
        XCTAssertTrue(found.waitForExistence(timeout: patience), identifier)
        return found.value as? String ?? ""
    }

    /// Waits until the place in the status line is this.
    @MainActor
    private func waitForPlace(_ app: XCUIApplication, _ place: String) {
        let status = element(app, "status.place")
        let arrived = NSPredicate(format: "label == %@", place)
        expectation(for: arrived, evaluatedWith: status)
        waitForExpectations(timeout: patience)
    }

    // S§ "Layout"; FR-003; US1 scenario 2.
    @MainActor
    func testStatusAtLaunch() {
        let app = launch()
        _ = turn(app, 0)
        XCTAssertEqual(element(app, "status.place").label, "West of House")
        XCTAssertEqual(value(app, "status.score"), "0")
        XCTAssertEqual(value(app, "status.moves"), "0")
    }

    // FR-005; US1 scenario 4: a new process is a new game.
    @MainActor
    func testEachLaunchIsANewGame() {
        let app = launch()
        _ = turn(app, 0)
        element(app, "compass.north").tap()
        _ = turn(app, 1)
        app.terminate()
        app.launch()
        _ = turn(app, 0)
        // Given time to appear, nothing of the last game does.
        XCTAssertFalse(
            element(app, "turn.1.text").waitForExistence(timeout: 2),
            "nothing of the last game is shown")
        XCTAssertEqual(element(app, "status.place").label, "West of House")
        XCTAssertEqual(value(app, "status.moves"), "0")
    }

    // S§ "The compass", "For automated tests"; FR-007, FR-008, FR-009, FR-012;
    // US2 scenarios 1, 5, 6. The states are those stories/…/introspection/exits.txt
    // gives for the opening and for Behind House.
    @MainActor
    func testCompassAtTheOpening() {
        let app = launch()
        _ = turn(app, 0)
        for place in places {
            XCTAssertTrue(element(app, "compass.\(place)").exists, place)
        }
        for place in ["north", "south", "west"] {
            XCTAssertEqual(value(app, "compass.\(place)"), "open", place)
        }
        XCTAssertEqual(value(app, "compass.east"), "closed")
        XCTAssertEqual(element(app, "compass.north").label, "North, to North of House")
        XCTAssertEqual(element(app, "compass.east").label, "East, no exit")
        element(app, "compass.east").tap()
        XCTAssertFalse(element(app, "turn.1.text").waitForExistence(timeout: 1), "a closed way sends nothing")

        element(app, "compass.north").tap()
        waitForPlace(app, "North of House")
        element(app, "compass.east").tap()
        waitForPlace(app, "Behind House")
        // The way west is through a window, which the story's game pack lets be
        // read as a door (feature 006). Before packs this was "may be blocked".
        XCTAssertEqual(value(app, "compass.west"), "door")
        XCTAssertEqual(element(app, "compass.west").label, "West, closed door")
        XCTAssertEqual(value(app, "compass.east"), "open")
    }

    // Feature 007 (specs/007-inventory-panel/spec.md User Story 3; FR-020 to
    // FR-024; SC-006): what the player carries is on the screen without being
    // asked for. A closed container shows nothing in it; opened, it shows what
    // it holds. Touching the strip sends nothing.
    @MainActor
    func testTheInventoryStrip() {
        let app = launch()
        _ = turn(app, 0)
        XCTAssertEqual(element(app, "inventory.note").label, "Carrying nothing")
        XCTAssertFalse(element(app, "inventory.strip").exists)

        type(app, "open mailbox")
        _ = turn(app, 1)
        // The keyboard is up: the strip gives way with the compass.
        XCTAssertFalse(element(app, "inventory.note").isHittable)
        app.typeText("take leaflet")
        element(app, "button.send").tap()
        _ = turn(app, 2)
        element(app, "button.compass").tap()
        let strip = element(app, "inventory.strip")
        XCTAssertTrue(strip.waitForExistence(timeout: patience))
        XCTAssertEqual(strip.label, "Carrying: leaflet")
        XCTAssertFalse(element(app, "inventory.note").exists)

        element(app, "compass.north").tap()
        waitForPlace(app, "North of House")
        element(app, "compass.east").tap()
        waitForPlace(app, "Behind House")
        type(app, "open window")
        _ = turn(app, 5)
        element(app, "button.compass").tap()
        element(app, "compass.west").tap()
        waitForPlace(app, "Kitchen")
        type(app, "take sack")
        _ = turn(app, 7)
        element(app, "button.compass").tap()
        XCTAssertEqual(strip.label, "Carrying: brown sack, closed; leaflet")

        // Touching the strip sends nothing.
        strip.tap()
        XCTAssertFalse(element(app, "turn.8.text").waitForExistence(timeout: 1))

        type(app, "open sack")
        _ = turn(app, 8)
        element(app, "button.compass").tap()
        XCTAssertEqual(
            strip.label,
            "Carrying: brown sack, open, holding clove of garlic and lunch; leaflet")
    }

    // Feature 008, User Story 4 (FR-018 to FR-020; SC-007): a way into darkness
    // is told before it is walked; in the dark the layer's note stands above
    // the compass, which is the room's own; with a light, neither is shown.
    @MainActor
    func testDarknessIsWarnedOf() {
        let app = launch()
        _ = turn(app, 0)
        XCTAssertFalse(element(app, "light.note").exists)
        element(app, "compass.north").tap()
        waitForPlace(app, "North of House")
        element(app, "compass.east").tap()
        waitForPlace(app, "Behind House")
        type(app, "open window")
        _ = turn(app, 3)
        element(app, "button.compass").tap()
        element(app, "compass.west").tap()
        waitForPlace(app, "Kitchen")
        let up = element(app, "compass.up")
        XCTAssertTrue(up.label.hasSuffix(", into darkness"), up.label)
        XCTAssertFalse(element(app, "compass.west").label.contains("darkness"))
        XCTAssertFalse(element(app, "light.note").exists)

        up.tap()
        waitForPlace(app, "Attic")
        let note = element(app, "light.note")
        XCTAssertTrue(note.waitForExistence(timeout: patience))
        XCTAssertEqual(note.label, "It is dark")
        // The compass is the room's own: the way back is offered.
        XCTAssertEqual(value(app, "compass.down"), "open")
        XCTAssertTrue(element(app, "compass.down").isEnabled)

        element(app, "compass.down").tap()
        waitForPlace(app, "Kitchen")
        XCTAssertFalse(note.exists)
        element(app, "compass.west").tap()
        waitForPlace(app, "Living Room")
        type(app, "take lamp")
        _ = turn(app, 8)
        app.typeText("turn on lamp")
        element(app, "button.send").tap()
        _ = turn(app, 9)
        element(app, "button.compass").tap()
        // The compass comes back as the keyboard goes: wait until it can be
        // touched.
        let east = element(app, "compass.east")
        expectation(for: NSPredicate(format: "isHittable == true"), evaluatedWith: east)
        waitForExpectations(timeout: patience)
        east.tap()
        waitForPlace(app, "Kitchen")
        XCTAssertFalse(up.label.contains("darkness"), up.label)
        up.tap()
        waitForPlace(app, "Attic")
        XCTAssertEqual(value(app, "compass.down"), "open")
        XCTAssertFalse(note.exists)
    }

    // Feature 009 (SC-001, SC-006, SC-007; contracts/lookahead.md statement
    // 24): the way down from the living room is no way until the trap door is
    // open; then it is an open way, said to lead to the cellar and into
    // darkness; and with the lamp lit, not into darkness. No code of the app
    // was written for this: it draws what the layer reports.
    @MainActor
    func testTheTrapDoorComesAndGoes() {
        let app = launch()
        _ = turn(app, 0)
        element(app, "compass.north").tap()
        waitForPlace(app, "North of House")
        element(app, "compass.east").tap()
        waitForPlace(app, "Behind House")
        type(app, "open window")
        _ = turn(app, 3)
        element(app, "button.compass").tap()
        element(app, "compass.west").tap()
        waitForPlace(app, "Kitchen")
        element(app, "compass.west").tap()
        waitForPlace(app, "Living Room")
        let down = element(app, "compass.down")
        XCTAssertEqual(value(app, "compass.down"), "closed")
        XCTAssertFalse(down.isEnabled, "a way the game refuses cannot be touched")

        type(app, "move rug")
        _ = turn(app, 6)
        app.typeText("open trap door")
        element(app, "button.send").tap()
        _ = turn(app, 7)
        element(app, "button.compass").tap()
        // The compass comes back as the keyboard goes: wait until it can be
        // touched.
        expectation(for: NSPredicate(format: "isHittable == true"), evaluatedWith: down)
        waitForExpectations(timeout: patience)
        XCTAssertEqual(value(app, "compass.down"), "open")
        XCTAssertTrue(down.isEnabled)
        XCTAssertEqual(down.label, "Down, to Cellar, into darkness")

        type(app, "take lamp")
        _ = turn(app, 8)
        app.typeText("turn on lamp")
        element(app, "button.send").tap()
        _ = turn(app, 9)
        element(app, "button.compass").tap()
        expectation(for: NSPredicate(format: "isHittable == true"), evaluatedWith: down)
        waitForExpectations(timeout: patience)
        XCTAssertEqual(value(app, "compass.down"), "open")
        XCTAssertEqual(down.label, "Down, to Cellar")
    }

    // Feature 008, FR-021; US4 scenario 5: where the layer does not know of
    // light, nothing is shown; and where it says dark, its words are shown as
    // they are. Drawn from hand-made snapshots.
    @MainActor
    func testWhereLightIsNotKnownNothingIsShown() {
        var app = launch(drawing: handMade(
            state: "open", spokenTail: "to A Place", known: true, note: nil))
        _ = turn(app, 0)
        XCTAssertFalse(element(app, "light.note").exists)
        XCTAssertEqual(element(app, "compass.north").label, "North, to A Place")
        app.terminate()

        app = launch(drawing: handMade(
            state: "open", spokenTail: "to A Place", known: true, note: nil,
            light: #"{"dark":true,"note":"The layer's own words"}"#, darkAhead: ["north"]))
        _ = turn(app, 0)
        XCTAssertEqual(element(app, "light.note").label, "The layer's own words")
        app.terminate()
    }

    // Feature 007, User Story 4 (FR-022, FR-025): where what is carried is not
    // known the strip says so and shows no item; and it is drawn, and read, at
    // times the game is not waiting for a command. Drawn from hand-made
    // snapshots: the bundled game has no such moment that is quick to reach.
    @MainActor
    func testAStripThatIsNotKnown() {
        let unknown = #"{"known":false,"note":"Not known: type to ask","spoken":"Not known: type to ask","items":[]}"#
        var app = launch(drawing: handMade(
            state: "open", spokenTail: "to somewhere", known: true, note: nil,
            inventory: unknown))
        XCTAssertTrue(element(app, "inventory.note").waitForExistence(timeout: patience))
        XCTAssertEqual(element(app, "inventory.note").label, "Not known: type to ask")
        XCTAssertFalse(element(app, "inventory.strip").exists)
        app.terminate()

        // A closed container and a plain item, with the game not waiting for a
        // command: still there, and still read.
        let carrying = #"{"known":true,"note":null,"spoken":"Carrying: box, closed; key","items":[{"name":"box","state":"closed","contents":[]},{"name":"key","state":"plain","contents":[]}]}"#
        app = launch(drawing: handMade(
            state: "idle", spokenTail: "not available now", offered: false, known: false,
            note: nil, waiting: "ended", inventory: carrying))
        let strip = element(app, "inventory.strip")
        XCTAssertTrue(strip.waitForExistence(timeout: patience))
        XCTAssertEqual(strip.label, "Carrying: box, closed; key")
    }

    // Feature 006, User Story 5 (specs/006-game-packs/spec.md FR-030, FR-031;
    // SC-010): a closed door is shown as closed and can be tried; opened by a
    // typed command it is an open way; and a way the game decides in its own
    // code still may be blocked.
    @MainActor
    func testADoorClosedThenOpen() {
        let app = launch()
        _ = turn(app, 0)
        element(app, "compass.north").tap()
        waitForPlace(app, "North of House")
        element(app, "compass.east").tap()
        waitForPlace(app, "Behind House")
        for way in ["west", "in"] {
            XCTAssertEqual(value(app, "compass.\(way)"), "door", way)
            XCTAssertTrue(element(app, "compass.\(way)").label.hasSuffix(", closed door"), way)
            XCTAssertTrue(element(app, "compass.\(way)").isEnabled, "a closed door can be tried")
        }
        // Trying it types the direction; the game refuses, and nothing else is done.
        element(app, "compass.west").tap()
        _ = turn(app, 3)
        XCTAssertEqual(element(app, "turn.3.command").label, "west")
        XCTAssertEqual(element(app, "status.place").label, "Behind House")
        XCTAssertEqual(value(app, "compass.west"), "door")

        type(app, "open window")
        _ = turn(app, 4)
        // The keyboard is up after typing, and the compass away: bring it back.
        element(app, "button.compass").tap()
        XCTAssertEqual(value(app, "compass.west"), "open")
        XCTAssertEqual(element(app, "compass.west").label, "West, to Kitchen")
        element(app, "compass.west").tap()
        waitForPlace(app, "Kitchen")
        XCTAssertEqual(element(app, "compass.east").label, "East, to Behind House")

        // The next room has a way down that the game decides in its own code.
        element(app, "compass.west").tap()
        waitForPlace(app, "Living Room")
        // Look-ahead finds the game refuses it while the trap door is hidden:
        // it is no way at all (feature 009).
        XCTAssertEqual(value(app, "compass.down"), "closed")
        XCTAssertEqual(element(app, "compass.down").label, "Down, no exit")
    }

    // S§ "What the app sends", "Waiting states"; FR-010, FR-011, FR-016;
    // US2 scenarios 2, 3, 4.
    @MainActor
    func testATapMovesThePlayer() {
        let app = launch()
        _ = turn(app, 0)
        element(app, "compass.north").tap()
        XCTAssertTrue(turn(app, 1).hasPrefix("North of House"))
        XCTAssertEqual(element(app, "turn.1.command").label, "north")
        XCTAssertEqual(element(app, "status.place").label, "North of House")
        XCTAssertEqual(value(app, "status.moves"), "1")
        XCTAssertEqual(value(app, "compass.south"), "closed", "the compass is this room's")
        XCTAssertEqual(element(app, "compass.north").label, "North, to Forest Path")
        element(app, "compass.north").tap()
        XCTAssertTrue(turn(app, 2).hasPrefix("Forest Path"), "where the label said it led")
    }

    // FR-016: two taps as fast as they can be made never send a command the
    // compass on screen did not offer. From the opening, north then north is
    // offered both times, so one or two turns follow, and never a third.
    @MainActor
    func testFastTapsSendOnlyWhatWasOffered() {
        let app = launch()
        _ = turn(app, 0)
        let north = element(app, "compass.north")
        north.tap()
        north.tap()
        _ = turn(app, 1)
        XCTAssertFalse(element(app, "turn.3.text").waitForExistence(timeout: 1))
        let moves = Int(value(app, "status.moves")) ?? -1
        XCTAssertTrue(moves == 1 || moves == 2, "moves: \(moves)")
    }

    // US2 scenario 4: ten taps on whatever is open reach at least five rooms.
    @MainActor
    func testTenTapsReachFiveRooms() {
        let app = launch()
        _ = turn(app, 0)
        var rooms: Set<String> = [element(app, "status.place").label]
        var cameBy = ""
        let opposite = [
            "north": "south", "south": "north", "east": "west", "west": "east",
            "northeast": "southwest", "southwest": "northeast",
            "northwest": "southeast", "southeast": "northwest",
        ]
        for tap in 1...10 {
            // Any open way, but not straight back if there is another.
            let open = places.filter { value(app, "compass.\($0)") == "open" }
            XCTAssertFalse(open.isEmpty, "something is open at tap \(tap)")
            let choice = open.first { $0 != opposite[cameBy] } ?? open[0]
            element(app, "compass.\(choice)").tap()
            _ = turn(app, tap)
            cameBy = choice
            rooms.insert(element(app, "status.place").label)
        }
        XCTAssertGreaterThanOrEqual(rooms.count, 5, "\(rooms)")
    }

    // S§ "Layout" keyboard rule, "Waiting states"; FR-017, FR-018, FR-019;
    // US3 scenarios 1, 2, 3.
    @MainActor
    func testTypingACommand() {
        let app = launch()
        _ = turn(app, 0)
        let send = element(app, "button.send")
        XCTAssertFalse(send.isEnabled, "nothing to send yet")
        XCTAssertEqual(element(app, "field.command").placeholderValue, "Type a command")
        type(app, "open mailbox")
        XCTAssertTrue(turn(app, 1).contains("Opening the small mailbox reveals a leaflet."))
        XCTAssertEqual(element(app, "turn.1.command").label, "open mailbox")
        XCTAssertFalse(send.isEnabled, "the field was cleared")

        // The keyboard is still up: the compass is away, the newest text and
        // the field are in reach.
        XCTAssertFalse(element(app, "compass.north").isHittable)
        XCTAssertTrue(element(app, "turn.1.text").isHittable)
        XCTAssertTrue(element(app, "field.command").isHittable)
        // Typing again needs no tap on the field.
        app.typeText("north")
        send.tap()
        XCTAssertTrue(turn(app, 2).hasPrefix("North of House"))
        XCTAssertEqual(element(app, "status.place").label, "North of House")

        element(app, "button.compass").tap()
        XCTAssertTrue(element(app, "compass.north").waitForExistence(timeout: patience))
        XCTAssertTrue(element(app, "compass.north").isHittable)
        XCTAssertEqual(element(app, "compass.north").label, "North, to Forest Path")
    }

    // S§ "Waiting states"; FR-015; US3 scenario 4.
    @MainActor
    func testAQuestionDarkensTheCompass() {
        let app = launch()
        _ = turn(app, 0)
        type(app, "quit")
        _ = turn(app, 1)
        XCTAssertEqual(element(app, "field.command").placeholderValue, "Type your answer")
        element(app, "button.compass").tap()
        for place in places {
            XCTAssertEqual(value(app, "compass.\(place)"), "idle", place)
            XCTAssertFalse(element(app, "compass.\(place)").isEnabled, place)
        }
        type(app, "n")
        _ = turn(app, 2)
        element(app, "button.compass").tap()
        XCTAssertEqual(value(app, "compass.north"), "open")
        XCTAssertEqual(element(app, "field.command").placeholderValue, "Type a command")
    }

    // S§ "Waiting states"; FR-022; SC-008; US4 scenario 2.
    @MainActor
    func testTheEndAndANewGameTenTimes() {
        let app = launch()
        for round in 1...10 {
            _ = turn(app, 0)
            type(app, "quit")
            _ = turn(app, 1)
            type(app, "y")
            let again = element(app, "button.newGame")
            XCTAssertTrue(again.waitForExistence(timeout: patience), "round \(round)")
            XCTAssertTrue(app.staticTexts["The game has ended."].exists)
            XCTAssertFalse(element(app, "field.command").exists)
            for place in ["north", "south", "west"] {
                XCTAssertFalse(element(app, "compass.\(place)").isEnabled, place)
            }
            again.tap()
            _ = turn(app, 0)
            XCTAssertFalse(element(app, "turn.1.text").exists, "only the opening, round \(round)")
            XCTAssertEqual(value(app, "compass.north"), "open")
        }
    }

    // S§ "Starting"; FR-024; SC-009; US4 scenario 5.
    @MainActor
    func testAStoryThatIsMissing() {
        let app = launch(["-story", "nosuchfile"])
        XCTAssertTrue(app.staticTexts["The game could not be started"].waitForExistence(timeout: patience))
        XCTAssertFalse(element(app, "compass.north").exists)
        XCTAssertFalse(element(app, "field.command").exists)
        XCTAssertEqual(app.state, .runningForeground)
    }

    // FR-023; US4 scenario 3: the game's own save, restore and script commands
    // are answered and the game goes on.
    @MainActor
    func testSaveRestoreAndScriptGoOn() {
        let app = launch()
        _ = turn(app, 0)
        for (number, line) in ["save", "restore", "script"].enumerated() {
            type(app, line)
            XCTAssertTrue(turn(app, number + 1).hasSuffix(">"), line)
            XCTAssertEqual(element(app, "turn.\(number + 1).command").label, line)
        }
        type(app, "look")
        XCTAssertTrue(turn(app, 4).hasPrefix("West of House"))
    }

    // FR-006; US4 scenario 4: leaving and returning changes nothing.
    @MainActor
    func testLeavingAndReturning() {
        let app = launch()
        _ = turn(app, 0)
        element(app, "compass.north").tap()
        let before = turn(app, 1)
        XCUIDevice.shared.press(.home)
        app.activate()
        XCTAssertEqual(turn(app, 1), before)
        XCTAssertFalse(element(app, "turn.2.text").exists)
        XCTAssertEqual(element(app, "status.place").label, "North of House")
        XCTAssertEqual(element(app, "compass.north").label, "North, to Forest Path")
    }

    // S§ "About"; FR-029.
    @MainActor
    func testAbout() {
        let app = launch()
        _ = turn(app, 0)
        element(app, "button.about").tap()
        XCTAssertTrue(app.staticTexts["Lamplight"].waitForExistence(timeout: patience))
        XCTAssertTrue(element(app, "about.demonstration").exists)
        let license = element(app, "about.license")
        XCTAssertTrue(license.exists)
        XCTAssertTrue(license.label.contains("MIT License"), license.label)
        XCTAssertTrue(license.label.contains("Permission is hereby granted"))
        // The notice's own line ends are for a wide page; on a phone each
        // paragraph is one run of text, its words unchanged.
        XCTAssertTrue(
            license.label.contains("to any person obtaining a copy of this software"),
            license.label)
        XCTAssertTrue(license.label.contains(")\n\nMIT License\n\nCopyright"), license.label)
    }

    // Found by playing by hand: a transcript shorter than its room sat at the
    // bottom at launch and at the top after "New game". It sits at the bottom,
    // next to the compass, both times.
    @MainActor
    func testAShortTranscriptSitsAtTheBottom() {
        let app = launch()
        _ = turn(app, 0)
        func gap() -> CGFloat {
            element(app, "transcript").frame.maxY - element(app, "turn.0.text").frame.maxY
        }
        XCTAssertLessThan(abs(gap()), 30, "at launch")
        type(app, "quit")
        _ = turn(app, 1)
        type(app, "y")
        let again = element(app, "button.newGame")
        XCTAssertTrue(again.waitForExistence(timeout: patience))
        again.tap()
        XCTAssertTrue(element(app, "compass.north").waitForExistence(timeout: patience))
        XCTAssertFalse(element(app, "turn.1.text").waitForExistence(timeout: 1))
        XCTAssertLessThan(abs(gap()), 30, "after a new game")
    }

    /// A hand-made snapshot: a compass with every place in one state, given
    /// extras and a note. Debug builds draw it in place of a game when it is
    /// passed in the environment (S§ "For automated tests").
    private func handMade(
        state: String, spokenTail: String, offered: Bool = true, known: Bool,
        note: String?, extras: String = "", waiting: String = "command",
        inventory: String = #"{"known":true,"note":"Carrying nothing","spoken":"Carrying nothing","items":[]}"#,
        light: String = #"{"dark":null,"note":null}"#, darkAhead: [String] = []
    ) -> String {
        let labels = [
            "North", "Northeast", "East", "Southeast", "South", "Southwest", "West", "Northwest",
            "Up", "Down", "In", "Out",
        ]
        let drawn = zip(places, labels).map { place, label in
            """
            {"place":"\(place)","label":"\(label)","state":"\(state)","command":null,\
            "leads_to":null,"dark_ahead":\(darkAhead.contains(place)),"spoken":"\(label), \(spokenTail)"}
            """
        }.joined(separator: ",")
        let noteJSON = note.map { "\"\($0)\"" } ?? "null"
        return """
            {"generation":1,"turn_count":1,"waiting":"\(waiting)","message":null,\
            "status":{"place":"A Place","score":null,"moves":null,"hours":9,"minutes":5},\
            "turns":[{"number":0,"command":null,"source":null,"text":"Ready.\\n\\n>"}],\
            "compass":{"offered":\(offered),"known":\(known),"note":\(noteJSON),\
            "places":[\(drawn)],"extras":[\(extras)]},"inventory":\(inventory),"light":\(light)}
            """
    }

    @MainActor
    private func launch(drawing snapshot: String) -> XCUIApplication {
        let app = XCUIApplication()
        app.launchEnvironment["LAMPLIGHT_FIXTURE_SNAPSHOT"] = snapshot
        app.launch()
        return app
    }

    // FR-014; US4 scenario 1. No room on the bundled game's walkthrough has
    // unknown exits, so the compass is drawn from a hand-made snapshot.
    @MainActor
    func testUnknownExitsAreSaidToBeUnknown() {
        var app = launch(drawing: handMade(
            state: "unknown", spokenTail: "unknown", known: false,
            note: "Exits unknown: type a direction"))
        _ = turn(app, 0)
        XCTAssertEqual(element(app, "compass.note").label, "Exits unknown: type a direction")
        for place in places {
            XCTAssertEqual(value(app, "compass.\(place)"), "unknown", place)
            XCTAssertFalse(element(app, "compass.\(place)").isEnabled, place)
        }
        XCTAssertEqual(element(app, "compass.north").label, "North, unknown")
        XCTAssertTrue(element(app, "field.command").isEnabled, "typing is still offered")
        XCTAssertEqual(element(app, "status.time").label, "9:05")
        XCTAssertFalse(element(app, "status.score").exists)
        app.terminate()

        // Known, with nothing open, says something else.
        app = launch(drawing: handMade(
            state: "closed", spokenTail: "no exit", known: true, note: "No exits here"))
        _ = turn(app, 0)
        XCTAssertEqual(element(app, "compass.note").label, "No exits here")
        XCTAssertEqual(value(app, "compass.north"), "closed")
    }

    // FR-013; US2 scenario 7. The bundled game's first extra way out is over a
    // hundred commands in, so the row is drawn from a hand-made snapshot; that
    // a tap on an extra reaches the game is tested in the layer and bindings.
    @MainActor
    func testExtraWaysOutAreARowOfButtons() {
        let extras = """
            {"label":"Land","state":"open","command":"land","leads_to":"Sandy Beach","dark_ahead":false,\
            "spoken":"Land, to Sandy Beach"},\
            {"label":"Cross","state":"maybe","command":"cross","leads_to":null,"dark_ahead":false,\
            "spoken":"Cross, may be blocked"},\
            {"label":"Launch","state":"closed","command":null,"leads_to":null,"dark_ahead":false,\
            "spoken":"Launch, no exit"}
            """
        let app = launch(drawing: handMade(
            state: "closed", spokenTail: "no exit", known: true, note: nil, extras: extras))
        _ = turn(app, 0)
        let land = element(app, "compass.extra.0")
        XCTAssertTrue(land.exists)
        XCTAssertEqual(land.label, "Land, to Sandy Beach")
        XCTAssertEqual(land.value as? String, "open")
        XCTAssertTrue(land.isEnabled && land.isHittable)
        XCTAssertEqual(value(app, "compass.extra.1"), "maybe")
        XCTAssertTrue(element(app, "compass.extra.1").isEnabled)
        XCTAssertFalse(element(app, "compass.extra.2").isEnabled)
        XCTAssertFalse(element(app, "compass.extra.3").exists)
        XCTAssertFalse(element(app, "compass.note").exists)
    }

    // S§ "With the keyboard showing": sending does not put the keyboard away,
    // and nothing typed after Return is lost. Found by playing by hand: Return
    // took the focus from the field for a moment, and "examine window" typed
    // straight after it arrived as "e window".
    @MainActor
    func testTypingStraightThroughReturn() {
        let app = launch()
        _ = turn(app, 0)
        let field = element(app, "field.command")
        field.tap()
        field.typeText("open mailbox\nread leaflet\nnorth\n")
        XCTAssertTrue(turn(app, 3).hasPrefix("North of House"))
        XCTAssertEqual(element(app, "turn.1.command").label, "open mailbox")
        XCTAssertEqual(element(app, "turn.2.command").label, "read leaflet")
        XCTAssertEqual(element(app, "turn.3.command").label, "north")
        XCTAssertEqual(value(app, "status.moves"), "3")
        XCTAssertFalse(element(app, "button.send").isEnabled, "the field is empty")
    }

    // SC-001; S§ launch arguments. The limit is the specification's two seconds.
    // TEST_RUNNER_LAUNCH_LIMIT_MILLIS gives another, for a machine that is not
    // the measure of the app: a shared CI runner took 2122 ms.
    @MainActor
    func testLaunchTime() {
        let given = ProcessInfo.processInfo.environment["LAUNCH_LIMIT_MILLIS"]
        let limit = given.flatMap { Int($0) } ?? 2000
        let app = launch(["-seed", "11", "-showLaunchTime"])
        let shown = element(app, "debug.launchMillis")
        XCTAssertTrue(shown.waitForExistence(timeout: patience))
        let millis = Int(shown.label) ?? -1
        XCTAssertGreaterThan(millis, 0, shown.label)
        XCTAssertLessThan(millis, limit, shown.label)
    }

    // SC-007: after thirty commands every turn can still be reached.
    @MainActor
    func testALongSession() {
        let app = launch()
        _ = turn(app, 0)
        let lines = [
            "open mailbox", "read leaflet", "inventory", "look", "north", "look", "north", "look",
            "south", "south", "examine house", "west", "look", "take mailbox", "open door",
            "south", "look", "east", "look", "open window", "in", "look", "take all", "west",
            "look", "east", "inventory", "diagnose", "score", "look",
        ]
        XCTAssertEqual(lines.count, 30)
        for (index, line) in lines.enumerated() {
            type(app, line)
            _ = turn(app, index + 1)
        }
        XCTAssertTrue(element(app, "turn.30.text").isHittable, "the newest turn is in view")
        let transcript = element(app, "transcript")
        let first = element(app, "turn.0.text")
        var swipes = 0
        while !first.isHittable && swipes < 40 {
            transcript.swipeDown(velocity: .fast)
            swipes += 1
        }
        XCTAssertTrue(first.isHittable, "the opening can be reached")
        for number in 0...30 {
            XCTAssertTrue(element(app, "turn.\(number).text").exists, "turn \(number)")
        }
        // The next turn brings the newest text back into view.
        type(app, "look")
        _ = turn(app, 31)
        XCTAssertTrue(element(app, "turn.31.text").isHittable)
    }

    // SC-010; FR-030; S§ "At large text sizes".
    @MainActor
    func testTheLargestTextSize() {
        let app = launch([
            "-seed", "11",
            "-UIPreferredContentSizeCategoryName", "UICTContentSizeCategoryAccessibilityXXXL",
        ])
        _ = turn(app, 0)
        var frames: [(String, CGRect)] = []
        for place in places {
            let control = element(app, "compass.\(place)")
            XCTAssertTrue(control.isHittable, place)
            XCTAssertGreaterThanOrEqual(control.frame.width, 44, place)
            XCTAssertGreaterThanOrEqual(control.frame.height, 44, place)
            frames.append((place, control.frame))
        }
        // A figure in the status line is on one line: wider than it is tall. (A
        // first version of the bar passed every other check here while wrapping
        // "Score" a letter to a line.)
        for identifier in ["status.score", "status.moves"] {
            let frame = element(app, identifier).frame
            XCTAssertGreaterThan(frame.width, frame.height, "\(identifier) \(frame)")
        }
        XCTAssertLessThan(
            element(app, "status.place").frame.height, 200, "the place is not squeezed into a column")
        for identifier in [
            "field.command", "button.send", "button.about", "status.place", "status.score",
            "status.moves",
        ] {
            XCTAssertTrue(element(app, identifier).isHittable, identifier)
            frames.append((identifier, element(app, identifier).frame))
        }
        let screen = app.windows.firstMatch.frame
        for (index, (name, frame)) in frames.enumerated() {
            XCTAssertTrue(screen.contains(frame), "\(name) is on the screen: \(frame)")
            for (other, otherFrame) in frames[(index + 1)...] {
                XCTAssertFalse(
                    frame.insetBy(dx: 0.5, dy: 0.5).intersects(otherFrame),
                    "\(name) \(frame) overlaps \(other) \(otherFrame)")
            }
        }
        XCTAssertEqual(element(app, "compass.north").label, "North, to North of House")
    }

    /// Not a test: saves pictures of the states whose look no test can judge.
    /// Runs only when TEST_RUNNER_SHOTS names a folder to save them in, with
    /// TEST_RUNNER_SHOTS_SUFFIX ("light" or "dark") in each file's name.
    @MainActor
    func testScreenshots() throws {
        let environment = ProcessInfo.processInfo.environment
        guard let folder = environment["SHOTS"] else {
            throw XCTSkip("no folder for screenshots was given")
        }
        let suffix = environment["SHOTS_SUFFIX"] ?? "light"
        func save(_ name: String) throws {
            let url = URL(fileURLWithPath: folder).appendingPathComponent("\(name)-\(suffix).png")
            try XCUIScreen.main.screenshot().pngRepresentation.write(to: url)
        }
        let app = launch()
        _ = turn(app, 0)
        try save("opening")
        element(app, "compass.north").tap()
        waitForPlace(app, "North of House")
        element(app, "compass.east").tap()
        waitForPlace(app, "Behind House")
        try save("behind-house")
        type(app, "open window")
        _ = turn(app, 3)
        try save("keyboard")
        type(app, "quit")
        _ = turn(app, 4)
        element(app, "button.compass").tap()
        try save("question")
        type(app, "y")
        XCTAssertTrue(element(app, "button.newGame").waitForExistence(timeout: patience))
        try save("ended")
        app.terminate()
        let unknown = launch(drawing: handMade(
            state: "unknown", spokenTail: "unknown", known: false,
            note: "Exits unknown: type a direction"))
        _ = turn(unknown, 0)
        try save("unknown-exits")
        unknown.terminate()

        // Feature 008: a way into darkness, the dark place, and both with a
        // light.
        let dark = launch()
        _ = turn(dark, 0)
        element(dark, "compass.north").tap()
        waitForPlace(dark, "North of House")
        element(dark, "compass.east").tap()
        waitForPlace(dark, "Behind House")
        type(dark, "open window")
        _ = turn(dark, 3)
        element(dark, "button.compass").tap()
        element(dark, "compass.west").tap()
        waitForPlace(dark, "Kitchen")
        try save("dark-ahead")
        element(dark, "compass.up").tap()
        waitForPlace(dark, "Attic")
        XCTAssertTrue(element(dark, "light.note").waitForExistence(timeout: patience))
        try save("in-the-dark")
        element(dark, "compass.down").tap()
        waitForPlace(dark, "Kitchen")
        element(dark, "compass.west").tap()
        waitForPlace(dark, "Living Room")
        type(dark, "take lamp")
        _ = turn(dark, 8)
        dark.typeText("turn on lamp")
        element(dark, "button.send").tap()
        _ = turn(dark, 9)
        element(dark, "button.compass").tap()
        let east = element(dark, "compass.east")
        expectation(for: NSPredicate(format: "isHittable == true"), evaluatedWith: east)
        waitForExpectations(timeout: patience)
        east.tap()
        waitForPlace(dark, "Kitchen")
        try save("lamp-on-ahead")
        element(dark, "compass.up").tap()
        waitForPlace(dark, "Attic")
        try save("lamp-on-there")
    }
}
