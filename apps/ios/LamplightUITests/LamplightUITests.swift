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
        // The app brings back the game it kept (feature 011). `-fresh` has it
        // give that up first, so that every test starts from a new game.
        app.launchArguments = arguments + ["-fresh"]
        app.launch()
        return app
    }

    /// Ends the app abruptly, as the system may, and opens it again on what
    /// it kept: without `-fresh`, with `adding` and without `removing`.
    @MainActor
    private func relaunch(
        _ app: XCUIApplication, adding: [String] = [], removing: [String] = []
    ) {
        app.terminate()
        app.launchArguments.removeAll { $0 == "-fresh" || removing.contains($0) }
        app.launchArguments += adding
        app.launch()
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

    /// The state of a way of the place the player is in, or nil where the map
    /// shows no such way.
    @MainActor
    private func way(_ app: XCUIApplication, _ place: String) -> String? {
        let control = element(app, "compass.\(place)")
        return control.exists ? control.value as? String : nil
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

    // -----------------------------------------------------------------
    // Feature 011: the game comes back where it was left
    // (specs/011-resume-game/contracts/kept-game.md, cited A§). It replaces
    // feature 004's FR-005, "each launch is a new game", and the test of it.
    // -----------------------------------------------------------------

    // A§1, A§2; FR-001 to FR-005; US1 scenarios 1 to 4.
    @MainActor
    func testTheGameComesBackWhereItWasLeft() {
        let app = launch()
        let opening = turn(app, 0)
        element(app, "compass.north").tap()
        let north = turn(app, 1)
        type(app, "open mailbox")
        let mailbox = turn(app, 2)

        relaunch(app)
        XCTAssertEqual(turn(app, 2), mailbox)
        XCTAssertEqual(turn(app, 1), north)
        XCTAssertEqual(turn(app, 0), opening)
        XCTAssertEqual(element(app, "status.place").label, "North of House")
        XCTAssertEqual(value(app, "status.moves"), "2")
        XCTAssertEqual(value(app, "compass.north"), "open")
        XCTAssertFalse(element(app, "notice").exists)
        // Nothing stands between opening the app and a command.
        type(app, "look")
        XCTAssertTrue(turn(app, 3).hasPrefix("North of House"))
        XCTAssertEqual(value(app, "status.moves"), "3")
    }

    // FR-010 to FR-012; US2 scenario 1: ended abruptly after each of several
    // commands, the app has every answered turn each time.
    @MainActor
    func testNothingAnsweredIsLostWhenTheAppIsEndedAbruptly() {
        let app = launch()
        var texts = [turn(app, 0)]
        for (number, way) in ["north", "west", "north", "west"].enumerated() {
            element(app, "compass.\(way)").tap()
            texts.append(turn(app, number + 1))
            relaunch(app)
            for (earlier, text) in texts.enumerated() {
                XCTAssertEqual(turn(app, earlier), text, "turn \(earlier) after \(number + 1)")
            }
            XCTAssertFalse(element(app, "turn.\(number + 2).text").exists)
        }
    }

    // Edge case; FR-016: an ended game comes back ended, and New game starts
    // without asking. The new game is then the one that comes back.
    @MainActor
    func testAnEndedGameComesBackEnded() {
        let app = launch()
        _ = turn(app, 0)
        type(app, "quit")
        _ = turn(app, 1)
        type(app, "y")
        let ending = turn(app, 2)
        XCTAssertTrue(element(app, "button.newGame").waitForExistence(timeout: patience))

        relaunch(app)
        XCTAssertEqual(turn(app, 2), ending)
        XCTAssertTrue(app.staticTexts["The game has ended."].exists)
        XCTAssertFalse(element(app, "field.command").exists)
        element(app, "button.newGame").tap()
        XCTAssertTrue(element(app, "field.command").waitForExistence(timeout: patience))
        XCTAssertFalse(element(app, "turn.1.text").exists)

        relaunch(app)
        _ = turn(app, 0)
        XCTAssertFalse(element(app, "turn.1.text").waitForExistence(timeout: 2))
        XCTAssertTrue(element(app, "field.command").exists)
    }

    // FR-027: on reopening, the newest reply is asked to be spoken, once.
    @MainActor
    func testTheNewestReplyIsSpokenOnceOnReopening() {
        let app = launch(["-seed", "11", "-speakAlways"])
        _ = turn(app, 0)
        element(app, "compass.north").tap()
        let north = turn(app, 1)
        XCTAssertEqual(asked(app, times: 2), spoken(north))

        relaunch(app)
        _ = turn(app, 1)
        XCTAssertEqual(asked(app, times: 1), spoken(north))
    }

    // A§4; FR-014, FR-015; US4 scenarios 1 to 3; SC-007: starting over asks
    // first, in the layer's words, and is not undone by opening the app again.
    @MainActor
    func testStartingOverAsksAndIsNotUndone() {
        let app = launch()
        _ = turn(app, 0)
        element(app, "compass.north").tap()
        let north = turn(app, 1)

        // The button can be touched once the sheet has finished rising.
        let ready = NSPredicate(format: "exists == true AND isHittable == true")
        element(app, "button.about").tap()
        let startOver = element(app, "button.startOver")
        expectation(for: ready, evaluatedWith: startOver)
        waitForExpectations(timeout: patience)
        startOver.tap()
        let question = app.staticTexts["Start over? The game in progress will be given up."]
        XCTAssertTrue(question.waitForExistence(timeout: patience))
        // Declined: by the dialog's Cancel where the system gives it one,
        // and otherwise by touching away from it.
        if app.buttons["Cancel"].exists {
            app.buttons["Cancel"].tap()
        } else {
            app.coordinate(withNormalizedOffset: CGVector(dx: 0.5, dy: 0.1)).tap()
        }
        let unasked = NSPredicate(format: "exists == false")
        expectation(for: unasked, evaluatedWith: question)
        waitForExpectations(timeout: patience)
        expectation(for: ready, evaluatedWith: startOver)
        waitForExpectations(timeout: patience)
        element(app, "button.aboutDone").tap()
        XCTAssertEqual(turn(app, 1), north, "declined: nothing has changed")

        element(app, "button.about").tap()
        expectation(for: ready, evaluatedWith: startOver)
        waitForExpectations(timeout: patience)
        startOver.tap()
        XCTAssertTrue(question.waitForExistence(timeout: patience))
        // The dialog's own button: the one of that name that is not the
        // sheet's, which the dialog covers.
        app.buttons.matching(
            NSPredicate(
                format: "label == %@ AND identifier != %@", "Start over", "button.startOver")
        ).firstMatch.tap()
        // The sheet goes and the new game is alone on the screen.
        let gone = NSPredicate(format: "exists == false")
        expectation(for: gone, evaluatedWith: element(app, "turn.1.text"))
        waitForExpectations(timeout: patience)
        _ = turn(app, 0)
        XCTAssertEqual(element(app, "status.place").label, "West of House")
        XCTAssertFalse(startOver.exists, "the sheet has closed")

        relaunch(app)
        _ = turn(app, 0)
        XCTAssertFalse(element(app, "turn.1.text").waitForExistence(timeout: 2))
    }

    // A§5, A§6; FR-018; US5 scenarios 1, 4 and 5: a kept game that cannot be
    // brought back gives a new game, and says so once.
    @MainActor
    func testADamagedKeptGameGivesANewGameAndSaysSoOnce() {
        let app = launch()
        _ = turn(app, 0)
        element(app, "compass.north").tap()
        _ = turn(app, 1)

        relaunch(app, adding: ["-damageKept"])
        _ = turn(app, 0)
        XCTAssertFalse(element(app, "turn.1.text").waitForExistence(timeout: 2))
        XCTAssertEqual(
            element(app, "notice").label,
            "Your earlier game could not be brought back. This is a new game.")
        XCTAssertEqual(element(app, "status.place").label, "West of House")

        // The new game was kept, and comes back like any other, unremarked.
        relaunch(app, removing: ["-damageKept"])
        _ = turn(app, 0)
        XCTAssertFalse(element(app, "notice").exists)
        XCTAssertTrue(element(app, "field.command").exists)
    }

    // FR-020 in the simulator: opening on a kept game is within the launch
    // limit. The long games are timed in the layer's own tests.
    @MainActor
    func testLaunchTimeIntoAKeptGame() {
        let given = ProcessInfo.processInfo.environment["LAUNCH_LIMIT_MILLIS"]
        let limit = given.flatMap { Int($0) } ?? 2000
        let app = launch(["-seed", "11", "-showLaunchTime"])
        _ = turn(app, 0)
        for (number, way) in ["north", "west", "north", "west", "north", "west"].enumerated() {
            element(app, "compass.\(way)").tap()
            _ = turn(app, number + 1)
        }
        relaunch(app)
        _ = turn(app, 6)
        let shown = element(app, "debug.launchMillis")
        XCTAssertTrue(shown.waitForExistence(timeout: patience))
        let millis = Int(shown.label) ?? -1
        XCTAssertGreaterThan(millis, 0, shown.label)
        XCTAssertLessThan(millis, limit, shown.label)
        print("launch into a kept game of seven turns: \(millis) ms")
    }

    // -----------------------------------------------------------------
    // Feature 012, stage 2: the map's band in place of the compass
    // (specs/012-map-first-slice/contracts/map.md, cited A§).
    // -----------------------------------------------------------------

    /// Whether the software keyboard, or a hardware one's stand-in, has the
    /// command field: the field holds the keyboard's focus.
    @MainActor
    private func fieldHasTheKeyboard(_ app: XCUIApplication) -> Bool {
        (element(app, "field.command").value(forKey: "hasKeyboardFocus") as? Bool) ?? false
    }

    /// Scrolls back as a player does to put the keyboard away: draws the
    /// text down, the finger going on into the keyboard, which goes with it.
    @MainActor
    private func scrollBack(_ app: XCUIApplication) {
        let transcript = element(app, "transcript")
        let from = transcript.coordinate(withNormalizedOffset: CGVector(dx: 0.5, dy: 0.3))
        let to = app.coordinate(withNormalizedOffset: CGVector(dx: 0.5, dy: 0.97))
        from.press(forDuration: 0.1, thenDragTo: to)
        let down = NSPredicate { _, _ in !self.fieldHasTheKeyboard(app) }
        expectation(for: down, evaluatedWith: nil)
        waitForExpectations(timeout: patience)
    }

    /// The places drawn on the map.
    @MainActor
    private func placesDrawn(_ app: XCUIApplication) -> Int {
        app.descendants(matching: .any)
            .matching(NSPredicate(format: "identifier BEGINSWITH %@", "map.place."))
            .count
    }

    // A§1, A§3, A§9; US1 scenarios 1 and 2: the band shows the place the
    // player is in, and each place they walk to, with the layer's summary.
    @MainActor
    func testTheMapShowsWhereIAm() {
        let app = launch()
        _ = turn(app, 0)
        let map = element(app, "map")
        XCTAssertTrue(map.waitForExistence(timeout: patience))
        XCTAssertEqual(map.label, "Map. You are in West of House.")
        XCTAssertEqual(placesDrawn(app), 1)
        XCTAssertEqual(element(app, "map.here").label, "West of House")
        element(app, "compass.north").tap()
        waitForPlace(app, "North of House")
        XCTAssertEqual(placesDrawn(app), 2)
        XCTAssertEqual(element(app, "map.here").label, "North of House")
        XCTAssertEqual(map.label, "Map. You are in North of House.")
        // A§11: there is no compass to bring back.
        XCTAssertFalse(element(app, "button.compass").exists)
    }

    // A§2; W§7; SC-001: to the Kitchen and the Attic by touching the map
    // alone. A way names where it leads only once it has been walked.
    @MainActor
    func testWalkingToTheKitchenByTheMap() {
        let app = launch()
        _ = turn(app, 0)
        XCTAssertEqual(element(app, "compass.north").label, "North")
        element(app, "compass.north").tap()
        waitForPlace(app, "North of House")
        XCTAssertEqual(element(app, "turn.1.command").label, "north")
        element(app, "compass.east").tap()
        waitForPlace(app, "Behind House")
        // A closed door can be tried, and the game refuses.
        XCTAssertEqual(value(app, "compass.west"), "door")
        type(app, "open window")
        _ = turn(app, 3)
        element(app, "compass.west").tap()
        waitForPlace(app, "Kitchen")
        XCTAssertEqual(element(app, "compass.east").label, "East")
        element(app, "compass.up").tap()
        waitForPlace(app, "Attic")
        // The Attic is dark and has never been seen: the map shows no way
        // out of it, and the way back is typed.
        XCTAssertFalse(element(app, "compass.down").exists)
        app.typeText("down\n")
        waitForPlace(app, "Kitchen")
        // Walked both ways now: each end names the other.
        // The Attic has no name yet: it has only been stood in dark.
        XCTAssertEqual(element(app, "compass.up").label, "Up, into darkness")
        element(app, "compass.east").tap()
        waitForPlace(app, "Behind House")
        XCTAssertEqual(element(app, "compass.west").label, "West, to Kitchen")
        XCTAssertEqual(placesDrawn(app), 5)
        XCTAssertTrue(fieldHasTheKeyboard(app), "the keyboard never went down")
    }

    // A§4, A§5, A§6; US3 scenarios 1, 2, 5: the keyboard is up from the
    // start and stays up across typed and touched commands, with the band,
    // what is carried and the field all in view.
    @MainActor
    func testTheKeyboardStaysUp() {
        let app = launch()
        _ = turn(app, 0)
        let field = element(app, "field.command")
        XCTAssertTrue(field.waitForExistence(timeout: patience))
        let up = NSPredicate { _, _ in self.fieldHasTheKeyboard(app) }
        expectation(for: up, evaluatedWith: nil)
        waitForExpectations(timeout: patience)
        let band = element(app, "map.frame").frame
        // Typed without touching the field first.
        app.typeText("open mailbox\n")
        _ = turn(app, 1)
        XCTAssertTrue(fieldHasTheKeyboard(app))
        element(app, "compass.north").tap()
        _ = turn(app, 2)
        XCTAssertTrue(fieldHasTheKeyboard(app))
        app.typeText("look\n")
        _ = turn(app, 3)
        element(app, "compass.east").tap()
        _ = turn(app, 4)
        element(app, "compass.south").tap()
        _ = turn(app, 5)
        XCTAssertTrue(fieldHasTheKeyboard(app))
        XCTAssertEqual(element(app, "map.frame").frame, band, "the band keeps its size and place")
        XCTAssertEqual(band.height, 200, accuracy: 1)
        for identifier in ["map", "inventory.note", "field.command", "turn.5.text"] {
            XCTAssertTrue(element(app, identifier).isHittable, identifier)
        }
    }

    // A§7; US3 scenario 3: a reply longer than its room is shown from its
    // first line, with a sign that there is more.
    @MainActor
    func testALongReplyStartsAtItsFirstLine() {
        let app = launch()
        let opening = element(app, "turn.0.text")
        XCTAssertTrue(opening.waitForExistence(timeout: patience))
        let transcript = element(app, "transcript").frame
        XCTAssertGreaterThan(opening.frame.height, transcript.height, "the opening is long")
        XCTAssertEqual(opening.frame.minY, transcript.minY, accuracy: 12, "its first line is at the top")
        XCTAssertTrue(element(app, "transcript.more").exists)
        // A short reply fits, and nothing more is promised.
        type(app, "open mailbox")
        let reply = element(app, "turn.1.text")
        XCTAssertTrue(reply.waitForExistence(timeout: patience))
        // The view moves to the new turn over a moment.
        expectation(for: NSPredicate(format: "isHittable == true"), evaluatedWith: reply)
        waitForExpectations(timeout: patience)
        XCTAssertLessThanOrEqual(reply.frame.maxY, element(app, "transcript").frame.maxY + 1)
        XCTAssertFalse(element(app, "transcript.more").exists)
        // A later reply that is long is also shown from its first line.
        type(app, "take leaflet")
        _ = turn(app, 2)
        type(app, "north")
        _ = turn(app, 3)
        type(app, "south")
        _ = turn(app, 4)
        type(app, "read leaflet")
        let leaflet = element(app, "turn.5.text")
        XCTAssertTrue(leaflet.waitForExistence(timeout: patience))
        if leaflet.frame.height > element(app, "transcript").frame.height {
            let top = element(app, "transcript").frame.minY
            let settled = NSPredicate { _, _ in abs(leaflet.frame.minY - top) < 14 }
            expectation(for: settled, evaluatedWith: nil)
            waitForExpectations(timeout: patience)
            XCTAssertTrue(element(app, "transcript.more").exists)
        }
    }

    // Found by the owner on their phone, 2026-11-14: with the keyboard down
    // the whole transcript is in view; when the keyboard came back it covered
    // the newest text, and touching the transcript to bring that text up
    // dropped the keyboard again. The newest text comes back into view with
    // the keyboard, and reading on through a reply does not lower it.
    @MainActor
    func testTheNewestTextIsInViewWhenTheKeyboardReturns() {
        let app = launch()
        _ = turn(app, 0)
        type(app, "open mailbox")
        _ = turn(app, 1)
        type(app, "read leaflet")
        let newest = element(app, "turn.2.text")
        XCTAssertTrue(newest.waitForExistence(timeout: patience))
        // Scroll back: the keyboard goes down.
        scrollBack(app)
        // With the keyboard gone the command bar is at the foot of the
        // screen, and the transcript has the room the keyboard had: the
        // screen keeps none for it. (Found by the owner: the bar stayed up.)
        let field = element(app, "field.command")
        let screen = app.windows.firstMatch.frame
        let lowered = NSPredicate { _, _ in field.frame.maxY > screen.maxY - 80 }
        expectation(for: lowered, evaluatedWith: nil)
        waitForExpectations(timeout: patience)
        XCTAssertGreaterThan(element(app, "transcript").frame.height, 300)
        // Touch the field: the keyboard returns, and the newest text is
        // within the transcript, above the field: the whole of it where it
        // fits, and otherwise from its first line.
        element(app, "field.command").tap()
        let transcript = element(app, "transcript")
        let shown = NSPredicate { _, _ in
            let (text, room) = (newest.frame, transcript.frame)
            let whole = text.maxY <= room.maxY + 1 && text.minY >= room.minY - 1
            let fromItsStart = text.height > room.height && abs(text.minY - room.minY) < 14
            return self.fieldHasTheKeyboard(app) && (whole || fromItsStart)
        }
        expectation(for: shown, evaluatedWith: nil)
        waitForExpectations(timeout: patience)
        XCTAssertLessThan(transcript.frame.maxY, element(app, "field.command").frame.minY + 1)
    }

    // The same fault's other half: reading on through a reply that is longer
    // than its room does not lower the keyboard. Only scrolling back does.
    @MainActor
    func testReadingOnKeepsTheKeyboard() {
        let app = launch()
        let opening = element(app, "turn.0.text")
        XCTAssertTrue(opening.waitForExistence(timeout: patience))
        let ready = NSPredicate { _, _ in self.fieldHasTheKeyboard(app) }
        expectation(for: ready, evaluatedWith: nil)
        waitForExpectations(timeout: patience)
        XCTAssertTrue(element(app, "transcript.more").exists, "the opening is longer than its room")
        // Drag the text up to read the rest of it.
        let transcript = element(app, "transcript")
        let from = transcript.coordinate(withNormalizedOffset: CGVector(dx: 0.5, dy: 0.8))
        let to = transcript.coordinate(withNormalizedOffset: CGVector(dx: 0.5, dy: 0.1))
        from.press(forDuration: 0.05, thenDragTo: to)
        let read = NSPredicate { _, _ in !self.element(app, "transcript.more").exists }
        expectation(for: read, evaluatedWith: nil)
        waitForExpectations(timeout: patience)
        XCTAssertTrue(fieldHasTheKeyboard(app), "reading on keeps the keyboard")
        // And back up to its first line: still the newest turn, still up.
        to.press(forDuration: 0.05, thenDragTo: from)
        XCTAssertTrue(fieldHasTheKeyboard(app), "back to the reply's start keeps it too")
    }

    // A§4; US3 scenario 4: scrolling back lowers the keyboard; touching the
    // field raises it.
    @MainActor
    func testScrollingBackLowersTheKeyboard() {
        let app = launch()
        _ = turn(app, 0)
        app.typeText("open mailbox\nread leaflet\nlook\n")
        _ = turn(app, 3)
        XCTAssertTrue(fieldHasTheKeyboard(app))
        scrollBack(app)
        element(app, "field.command").tap()
        XCTAssertTrue(fieldHasTheKeyboard(app))
    }

    // The ways at the opening and behind the house, as the map shows them
    // (feature 004's FR-007 to FR-012 for the compass; feature 012's A§2,
    // W§3, W§7). Only the ways there are are drawn, and none names where it
    // leads until it has been walked.
    @MainActor
    func testTheWaysAtTheOpening() {
        let app = launch()
        _ = turn(app, 0)
        for place in ["north", "south", "west"] {
            XCTAssertEqual(value(app, "compass.\(place)"), "open", place)
        }
        for place in ["east", "up", "down", "in", "out", "northwest", "southwest"] {
            XCTAssertNil(way(app, place), "\(place) is no way here")
        }
        XCTAssertEqual(element(app, "compass.north").label, "North")

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
        // The keyboard is up, and what is carried is still shown (feature 012).
        XCTAssertTrue(element(app, "inventory.note").isHittable)
        app.typeText("take leaflet")
        element(app, "button.send").tap()
        _ = turn(app, 2)
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
        element(app, "compass.west").tap()
        waitForPlace(app, "Kitchen")
        type(app, "take sack")
        _ = turn(app, 7)
        XCTAssertEqual(strip.label, "Carrying: brown sack, closed; leaflet")

        // Touching the strip sends nothing.
        strip.tap()
        XCTAssertFalse(element(app, "turn.8.text").waitForExistence(timeout: 1))

        type(app, "open sack")
        _ = turn(app, 8)
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
        // The Attic has only been stood in dark: the map shows no way out of
        // it (feature 012, W§12), and the way back is typed.
        XCTAssertNil(way(app, "down"))
        type(app, "down")
        waitForPlace(app, "Kitchen")
        XCTAssertFalse(note.exists)
        element(app, "compass.west").tap()
        waitForPlace(app, "Living Room")
        type(app, "take lamp")
        _ = turn(app, 8)
        app.typeText("turn on lamp")
        element(app, "button.send").tap()
        _ = turn(app, 9)
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

    // -----------------------------------------------------------------
    // Feature 010: what the app asks a screen reader to speak
    // (specs/010-screen-reader-announcements/contracts/announcements.md,
    // cited N§). A test cannot hear. With `-speakAlways` the app behaves as
    // if a screen reader were running and shows what it last asked for, and
    // how many times it has asked, in an element these tests read.
    // -----------------------------------------------------------------

    /// A turn's text as it is to be spoken: without the white space at its end
    /// and the prompt the game ends it with.
    private func spoken(_ text: String) -> String {
        var said = text.trimmingCharacters(in: .whitespacesAndNewlines)
        if said.hasSuffix(">") { said.removeLast() }
        return said.trimmingCharacters(in: .whitespacesAndNewlines)
    }

    /// Waits until the app has asked `times` times, and returns what it asked
    /// for last.
    @MainActor
    private func asked(_ app: XCUIApplication, times: Int) -> String {
        let last = element(app, "announcement.last")
        XCTAssertTrue(last.waitForExistence(timeout: patience))
        expectation(for: NSPredicate(format: "value == %@", String(times)), evaluatedWith: last)
        waitForExpectations(timeout: patience)
        return last.label
    }

    // N§16 to 20; FR-001, FR-002, FR-007, FR-009; SC-004: the opening and each
    // reply are asked for, whole and without the prompt or the command, and
    // asking moves nothing.
    @MainActor
    func testEachReplyIsAskedToBeSpoken() {
        let app = launch(["-seed", "11", "-speakAlways"])
        let opening = turn(app, 0)
        XCTAssertEqual(asked(app, times: 1), spoken(opening))
        XCTAssertFalse(asked(app, times: 1).hasSuffix(">"))

        // A way that is touched.
        element(app, "compass.north").tap()
        let north = turn(app, 1)
        XCTAssertEqual(asked(app, times: 2), spoken(north))
        XCTAssertTrue(asked(app, times: 2).hasPrefix("North of House"))

        // A command that is typed: the field keeps the keyboard afterwards.
        type(app, "look")
        let look = turn(app, 2)
        XCTAssertEqual(asked(app, times: 3), spoken(look))
        XCTAssertFalse(asked(app, times: 3).contains("look"), "the command is not spoken")
        app.typeText("inventory\n")
        let listing = turn(app, 3)
        XCTAssertEqual(asked(app, times: 4), spoken(listing))
    }

    // N§18, 20; FR-010; SC-005: with no screen reader running and no argument,
    // nothing is asked for, and the element the argument adds is not there.
    @MainActor
    func testNothingIsAskedWithNoScreenReader() {
        let app = launch()
        _ = turn(app, 0)
        element(app, "compass.north").tap()
        _ = turn(app, 1)
        XCTAssertFalse(element(app, "announcement.last").exists)
    }

    // N§3, 4, 16; FR-004; SC-002: the note of darkness is asked for after the
    // reply on the turn the dark falls, and not on the turn that leaves it.
    @MainActor
    func testDarknessIsAskedToBeSpoken() {
        let app = launch(["-seed", "11", "-speakAlways"])
        _ = turn(app, 0)
        element(app, "compass.north").tap()
        waitForPlace(app, "North of House")
        element(app, "compass.east").tap()
        waitForPlace(app, "Behind House")
        type(app, "open window")
        _ = turn(app, 3)
        element(app, "compass.west").tap()
        waitForPlace(app, "Kitchen")
        XCTAssertFalse(asked(app, times: 5).contains("It is dark"))
        element(app, "compass.up").tap()
        waitForPlace(app, "Attic")
        let attic = turn(app, 5)
        XCTAssertEqual(asked(app, times: 6), spoken(attic) + "\nIt is dark")
        type(app, "down")
        waitForPlace(app, "Kitchen")
        let kitchen = turn(app, 6)
        XCTAssertEqual(asked(app, times: 7), spoken(kitchen))
    }

    // N§16; FR-008; SC-003: three lines typed straight through Return are
    // asked for one after another, and the last asked for is the third. That
    // each request is marked to cut off the one before is in the app's source;
    // no test can see it.
    @MainActor
    func testTheNewestReplyIsTheOneAskedFor() {
        let app = launch(["-seed", "11", "-speakAlways"])
        _ = turn(app, 0)
        let field = element(app, "field.command")
        field.tap()
        field.typeText("open mailbox\nread leaflet\nnorth\n")
        let third = turn(app, 3)
        XCTAssertEqual(asked(app, times: 4), spoken(third))
        XCTAssertTrue(asked(app, times: 4).hasPrefix("North of House"))
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
        element(app, "compass.west").tap()
        waitForPlace(app, "Kitchen")
        element(app, "compass.west").tap()
        waitForPlace(app, "Living Room")
        let down = element(app, "compass.down")
        XCTAssertFalse(down.exists, "a way the game refuses is not drawn")

        type(app, "move rug")
        _ = turn(app, 6)
        app.typeText("open trap door")
        element(app, "button.send").tap()
        _ = turn(app, 7)
        expectation(for: NSPredicate(format: "isHittable == true"), evaluatedWith: down)
        waitForExpectations(timeout: patience)
        XCTAssertEqual(value(app, "compass.down"), "open")
        XCTAssertTrue(down.isEnabled)
        XCTAssertEqual(down.label, "Down, into darkness")

        type(app, "take lamp")
        _ = turn(app, 8)
        app.typeText("turn on lamp")
        element(app, "button.send").tap()
        _ = turn(app, 9)
        expectation(for: NSPredicate(format: "isHittable == true"), evaluatedWith: down)
        waitForExpectations(timeout: patience)
        XCTAssertEqual(value(app, "compass.down"), "open")
        XCTAssertEqual(down.label, "Down")
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
        XCTAssertEqual(value(app, "compass.west"), "open")
        XCTAssertEqual(element(app, "compass.west").label, "West")
        element(app, "compass.west").tap()
        waitForPlace(app, "Kitchen")
        XCTAssertEqual(element(app, "compass.east").label, "East")

        // The next room has a way down that the game decides in its own code.
        element(app, "compass.west").tap()
        waitForPlace(app, "Living Room")
        // Look-ahead finds the game refuses it while the trap door is hidden:
        // it is no way at all (feature 009).
        XCTAssertNil(way(app, "down"))
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
        XCTAssertNil(way(app, "south"), "the ways are this room's")
        XCTAssertEqual(element(app, "compass.north").label, "North")
        element(app, "compass.north").tap()
        XCTAssertTrue(turn(app, 2).hasPrefix("Forest Path"))
        // The way back has not been walked from here: it names nothing yet.
        XCTAssertEqual(element(app, "compass.south").label, "South")
        element(app, "compass.south").tap()
        waitForPlace(app, "North of House")
        // Walked both ways now: each end names the other.
        XCTAssertEqual(element(app, "compass.north").label, "North, to Forest Path")
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
            let open = places.filter { way(app, $0) == "open" }
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

        // The keyboard is still up, and the ways, the newest text and the
        // field are all in reach (feature 012).
        XCTAssertTrue(element(app, "compass.north").isHittable)
        XCTAssertTrue(element(app, "turn.1.text").isHittable)
        XCTAssertTrue(element(app, "field.command").isHittable)
        // Typing again needs no tap on the field.
        app.typeText("north")
        send.tap()
        XCTAssertTrue(turn(app, 2).hasPrefix("North of House"))
        XCTAssertEqual(element(app, "status.place").label, "North of House")
        XCTAssertTrue(element(app, "compass.north").isHittable)
        XCTAssertEqual(element(app, "compass.north").label, "North")
    }

    // S§ "Waiting states"; FR-015; US3 scenario 4.
    @MainActor
    func testAQuestionDarkensTheCompass() {
        let app = launch()
        _ = turn(app, 0)
        type(app, "quit")
        _ = turn(app, 1)
        XCTAssertEqual(element(app, "field.command").placeholderValue, "Type your answer")
        for place in ["north", "south", "west"] {
            XCTAssertTrue(element(app, "compass.\(place)").exists, place)
            XCTAssertFalse(element(app, "compass.\(place)").isEnabled, place)
        }
        type(app, "n")
        _ = turn(app, 2)
        XCTAssertEqual(value(app, "compass.north"), "open")
        XCTAssertTrue(element(app, "compass.north").isEnabled)
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
        XCTAssertEqual(element(app, "compass.north").label, "North")
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
    // top after "New game". It sits at the bottom, next to the field. With
    // the keyboard up the opening is longer than its room (feature 012), so
    // the keyboard is lowered first, by scrolling back.
    @MainActor
    func testAShortTranscriptSitsAtTheBottom() {
        let app = launch()
        _ = turn(app, 0)
        scrollBack(app)
        let transcript = element(app, "transcript")
        let opening = element(app, "turn.0.text")
        XCTAssertLessThan(opening.frame.height, transcript.frame.height)
        let settled = NSPredicate { _, _ in abs(transcript.frame.maxY - opening.frame.maxY) < 30 }
        expectation(for: settled, evaluatedWith: nil)
        waitForExpectations(timeout: patience)
    }

    /// A hand-made snapshot: a map of one place with every one of the twelve
    /// ways in one state (or none, for a state that is no way), given extra
    /// ways and a note. Debug builds draw it in place of a game when it is
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
        let isWay = ["open", "maybe", "door"].contains(state)
        var ways = !isWay ? [] : zip(places, labels).map { place, label in
            """
            {"way":"\(place)","label":"\(label)","state":"\(state)",\
            "command":\(offered ? "\"\(place)\"" : "null"),"leads_to":null,\
            "dark_ahead":\(darkAhead.contains(place)),"refused":false,"varies":false,\
            "returns":false,"spoken":"\(label), \(spokenTail)"}
            """
        }
        if !extras.isEmpty { ways.append(extras) }
        let noteJSON = note.map { "\"\($0)\"" } ?? "null"
        _ = known
        return """
            {"generation":1,"turn_count":1,"waiting":"\(waiting)","message":null,\
            "status":{"place":"A Place","score":null,"moves":null,"hours":9,"minutes":5},\
            "turns":[{"number":0,"command":null,"source":null,"text":"Ready.\\n\\n>"}],\
            "inventory":\(inventory),"light":\(light),\
            "map":{"note":\(noteJSON),"spoken":"Map. You are in A Place.","sheet":1,"sheets":1,\
            "here":1,"lamp":"unknown",\
            "places":[{"id":1,"x":0,"y":0,"inset":false,"name":"A Place","attached_to":null}],\
            "lines":[],"leads":[],"stubs":[],"ways":[\(ways.joined(separator: ","))]}}
            """
    }

    @MainActor
    private func launch(drawing snapshot: String) -> XCUIApplication {
        let app = XCUIApplication()
        app.launchEnvironment["LAMPLIGHT_FIXTURE_SNAPSHOT"] = snapshot
        app.launch()
        return app
    }

    // FR-014; US4 scenario 1; feature 012 S§8. No room on the bundled game's
    // walkthrough has unknown exits, so the map is drawn from a hand-made
    // snapshot: the layer's note is shown, and no way is drawn.
    @MainActor
    func testUnknownExitsAreSaidToBeUnknown() {
        var app = launch(drawing: handMade(
            state: "unknown", spokenTail: "unknown", known: false,
            note: "Exits unknown: type a direction"))
        _ = turn(app, 0)
        XCTAssertEqual(element(app, "map.note").label, "Exits unknown: type a direction")
        for place in places {
            XCTAssertFalse(element(app, "compass.\(place)").exists, place)
        }
        XCTAssertEqual(element(app, "map.here").label, "A Place")
        XCTAssertTrue(element(app, "field.command").isEnabled, "typing is still offered")
        XCTAssertEqual(element(app, "status.time").label, "9:05")
        XCTAssertFalse(element(app, "status.score").exists)
        app.terminate()

        // A place with no ways has no note and no way.
        app = launch(drawing: handMade(
            state: "closed", spokenTail: "no exit", known: true, note: nil))
        _ = turn(app, 0)
        XCTAssertTrue(element(app, "map.here").waitForExistence(timeout: patience))
        XCTAssertFalse(element(app, "map.note").exists)
        XCTAssertFalse(element(app, "compass.north").exists)
    }

    // FR-013; US2 scenario 7. The bundled game's first extra way out is over a
    // hundred commands in, so the row is drawn from a hand-made snapshot; that
    // a tap on an extra reaches the game is tested in the layer and bindings.
    @MainActor
    func testExtraWaysOutAreARowOfButtons() {
        let extras = """
            {"way":"Land","label":"Land","state":"open","command":"land","leads_to":"Sandy Beach",\
            "dark_ahead":false,"refused":false,"varies":false,"returns":false,\
            "spoken":"Land, to Sandy Beach"},\
            {"way":"Cross","label":"Cross","state":"maybe","command":"cross","leads_to":null,\
            "dark_ahead":false,"refused":false,"varies":false,"returns":false,\
            "spoken":"Cross, may be blocked"}
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
        // A way the layer does not offer is not among the map's ways.
        XCTAssertFalse(element(app, "compass.extra.2").exists)
        XCTAssertFalse(element(app, "map.note").exists)
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
        for place in ["north", "south", "west"] {
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
        XCTAssertEqual(element(app, "compass.north").label, "North")
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
        element(dark, "compass.west").tap()
        waitForPlace(dark, "Kitchen")
        try save("dark-ahead")
        element(dark, "compass.up").tap()
        waitForPlace(dark, "Attic")
        XCTAssertTrue(element(dark, "light.note").waitForExistence(timeout: patience))
        try save("in-the-dark")
        type(dark, "down")
        waitForPlace(dark, "Kitchen")
        element(dark, "compass.west").tap()
        waitForPlace(dark, "Living Room")
        type(dark, "take lamp")
        _ = turn(dark, 8)
        dark.typeText("turn on lamp")
        element(dark, "button.send").tap()
        _ = turn(dark, 9)
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
