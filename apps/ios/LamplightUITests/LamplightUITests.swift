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
        XCTAssertTrue(opening.waitForExistence(timeout: 5), "the opening text is shown")
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
        XCTAssertTrue(field.waitForExistence(timeout: 5))
        field.tap()
        field.typeText(line)
        element(app, "button.send").tap()
    }

    /// Waits for turn `number` to be on screen and returns its text.
    @MainActor
    private func turn(_ app: XCUIApplication, _ number: Int) -> String {
        let text = element(app, "turn.\(number).text")
        XCTAssertTrue(text.waitForExistence(timeout: 5), "turn \(number) is shown")
        return text.label
    }

    @MainActor
    private func value(_ app: XCUIApplication, _ identifier: String) -> String {
        let found = element(app, identifier)
        XCTAssertTrue(found.waitForExistence(timeout: 5), identifier)
        return found.value as? String ?? ""
    }

    /// Waits until the place in the status line is this.
    @MainActor
    private func waitForPlace(_ app: XCUIApplication, _ place: String) {
        let status = element(app, "status.place")
        let arrived = NSPredicate(format: "label == %@", place)
        expectation(for: arrived, evaluatedWith: status)
        waitForExpectations(timeout: 5)
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
        XCTAssertFalse(element(app, "turn.1.text").exists, "nothing of the last game is shown")
        XCTAssertEqual(element(app, "status.place").label, "West of House")
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
        XCTAssertEqual(value(app, "compass.west"), "maybe")
        XCTAssertEqual(element(app, "compass.west").label, "West, may be blocked")
        XCTAssertEqual(value(app, "compass.east"), "open")
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
        XCTAssertTrue(element(app, "compass.north").waitForExistence(timeout: 2))
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
            XCTAssertTrue(again.waitForExistence(timeout: 5), "round \(round)")
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
        XCTAssertTrue(app.staticTexts["The game could not be started"].waitForExistence(timeout: 5))
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
        XCTAssertTrue(app.staticTexts["Lamplight"].waitForExistence(timeout: 2))
        XCTAssertTrue(element(app, "about.demonstration").exists)
        let license = element(app, "about.license")
        XCTAssertTrue(license.exists)
        XCTAssertTrue(license.label.contains("MIT License"), license.label)
        XCTAssertTrue(license.label.contains("Permission is hereby granted"))
    }
}
